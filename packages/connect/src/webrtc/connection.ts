import type { MultiaddrConnection } from '@libp2p/interface-connection'
import type { Instance as SimplePeer } from 'simple-peer'
import { durations, u8aToHex, defer, type DeferType, create_counter } from '@hoprnet/hopr-utils'

import toIterable from 'stream-to-it'
import Debug from 'debug'
import type { RelayConnectionInterface } from '../relay/connection.js'
import { randomBytes } from 'crypto'
import { encodeWithLengthPrefix, decodeWithLengthPrefix, eagerIterator } from '../utils/index.js'
import { abortableSource } from 'abortable-iterator'
import {
  type StreamResult,
  type StreamType,
  type StreamSource,
  type StreamSourceAsync,
  type HoprConnectTestingOptions,
  PeerConnectionType
} from '../types.js'
import assert from 'assert'
import type { DialOptions } from '@libp2p/interface-transport'

// @ts-ignore untyped library
import retimer from 'retimer'

const DEBUG_PREFIX = `hopr-connect`

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}:error`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

const directPackets = create_counter('connect_counter_webrtc_packets', 'Number of directly sent packets (WebRTC)')

export const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(10)

export enum MigrationStatus {
  NOT_DONE,
  DONE
}

enum ConnectionEventTypes {
  WEBRTC_INIT_FINISHED,
  SINK_SOURCE_ATTACHED,
  PAYLOAD,
  MIGRATED,
  STREAM_ENDED
}

enum WebRTCResult {
  AVAILABLE,
  UNAVAILABLE
}

type WebRTCInitFinishedEvent = {
  type: ConnectionEventTypes.WEBRTC_INIT_FINISHED
  value: WebRTCResult
}

type SinkSourceAttachedEvent = {
  type: ConnectionEventTypes.SINK_SOURCE_ATTACHED
  value: StreamSourceAsync
}

type PayloadEvent = {
  type: ConnectionEventTypes.PAYLOAD
  value: StreamResult
}

type MigrationEvent = {
  type: ConnectionEventTypes.MIGRATED
}

type StreamEndedEvent = {
  type: ConnectionEventTypes.STREAM_ENDED
}

type SinkEvent = PayloadEvent | SinkSourceAttachedEvent | WebRTCInitFinishedEvent

export interface WebRTCConnectionInterface extends MultiaddrConnection {
  tags: PeerConnectionType[]
  setOnClose: (closeHandler: () => void) => void
}
/**
 * Encapsulate state management and upgrade from relayed connection to
 * WebRTC connection
 *
 *          ┌─────────────────┐         ┌────────┐
 *          │Relay Connection ├────────►│        │
 *          └─────────────────┘         │Stream  │
 *                                  ┌──►│        │
 *                                  │   └────────┘
 *          ┌─────────────────┐     │
 *          │WebRTC           ├─────┘
 *          └─────────────────┘
 *
 * First forward messages from relayed connection and remove prefixes.
 * Once WebRTC connection is ready to take over, i.e. there was a
 * `connect` event, switch over to direct WebRTC connection.
 * @dev the handover happens transparently for libp2p
 */
export function WebRTCConnection(
  relayConn: RelayConnectionInterface,
  testingOptions: HoprConnectTestingOptions,
  onClose: (() => void) | undefined,
  options?: DialOptions
): WebRTCConnectionInterface {
  const state: {
    // mutexes
    _switchPromise: DeferType<WebRTCInitFinishedEvent>
    _sinkSourceAttachedPromise: DeferType<SinkSourceAttachedEvent>

    // ICE signalling is done, either with no direct
    // connection possible or new direct connection is ready
    // to take over
    _webRTCHandshakeFinished: boolean

    _sourceMigrated: boolean
    _sinkMigrated: boolean

    destroyed: boolean
  } = {
    destroyed: false,
    _switchPromise: defer<WebRTCInitFinishedEvent>(),
    _sinkSourceAttachedPromise: defer<SinkSourceAttachedEvent>(),
    _webRTCHandshakeFinished: false,

    // Sink and source get migrated individually
    _sourceMigrated: false,
    _sinkMigrated: false
    // Initial state + fallback if WebRTC failed
  }

  const timeline: MultiaddrConnection['timeline'] = {
    open: Date.now()
  }

  const tags = [PeerConnectionType.WEBRTC_RELAYED]

  const _id = u8aToHex(randomBytes(4), false)

  // Underlying connection. Always points the connection that
  // is currently used.
  // At start, this is a relayed connection. Once WebRTC connection
  // is ready, it points to the WebRTC instance

  const remoteAddr = relayConn.remoteAddr

  let webRTCTimeout: any | undefined

  relayConn.getCurrentChannel()?.once('close', () => {
    state.destroyed = true
    timeline.close ??= Date.now()
  })

  // Attach a listener to WebRTC to cleanup state
  // and remove stale connection from internal libp2p state
  // once there is a disconnect, set magic *close* property in
  // timeline object
  relayConn.getCurrentChannel()?.on('iceStateChange', (iceConnectionState: string, iceGatheringState: string) => {
    if (iceConnectionState === 'disconnected' && iceGatheringState === 'complete') {
      state.destroyed = true
      timeline.close ??= Date.now()
      onClose?.()
    }
  })
  /**
   * Log messages and add identity tag to distinguish multiple instances
   */
  const log = (...args: any[]) => {
    _log(`WRTC [${_id}]`, ...args)
  }

  /**
   * Log verbose messages and add identity tag to distinguish multiple instances
   */
  // @ts-ignore temporarily unused
  const verbose = (...args: any[]) => {
    _verbose(`WRTC [${_id}]`, ...args)
  }

  /**
   * Log errors and add identity tag to distinguish multiple instances
   */
  const error = (...args: any[]) => {
    _error(`WRTC [${_id}]`, ...args)
  }

  const flow = (...args: any[]) => {
    _flow(`WRTC [${_id}]`, ...args)
  }

  let onSinkError = (err: any) => {
    error(`sink threw error before source attach`, err)
  }

  const setOnClose = (closeHandler: () => void) => {
    onClose = closeHandler
  }

  /**
   * Called once WebRTC is finished
   * @param err pass error during WebRTC upgrade
   */
  const onWebRTCError = (err?: any) => {
    if (state._webRTCHandshakeFinished) {
      // Already handled, so nothing to do
      return
    }
    webRTCTimeout?.clear()
    state._webRTCHandshakeFinished = true

    if (err) {
      error(`ending WebRTC upgrade due error: ${err}`)
    }

    state._switchPromise.resolve({
      type: ConnectionEventTypes.WEBRTC_INIT_FINISHED,
      value: WebRTCResult.UNAVAILABLE
    })

    const currentChannel = relayConn.getCurrentChannel()
    // @TODO fail if no WebRTC instance
    if (currentChannel) {
      setImmediate(currentChannel.destroy.bind(currentChannel))
    }
  }

  /**
   * Called once WebRTC was able to connect *directly* to counterparty
   */
  const onWebRTCConnect = () => {
    if (state._webRTCHandshakeFinished) {
      // Already handled, so nothing to do
      return
    }
    webRTCTimeout?.clear()
    state._webRTCHandshakeFinished = true

    // For testing, disable WebRTC upgrade
    // to test fallback connection in case of e.g.
    // bidirectional NATs
    if (testingOptions.__noWebRTCUpgrade) {
      state._switchPromise.resolve({
        type: ConnectionEventTypes.WEBRTC_INIT_FINISHED,
        value: WebRTCResult.UNAVAILABLE
      })
    } else {
      state._switchPromise.resolve({
        type: ConnectionEventTypes.WEBRTC_INIT_FINISHED,
        value: WebRTCResult.AVAILABLE
      })
    }
  }

  /**
   * Takes a stream with messages to be sent to counterparty
   * @dev Resolves the sinkSourceAttached promise to start forwarding
   *      incoming messages
   * @param source stream with messages to be sent to counterparty
   * @returns
   */
  const sink = (source: StreamSource) => {
    webRTCTimeout = retimer(onWebRTCError, WEBRTC_UPGRADE_TIMEOUT)

    return new Promise<void>((resolve, reject) => {
      onSinkError = reject
      state._sinkSourceAttachedPromise.resolve({
        type: ConnectionEventTypes.SINK_SOURCE_ATTACHED,
        value: (async function* () {
          try {
            yield* options?.signal != undefined ? abortableSource(source, options.signal) : source
            resolve()
          } catch (err: any) {
            if (err.type === 'aborted' || err.code === 'ABORT_ERR') {
              // We can safely ignore abort errors
              resolve()
            } else {
              error(`sink error thrown`, err.message)
              reject(err)
            }
          }
        })()
      })
    })
  }

  /**
   * Starts the communication with the counterparty through the
   * relayed connection. Passes messages through relayed connection
   * until WebRTC connection is available.
   */
  const sinkFunction = async (): Promise<void> => {
    let source: AsyncIterator<StreamType> | undefined
    let sourcePromise: Promise<PayloadEvent> | undefined

    flow(`FLOW: webrtc sink 1`)

    const advanceIterator = () => {
      assert(source != undefined)
      sourcePromise = source.next().then((res) => ({
        type: ConnectionEventTypes.PAYLOAD,
        value: res
      }))
    }

    // handle sink stream of relay connection until it
    // either ends or webrtc becomes available
    const result = await new Promise<MigrationEvent | StreamEndedEvent>((resolve, reject) =>
      relayConn
        .sink(
          // start sinking status messages even if no source got
          // attached yet
          // this is important for sending webrtc signalling messages
          // even before payload messages are ready to send
          eagerIterator(
            (async function* (): StreamSource {
              let webRTCFinished = false

              let leave = false
              let reasonToLeave: MigrationEvent | StreamEndedEvent | undefined
              flow(`FLOW: webrtc sink: loop started`)

              while (!leave) {
                flow(`FLOW: webrtc sink: loop iteration`)
                const promises: Promise<SinkEvent>[] = []

                // No source available, need to wait for it
                if (source == undefined) {
                  promises.push(state._sinkSourceAttachedPromise.promise)
                }

                // WebRTC handshake is not completed yet
                if (!webRTCFinished) {
                  promises.push(state._switchPromise.promise)
                }

                // Source already attached, wait for incoming messages
                if (source != undefined) {
                  if (sourcePromise == undefined) {
                    advanceIterator()
                  }
                  promises.push(sourcePromise as Promise<PayloadEvent>)
                }

                flow(`FLOW: webrtc sink: awaiting promises`)
                const relayConnResult = await Promise.race(promises)

                let toYield: Uint8Array | undefined

                switch (relayConnResult.type) {
                  case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
                    flow(`FLOW: webrtc sink: source attached, continue`)
                    source = relayConnResult.value[Symbol.asyncIterator]()
                    break
                  case ConnectionEventTypes.WEBRTC_INIT_FINISHED:
                    flow(`FLOW: webrtc sink: webrtc finished, handle`)
                    webRTCFinished = true
                    switch (relayConnResult.value) {
                      // WebRTC is available, so notifiy counterparty and
                      // afterwards end stream
                      case WebRTCResult.AVAILABLE:
                        reasonToLeave = { type: ConnectionEventTypes.MIGRATED }
                        leave = true
                        toYield = Uint8Array.of(MigrationStatus.DONE)
                        break
                      // Direct WebRTC connection is not available, e.g. due to
                      // bidirectional NAT, so stick to relayed conneciton
                      case WebRTCResult.UNAVAILABLE:
                        flow(`FLOW: webrtc sink: WebRTC upgrade finished but no connection, continue`)
                        // WebRTC upgrade finished but no connection possible
                        break
                      default:
                        throw Error(`Invalid WebRTC result. Received ${JSON.stringify(relayConnResult)}`)
                    }
                    break
                  // Forward payload messages to libp2p
                  case ConnectionEventTypes.PAYLOAD:
                    // Stream might end without any upgrade
                    if (relayConnResult.value.done) {
                      flow(`FLOW: webrtc sink: received.done, break`)
                      reasonToLeave = { type: ConnectionEventTypes.STREAM_ENDED }
                      leave = true
                      break
                    }
                    toYield = Uint8Array.from([MigrationStatus.NOT_DONE, ...relayConnResult.value.value])
                    advanceIterator()
                    break
                  default:
                    throw Error(`Invalid result ${JSON.stringify(relayConnResult)}`)
                }

                if (toYield != undefined) {
                  // this.log(`sinking ${toYield.length} bytes into relayed connection`)
                  yield toYield
                }
              }
              flow(`FLOW: webrtc sink: loop ended`)
              resolve(reasonToLeave as MigrationEvent | StreamEndedEvent)
            })()
          )
        )
        // catch stream errors and forward them
        .catch(reject)
    )

    // Relay connection *has* ended, let's find out why
    switch (result.type) {
      case ConnectionEventTypes.STREAM_ENDED:
        // nothing to do
        return
      case ConnectionEventTypes.MIGRATED:
        // WebRTC is available, let's attach sink source to it
        flow(`FLOW: sending UPGRADED to relay`)
        relayConn.sendUpgraded()

        // WebRTC handshake was successful, now using direct connection
        state._sinkMigrated = true
        if (state._sourceMigrated) {
          // Update state object once source *and* sink are migrated
          if (!tags.includes(PeerConnectionType.WEBRTC_DIRECT)) {
            tags.push(PeerConnectionType.WEBRTC_DIRECT)
          }
        }
        try {
          await toIterable.sink(relayConn.getCurrentChannel() as SimplePeer)(
            (async function* (): StreamSource {
              let webRTCresult: PayloadEvent | SinkSourceAttachedEvent
              let toYield: Uint8Array | undefined
              let leave = false

              while (!leave) {
                // If no source attached, wait until there is one,
                // otherwise wait for messages
                if (source == undefined) {
                  webRTCresult = await state._sinkSourceAttachedPromise.promise
                } else {
                  if (sourcePromise == undefined) {
                    advanceIterator()
                  }
                  webRTCresult = await (sourcePromise as Promise<PayloadEvent>)
                }

                switch (webRTCresult.type) {
                  case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
                    // Libp2p has started to send messages through this connection,
                    // so forward these messages
                    // @dev this can happen *before* or *after* WebRTC upgrade -
                    //      or never!
                    source = webRTCresult.value[Symbol.asyncIterator]()
                    break
                  case ConnectionEventTypes.PAYLOAD:
                    const received = webRTCresult.value

                    // Anything can happen
                    if (received.done || state.destroyed || relayConn.getCurrentChannel()?.destroyed) {
                      leave = true

                      // WebRTC uses UDP, so we need to explicitly end the connection
                      toYield = encodeWithLengthPrefix(Uint8Array.of(MigrationStatus.DONE))
                      break
                    }

                    directPackets.increment()

                    // log(
                    //   `sinking ${received.value.slice().length} bytes into webrtc[${
                    //     (relayConn.getCurrentChannel() as any)._id
                    //   }]`
                    // )

                    // WebRTC tends to send multiple messages in one chunk, so add a
                    // length prefix to split messages when receiving them
                    toYield = encodeWithLengthPrefix(
                      Uint8Array.from([MigrationStatus.NOT_DONE, ...received.value.subarray()])
                    )

                    advanceIterator()

                    break
                  default:
                    throw Error(`Received invalid result. Got ${JSON.stringify(result)}`)
                }

                if (toYield != undefined) {
                  yield toYield
                }
              }
            })()
          )

          // End the stream
          relayConn.getCurrentChannel()?.end()
        } catch (err) {
          error(`WebRTC sink err`, err)
          // Initiates Connection object teardown
          // by using meta programming
          timeline.close ??= Date.now()
          onClose?.()
        }
    }
  }

  /**
   * Creates a connection endpoint for libp2p.
   *
   * First yield all messages from relayed connection. Once
   * migration happenend, forward messages coming from WebRTC
   * instance.
   */
  async function* createSource(): StreamSource {
    let migrated = false

    for await (const msg of relayConn.source) {
      if (msg.length == 0) {
        continue
      }

      const [migrationStatus, payload] = [msg.subarray(0, 1), msg.subarray(1)]

      // Handle sub-protocol
      switch (migrationStatus[0] as MigrationStatus) {
        case MigrationStatus.DONE:
          migrated = true
          break
        case MigrationStatus.NOT_DONE:
          // this.log(`getting ${payload.length} bytes from relayed connection`)
          yield payload
          break
        default:
          throw Error(`Invalid WebRTC migration status prefix. Got ${JSON.stringify(migrationStatus)}`)
      }

      if (migrated) {
        break
      }
    }

    if (!migrated) {
      // Stream has ended but no migration happened
      return
    }

    // Wait for finish of WebRTC handshake
    const result = await state._switchPromise.promise

    switch (result.value) {
      // Anything can happen
      case WebRTCResult.UNAVAILABLE:
        throw Error(`Fatal error: Counterparty migrated stream but WebRTC is not avaialable`)
      // Forward messages from WebRTC instance
      case WebRTCResult.AVAILABLE:
        state._sourceMigrated = true

        if (state._sinkMigrated) {
          // Update state object once sink *and* source are migrated
          if (!tags.includes(PeerConnectionType.WEBRTC_DIRECT)) {
            tags.push(PeerConnectionType.WEBRTC_DIRECT)
          }
        }

        log(`webRTC source handover done. Using direct connection to peer ${relayConn.counterparty.toString()}`)

        let done = false
        for await (const chunkBuffer of relayConn.getCurrentChannel() as SimplePeer) {
          // Node.js emits Buffer instances, so turn them into Uint8Arrays.
          const chunk = new Uint8Array(chunkBuffer.buffer, chunkBuffer.byteOffset, chunkBuffer.byteLength)
          // WebRTC tends to bundle multiple message into one chunk,
          // so we need to encode messages and decode them before passing
          // to libp2p
          const decoded = decodeWithLengthPrefix(chunk)

          for (const decodedMsg of decoded) {
            const [finished, payload] = [decodedMsg.subarray(0, 1), decodedMsg.subarray(1)]

            // WebRTC is based on UDP, so we need to explicitly end the connection
            if (finished[0] == MigrationStatus.DONE) {
              log(`received DONE from WebRTC - ending stream`)
              done = true
              break
            }

            // log(`Getting NOT_DONE from WebRTC - ${chunk.length} bytes`)
            yield payload
          }

          if (done) {
            break
          }
        }
        break
      default:
        throw Error(`Invalid result. Received ${JSON.stringify(result)}`)
    }
  }

  // @TODO fail if no WebRTC
  relayConn.getCurrentChannel()?.on(
    'error',
    // not supposed to produce any errors
    onWebRTCError
  )
  relayConn.getCurrentChannel()?.once(
    'connect',
    // not supposed to produce any errors
    onWebRTCConnect
  )

  sinkFunction().catch(onSinkError)

  /**
   * Closes the connection by closing WebRTC instance and closing
   * relayed connection. Log errors if any.
   * @param err
   * @returns
   */
  const close = async (err?: Error): Promise<void> => {
    if (err) {
      error(`Error while attempting to close stream to ${remoteAddr}: ${err}`)
    }
    if (state.destroyed) {
      return
    }

    // Tell libp2p that connection is closed
    timeline.close = Date.now()
    onClose?.()
    state.destroyed = true

    try {
      // @TODO check if already closed
      relayConn.getCurrentChannel()?.destroy()
    } catch (e) {
      error(`Error while destroying WebRTC instance to ${remoteAddr}: ${e}`)
    }

    try {
      await relayConn.close()
    } catch (e) {
      error(`Error while destroying relay connection to ${remoteAddr}: ${e}`)
    }

    log(`Connection to ${remoteAddr} has been destroyed`)
  }

  return {
    sink,
    source: options?.signal != undefined ? abortableSource(createSource(), options.signal) : createSource(),
    close,
    tags,
    remoteAddr,
    // Set magic *close* property to end connection
    // @dev this is done using meta programming in libp2p
    timeline,
    setOnClose
  }
}
