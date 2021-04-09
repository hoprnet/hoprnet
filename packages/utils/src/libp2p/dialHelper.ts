/*
* Add a more usable API on top of LibP2P
*/
import LibP2P from 'libp2p'
import AbortController from 'abort-controller'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'
import Debug from 'debug'
import type { Connection, MuxedStream } from 'libp2p'
import pipe from 'it-pipe'

const verbose = Debug('hopr-core:libp2p:verbose')
const error = Debug(`hopr-core:libp2p:error`)

const DEFAULT_DHT_QUERY_TIMEOUT = 10000

export type DialOpts = {
  timeout: number
}

export type DialResponse = {
  status: 'SUCCESS'
  resp: { stream: MuxedStream, protocol: string }
} | {
  status: 'E_TIMEOUT'
} | {
  status: 'E_DIAL',
  error: Error,
  dht: boolean
} | {
  status: 'E_DHT_QUERY'
  error: Error,
  query: PeerId
}


/**
 * Combines libp2p methods such as dialProtocol and peerRouting.findPeer
 * to establish a connection.
 * Contains a baseline protection against dialing same addresses twice.
 * @param libp2p a libp2p instance
 * @param destination PeerId of the destination
 * @param protocols protocols to use
 * @param opts
 */
export async function dial(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  opts: DialOpts
): Promise<DialResponse> {
  let signal: AbortSignal
  let timeout: NodeJS.Timeout | undefined

  const abort = new AbortController()
  signal = abort.signal
  timeout = setTimeout(() => {
    abort.abort()
    verbose(`timeout while querying ${destination.toB58String()}`)
  }, opts.timeout)

  let err: any
  let struct: any

  let addresses = (libp2p.peerStore.get(destination)?.addresses ?? []).map((addr: any) => addr.multiaddr.toString())

  // Try to use known addresses
  if (addresses.length > 0) {
    try {
      struct = await libp2p.dialProtocol(destination, protocol, { signal })
    } catch (_err) {
      err = _err
    }
  }

  if (struct != null) {
    clearTimeout(timeout)
    return { status: 'SUCCESS', resp: struct }
  }

  if (signal.aborted) {
    return { status: 'E_TIMEOUT' }
  }

  if (((err != null || struct == null) && libp2p._dht == undefined)) {
    error(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`)
    return { status: 'E_DIAL', error: err, dht: false }
  }

  // Try to get some fresh addresses from the DHT
  let dhtAddresses: Multiaddr[]

  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtAddresses =
      (await libp2p._dht.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })?.multiaddrs) ?? []
  } catch (err) {
    error(`Querying the DHT for ${destination.toB58String()} failed. ${err.message}`)
    return { status: 'E_DHT_QUERY', error: err, query: destination}
  }

  const newAddresses = dhtAddresses.filter((addr) => addresses.includes(addr.toString()))

  // Only start a dial attempt if we have received new addresses
  if (signal.aborted || newAddresses.length > 0) {
    return { status: 'E_DIAL', error: new Error('No new addresses'), dht: true }
  }

  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal })
    verbose(`Dial after DHT request successful`, struct)
  } catch (err) {
    error(`Using new addresses after querying the DHT did not lead to a connection. Cannot connect. ${err.message}`)
    return { status: 'E_DIAL', error: err, dht: true }
  }

  if (signal.aborted) {
    return { status: 'E_TIMEOUT' }
  }

  if (struct != null) {
    clearTimeout(timeout)
    return { status: 'SUCCESS', resp: struct }
  }
  throw new Error('Missing error case in dial')
}

/*
*  LibP2P API uses async iterables which aren't particularly friendly to
*  interact with - this function simply allows us to assign a handler
*  function that is called on each 'message' of the stream.
*/
type LibP2PHandlerArgs = { connection: Connection; stream: MuxedStream; protocol: string }
type HandlerFunction = (msg: Uint8Array, remotePeer: PeerId) => any
function generateHandler(handlerFunction: HandlerFunction, includeReply = false) {
  // Return a function to be consumed by Libp2p.handle()
  return function libP2PHandler(args: LibP2PHandlerArgs){
    // Create the async iterable that we will use in the pipeline
    async function* pipeToHandler(source: AsyncIterable<Uint8Array>) {
      for await (const msg of source) {
        yield await handlerFunction(msg, args.connection.remotePeer)
      }
    }
    if (includeReply) {
      pipe(args.stream, pipeToHandler, args.stream)
    } else {
      pipe(args.stream, pipeToHandler)
    }
  }
}

// Subscribe to messages to a protocol with a function
export function subscribe(libp2p: LibP2P, protocol: string, handler: HandlerFunction, includeReply = false) {
  libp2p.handle([protocol], generateHandler(handler, includeReply))
}
