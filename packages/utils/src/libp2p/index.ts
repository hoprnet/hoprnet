/*
 * Add a more usable API on top of LibP2P
 */
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Components } from '@libp2p/interfaces/components'
import type { Connection, ProtocolStream } from '@libp2p/interface-connection'

import { debug } from '../process/index.js'
import { pipe } from 'it-pipe'
import { dial } from './dialHelper.js'
import { abortableSource } from 'abortable-iterator'
import { TimeoutController } from 'timeout-abort-controller'

export * from './addressSorters.js'
export * from './dialHelper.js'
export * from './pickVersion.js'
export * from './pubKeyToPeerId.js'
export * from './privKeyToPeerId.js'
export * from './relayCode.js'
export * from './verifySignatureFromPeerId.js'
export * from './connection.js'

/**
 * Regular expresion used to match b58Strings
 *
 */
export const b58StringRegex = /16Uiu2HA[A-Za-z0-9]{45}/i

/**
 *
 * Returns true or false if given string does not contain a b58string
 *
 * @param content arbitrary content with maybe a b58string
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
 * @param content arbitrary content with maybe a b58string
 */
export function getB58String(content: string): string {
  const hasMatcheableContent = content.match(b58StringRegex)
  if (hasMatcheableContent) {
    const [maybeB58String] = hasMatcheableContent
    const b58String = maybeB58String.substring(0, 53)
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
  return peer.type === 'secp256k1'
}

const logError = debug(`hopr-core:libp2p:error`)

const DEFAULT_SEND_TIMEOUT = 10_000

/**
 * Asks libp2p to establish a connection to another node and
 * send message. If `includeReply` is set, wait for a response
 * @param components libp2p components
 * @param destination peer to connect to
 * @param protocols protocols to speak
 * @param message message to send
 * @param includeReply try to receive a reply
 * @param opts [optional] timeout
 */
export async function libp2pSendMessage<T extends boolean>(
  components: Components,
  destination: PeerId,
  protocols: string | string[],
  message: Uint8Array,
  includeReply: T,
  opts: {
    timeout?: number
  } = {}
): Promise<T extends true ? Uint8Array[] : void> {
  // Components is not part of interface
  const r = await dial(components, destination, protocols)

  if (r.status !== 'SUCCESS') {
    logError(r)
    throw Error(`Failed to dial destination while sending message. Status: ${r.status}`)
  }

  const timeoutController = new TimeoutController(opts.timeout ?? DEFAULT_SEND_TIMEOUT)

  if (includeReply) {
    try {
      const result = await pipe(
        // prettier-ignore
        abortableSource([message], timeoutController.signal),
        r.resp.stream,
        async function collect(source: AsyncIterable<Uint8Array>) {
          const vals: Uint8Array[] = []
          for await (const val of source) {
            // Convert from potential BufferList to Uint8Array
            vals.push(Uint8Array.from(val.slice()))
          }
          return vals
        }
      )
      return result as any
    } catch (err) {
      throw Error(`Could not send message to ${destination.toString()} due to "${err?.message}".`)
    } finally {
      timeoutController.clear()
    }

    return [] as any // limitation of Typescript
  } else {
    try {
      await pipe(
        // prettier-ignore
        abortableSource([message], timeoutController.signal),
        r.resp.stream
      )
    } catch (err) {
      throw Error(`Could not send message to ${destination.toString()} due to "${err?.message}".`)
    } finally {
      timeoutController.clear()
    }
  }
}

/*
 *  LibP2P API uses async iterables which aren't particularly friendly to
 *  interact with - this function simply allows us to assign a handler
 *  function that is called on each 'message' of the stream.
 */
export type LibP2PHandlerArgs = { connection: Connection; stream: ProtocolStream['stream']; protocol: string }
export type LibP2PHandlerFunction<T> = (msg: Uint8Array, remotePeer: PeerId) => T

type HandlerFunction<T> = (props: LibP2PHandlerArgs) => T

type ErrHandler = (msg: any) => void

function generateHandler<T extends boolean>(
  handlerFunction: LibP2PHandlerFunction<T extends true ? Promise<Uint8Array> : Promise<void> | void>,
  errHandler: ErrHandler,
  includeReply: T
): HandlerFunction<T extends true ? Promise<void> : void> {
  // Return a function to be consumed by Libp2p.handle()

  if (includeReply) {
    return async function libP2PHandler(props: LibP2PHandlerArgs): Promise<void> {
      try {
        await pipe(
          // prettier-ignore
          props.stream,
          async function* pipeToHandler(source: AsyncIterable<Uint8Array>) {
            for await (const msg of source) {
              // Convert from potential BufferList to Uint8Array
              yield (await handlerFunction(
                Uint8Array.from(msg.slice()),
                props.connection.remotePeer
              )) as Promise<Uint8Array>
            }
          },
          props.stream
        )
      } catch (err) {
        // Mostly used to capture send errors
        errHandler(err)
      }
    } as any // Limitation of Typescript
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
            // Convert from potential BufferList to Uint8Array
            await handlerFunction(Uint8Array.from(msg.slice()), props.connection.remotePeer)
          }
        }
      )
    } as any // Limitation of Typescript
  }
}

/**
 * Generates a handler that pulls messages out of a stream
 * and feeds them to the given handler.
 * @param components libp2p components
 * @param protocols protocol to dial
 * @param handler called once another node requests that protocol
 * @param errHandler handle stream pipeline errors
 * @param includeReply try to receive a reply
 */
export async function libp2pSubscribe<T extends boolean>(
  components: Components,
  protocols: string | string[],
  handler: LibP2PHandlerFunction<T extends true ? Promise<Uint8Array> : Promise<void> | void>,
  errHandler: ErrHandler,
  includeReply: T
): Promise<void> {
  await components.getRegistrar().handle(protocols, generateHandler(handler, errHandler, includeReply))
}
