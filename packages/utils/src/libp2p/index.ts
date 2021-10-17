/*
 * Add a more usable API on top of LibP2P
 */
import PeerId from 'peer-id'
import { keys, PublicKey } from 'libp2p-crypto'
import multihashes from 'multihashes'
import type { Connection, MuxedStream } from 'libp2p'
import type LibP2P from 'libp2p'

import { debug } from '../debug'
import pipe from 'it-pipe'
import { dial } from './dialHelper'

export * from './privKeyToPeerId'
export * from './pubKeyToPeerId'
export * from './addressSorters'
export * from './verifySignatureFromPeerId'
export * from './dialHelper'
export * from './pickVersion'

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

const logError = debug(`hopr-core:libp2p:error`)

export type DialOpts = {
  timeout: number
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
  logError('libp2p error', r)
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
