/*
 * Add a more usable API on top of LibP2P
 */
import PeerId from 'peer-id'
import { keys, PublicKey } from 'libp2p-crypto'
import multihashes from 'multihashes'
import type { Connection, MuxedStream } from 'libp2p'
import type LibP2P from 'libp2p'

import { debug } from '../process'
import pipe from 'it-pipe'
import { dial, type DialOpts } from './dialHelper'

export * from './addressSorters'
export * from './dialHelper'
export * from './pickVersion'
export * from './pubKeyToPeerId'
export * from './privKeyToPeerId'
export * from './relayCode'
export * from './verifySignatureFromPeerId'

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

/**
 * Asks libp2p to establish a connection to another node and
 * send message. If `includeReply` is set, wait for a response
 * @param libp2p libp2p instance
 * @param destination peer to connect to
 * @param protocol protocol to speak
 * @param message message to send
 * @param includeReply try to receive a reply
 * @param opts [optional] timeout
 */

export type libp2pSendMessage = ((
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  message: Uint8Array,
  includeReply: false,
  opts?: DialOpts
) => Promise<void>) &
  ((
    libp2p: LibP2P,
    destination: PeerId,
    protocol: string,
    message: Uint8Array,
    includeReply: true,
    opts?: DialOpts
  ) => Promise<Uint8Array[]>)

export async function libp2pSendMessage(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  message: Uint8Array,
  includeReply: boolean,
  opts?: DialOpts
): Promise<void | Uint8Array[]> {
  const r = await dial(libp2p, destination, protocol, opts)

  if (r.status !== 'SUCCESS') {
    logError(r)
    throw new Error(r.status)
  }

  if (includeReply) {
    const result = (await pipe(
      // prettier-ignore
      [message],
      r.resp.stream,
      async function collect(source: AsyncIterable<any>) {
        const vals = []
        for await (const val of source) {
          // Convert from BufferList to Uint8Array
          vals.push(Uint8Array.from(val.slice()))
        }
        return vals
      }
    )) as Uint8Array[]

    return result
  } else {
    await pipe(
      // prettier-ignore
      [message],
      r.resp.stream
    )
  }
}

/*
 *  LibP2P API uses async iterables which aren't particularly friendly to
 *  interact with - this function simply allows us to assign a handler
 *  function that is called on each 'message' of the stream.
 */
export type LibP2PHandlerArgs = { connection: Connection; stream: MuxedStream; protocol: string }
export type LibP2PHandlerFunction<T> = (msg: Uint8Array, remotePeer: PeerId) => T

type HandlerFunction<T> = (props: LibP2PHandlerArgs) => T

type ErrHandler = (msg: any) => void

type generateHandler = ((
  handlerFunction: LibP2PHandlerFunction<Promise<void> | void>,
  errHandler: ErrHandler,
  includeReply: false
) => HandlerFunction<void>) &
  ((
    handlerFunction: LibP2PHandlerFunction<Promise<Uint8Array>>,
    errHandler: ErrHandler,
    includeReply: true
  ) => HandlerFunction<Promise<void>>)

function generateHandler(
  handlerFunction: LibP2PHandlerFunction<Promise<void | Uint8Array> | void>,
  errHandler: ErrHandler,
  includeReply = false
): HandlerFunction<void> | HandlerFunction<Promise<void>> {
  // Return a function to be consumed by Libp2p.handle()

  if (includeReply) {
    return async function libP2PHandler(props: LibP2PHandlerArgs): Promise<void> {
      try {
        await pipe(
          // prettier-ignore
          props.stream,
          async function* pipeToHandler(source: AsyncIterable<Uint8Array>) {
            for await (const msg of source) {
              // Convert from BufferList to Uint8Array
              yield await handlerFunction(Uint8Array.from(msg.slice()), props.connection.remotePeer)
            }
          },
          props.stream
        )
      } catch (err) {
        // Mostly used to capture send errors
        errHandler(err)
      }
    }
  } else {
    return function libP2PHandler(props: LibP2PHandlerArgs): void {
      try {
        // End the send stream by sending nothing
        props.stream.sink((async function* () {})()).catch(errHandler)
      } catch (err) {
        errHandler(err)
      }

      pipe(
        // prettier-ignore
        props.stream,
        async function collect(source: AsyncIterable<Uint8Array>) {
          for await (const msg of source) {
            // Convert from BufferList to Uint8Array
            await handlerFunction(Uint8Array.from(msg.slice()), props.connection.remotePeer)
          }
        }
      )
    }
  }
}

/**
 * Generates a handler that pulls messages out of a stream
 * and feeds them to the given handler.
 * @param libp2p libp2p instance
 * @param protocol protocol to dial
 * @param handler called once another node requests that protocol
 * @param errHandler handle stream pipeline errors
 * @param includeReply try to receive a reply
 */

export type libp2pSubscribe = ((
  libp2p: LibP2P,
  protocol: string,
  handler: LibP2PHandlerFunction<Promise<void> | void>,
  errHandler: ErrHandler,
  includeReply: false
) => void) &
  ((
    libp2p: LibP2P,
    protocol: string,
    handler: LibP2PHandlerFunction<Promise<Uint8Array>>,
    errHandler: ErrHandler,
    includeReply: true
  ) => void)

export async function libp2pSubscribe(
  libp2p: LibP2P,
  protocol: string,
  handler: LibP2PHandlerFunction<Promise<void | Uint8Array> | void>,
  errHandler: ErrHandler,
  includeReply = false
): Promise<void> {
  await libp2p.handle([protocol], generateHandler(handler, errHandler, includeReply))
}
