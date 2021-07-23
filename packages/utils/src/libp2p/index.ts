/*
 * Add a more usable API on top of LibP2P
 */
import PeerId from 'peer-id'
import { keys, PublicKey } from 'libp2p-crypto'
import multihashes from 'multihashes'
import type { PeerRoutingModule, Connection, MuxedStream } from 'libp2p'
import type LibP2P from 'libp2p'

import AbortController from 'abort-controller'
import Debug from 'debug'
import pipe from 'it-pipe'
import type { PromiseValue } from '../typescript'

export * from './privKeyToPeerId'
export * from './pubKeyToPeerId'
export * from './addressSorters'

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

/**
 * Check if PeerId contains a secp256k1 privKey
 * @param peer PeerId to check
 * @returns whether embedded privKey is a secp256k1 key
 */
export function isSecp256k1PeerId(peer: PeerId): boolean {
  const decoded = keys.keysPBM.PrivateKey.decode(peer.privKey.bytes)

  return decoded.Type == keys.keysPBM.KeyType.Secp256k1
}

const verbose = Debug('hopr-core:libp2p:verbose')
const logError = Debug(`hopr-core:libp2p:error`)

const DEFAULT_DHT_QUERY_TIMEOUT = 10000

export type DialOpts = {
  timeout: number
}

export type DialResponse =
  | {
      status: 'SUCCESS'
      resp: PromiseValue<ReturnType<LibP2P['dialProtocol']>>
    }
  | {
      status: 'E_TIMEOUT'
    }
  | {
      status: 'E_DIAL'
      error: string
      dhtContacted: boolean
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
  let timeout: NodeJS.Timeout

  const abort = new AbortController()
  signal = abort.signal
  timeout = setTimeout(() => {
    abort.abort()
    verbose(`timeout while trying to dial ${destination.toB58String()}`)
  }, opts.timeout || DEFAULT_DHT_QUERY_TIMEOUT)

  let err: any
  let struct: PromiseValue<ReturnType<LibP2P['dialProtocol']>> | null

  let addresses = (libp2p.peerStore.get(destination)?.addresses ?? []).map((addr) => addr.multiaddr.toString())

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
    logError(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`)
    clearTimeout(timeout)
    return { status: 'E_DIAL', error: err.message, dhtContacted: false }
  }

  // Try to get some fresh addresses from the DHT
  let dhtResponse: PromiseValue<ReturnType<PeerRoutingModule['findPeer']>>
  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtResponse = await libp2p._dht.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })
  } catch (err) {
    logError(`Querying the DHT for ${destination.toB58String()} failed. ${err.message}`)
  }

  const newAddresses = (dhtResponse?.multiaddrs ?? []).filter((addr) => addresses.includes(addr.toString()))

  if (signal.aborted) {
    return { status: 'E_TIMEOUT' }
  }

  // Only start a dial attempt if we have received new addresses
  if (newAddresses.length == 0) {
    clearTimeout(timeout)
    return { status: 'E_DIAL', error: 'No new addresses after contacting the DHT', dhtContacted: true }
  }

  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal })
    verbose(`Dial after DHT request successful`, struct)
  } catch (err) {
    logError(`Using new addresses after querying the DHT did not lead to a connection. Cannot connect. ${err.message}`)
    clearTimeout(timeout)
    return { status: 'E_DIAL', error: err.message, dhtContacted: true }
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
    logError(r)
    throw new Error(r.status)
  }
}

export async function libp2pSendMessageAndExpectResponse(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  message: Uint8Array,
  opts?: DialOpts
): Promise<Uint8Array[]> {
  const r = await dial(libp2p, destination, protocol, opts)
  if (r.status === 'SUCCESS') {
    return await pipe([message], r.resp.stream, async function collect(source: AsyncIterable<any>) {
      const vals = []
      for await (const val of source) {
        // Convert from BufferList to Uint8Array
        vals.push(Uint8Array.from(val.slice()))
      }
      return vals
    })
  }
  logError(r)
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
  return function libP2PHandler(args: LibP2PHandlerArgs): void {
    // Create the async iterable that we will use in the pipeline

    if (includeReply) {
      pipe(
        // prettier-ignore
        args.stream,
        async function* pipeToHandler(source: AsyncIterable<Uint8Array>) {
          for await (const msg of source) {
            // Convert from BufferList to Uint8Array
            yield await handlerFunction(Uint8Array.from(msg.slice()), args.connection.remotePeer)
          }
        },
        args.stream
      )
    } else {
      pipe(
        // prettier-ignore
        args.stream,
        async function collect(source: AsyncIterable<Uint8Array>) {
          for await (const msg of source) {
            // Convert from BufferList to Uint8Array
            await handlerFunction(Uint8Array.from(msg.slice()), args.connection.remotePeer)
          }
        }
      )
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
