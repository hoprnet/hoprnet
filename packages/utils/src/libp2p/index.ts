/*
 * Add a more usable API on top of LibP2P
 */
import PeerId from 'peer-id'
import { keys, PublicKey } from 'libp2p-crypto'
import multihashes from 'multihashes'
import LibP2P from 'libp2p'
import AbortController from 'abort-controller'
import Multiaddr from 'multiaddr'
import type { Connection, MuxedStream } from 'libp2p'
import pipe from 'it-pipe'
import { Logger } from '../logger'

export * from './privKeyToPeerId'
export * from './peerIdToPubKey'
export * from './pubKeyFromPeerId'

/**
 * Regular expresion used to match b58Strings
 *
 */

export const b58StringRegex = /16Uiu2HA[A-Za-z0-9]{1,45}/i

/**
 * Takes a peerId and returns its corresponding public key.
 *
 * @param peerId the PeerId used to generate a public key
 */
export async function convertPubKeyFromPeerId(peerId: PeerId): Promise<PublicKey> {
  return keys.unmarshalPublicKey(multihashes.decode(peerId.toBytes()).digest)
}

/**
 *
 * Takes a B58String and converts them to a PublicKey
 *
 * @param string the B58String used to represent the PeerId
 */
export async function convertPubKeyFromB58String(b58string: string): Promise<PublicKey> {
  return await convertPubKeyFromPeerId(PeerId.createFromB58String(b58string))
}

/**
 *
 * Returns true or false if given string does not contain a b58string
 *
 * @param string arbitrary content with maybe a b58string
 */
export function hasB58String(content: string): Boolean {
  const hasMatcheableContent = content.match(b58StringRegex)
  if (hasMatcheableContent) {
    const [maybeB58String] = hasMatcheableContent
    const b58String = maybeB58String.substr(0, 53)
    return b58String.length === 53
  } else {
    return false
  }
}

/**
 *
 * Returns the b58String within a given content. Returns empty string if none is found.
 *
 * @param string arbitrary content with maybe a b58string
 */
export function getB58String(content: string): string {
  const hasMatcheableContent = content.match(b58StringRegex)
  if (hasMatcheableContent) {
    const [maybeB58String] = hasMatcheableContent
    const b58String = maybeB58String.substr(0, 53)
    return b58String
  } else {
    return ''
  }
}

const log = Logger.getLogger('hoprd.libp2p')

const DEFAULT_DHT_QUERY_TIMEOUT = 10000

export type DialOpts = {
  timeout: number
}

export type DialResponse =
  | {
      status: 'SUCCESS'
      resp: { stream: MuxedStream; protocol: string }
    }
  | {
      status: 'E_TIMEOUT'
    }
  | {
      status: 'E_DIAL'
      error: Error
      dht: boolean
    }
  | {
      status: 'E_DHT_QUERY'
      error: Error
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
  opts?: DialOpts
): Promise<DialResponse> {
  let signal: AbortSignal
  let timeout: NodeJS.Timeout | undefined

  const abort = new AbortController()
  signal = abort.signal
  timeout = setTimeout(() => {
    abort.abort()
    log.debug(`timeout while querying ${destination.toB58String()}`)
  }, opts.timeout || DEFAULT_DHT_QUERY_TIMEOUT)

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

  if ((err != null || struct == null) && libp2p._dht == undefined) {
    log.error(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT`, err)
    return { status: 'E_DIAL', error: err, dht: false }
  }

  // Try to get some fresh addresses from the DHT
  let dhtAddresses: Multiaddr[]

  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtAddresses = (await libp2p._dht.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })?.multiaddrs) ?? []
  } catch (err) {
    log.error(`Querying the DHT for ${destination.toB58String()} failed`, err)
    return { status: 'E_DHT_QUERY', error: err, query: destination }
  }

  const newAddresses = dhtAddresses.filter((addr) => addresses.includes(addr.toString()))

  // Only start a dial attempt if we have received new addresses
  if (signal.aborted || newAddresses.length > 0) {
    return { status: 'E_DIAL', error: new Error('No new addresses'), dht: true }
  }

  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal })
    log.debug(`Dial after DHT request successful`, struct)
  } catch (err) {
    log.error(`Using new addresses after querying the DHT did not lead to a connection. Cannot connect`, err)
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

export async function libp2pSendMessage(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  message: Uint8Array,
  opts?: DialOpts
) {
  const r = await dial(libp2p, destination, protocol, opts)
  if (r.status === 'SUCCESS') {
    pipe([message], r.resp.stream)
  } else {
    log.error(r)
    throw new Error(r.status)
  }
}

export async function libp2pSendMessageAndExpectResponse(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  message: Uint8Array,
  opts?: DialOpts
): Promise<Uint8Array> {
  const r = await dial(libp2p, destination, protocol, opts)
  if (r.status === 'SUCCESS') {
    return await pipe([message], r.resp.stream)
  }
  log.error(r)
  throw new Error(r.status)
}

/*
 *  LibP2P API uses async iterables which aren't particularly friendly to
 *  interact with - this function simply allows us to assign a handler
 *  function that is called on each 'message' of the stream.
 */
export type LibP2PHandlerArgs = { connection: Connection; stream: MuxedStream; protocol: string }
export type LibP2PHandlerFunction = (msg: Uint8Array, remotePeer: PeerId) => any
function generateHandler(handlerFunction: LibP2PHandlerFunction, includeReply = false) {
  // Return a function to be consumed by Libp2p.handle()
  return function libP2PHandler(args: LibP2PHandlerArgs) {
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
export function libp2pSubscribe(
  libp2p: LibP2P,
  protocol: string,
  handler: LibP2PHandlerFunction,
  includeReply = false
) {
  libp2p.handle([protocol], generateHandler(handler, includeReply))
}
