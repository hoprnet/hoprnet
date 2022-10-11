import type { MultiaddrConnection } from '@libp2p/interface-connection'
import type { Instance as SimplePeer } from 'simple-peer'
import { durations, u8aToHex, defer, type DeferType } from '@hoprnet/hopr-utils'

import toIterable from 'stream-to-it'
import Debug from 'debug'
import type { RelayConnection } from '../relay/connection.js'
import { randomBytes } from 'crypto'
import { toU8aStream, encodeWithLengthPrefix, decodeWithLengthPrefix, eagerIterator } from '../utils/index.js'
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

const DEBUG_PREFIX = `hopr-connect`

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}:error`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

export const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(10)

export enum MigrationStatus {
  NOT_DONE,
  DONE
}

function getAbortableSource(source: StreamSource, signal?: AbortSignal) {
  if (signal != undefined) {
    source = abortableSource(source, signal) as StreamSource
  }

  return source
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
class WebRTCConnection implements MultiaddrConnection {
  // mutexes
  private _switchPromise: DeferType<WebRTCInitFinishedEvent>
  private _sinkSourceAttachedPromise: DeferType<SinkSourceAttachedEvent>

  // ICE signalling is done, either with no direct
  // connection possible or new direct connection is ready
  // to take over
  private _webRTCHandshakeFinished: boolean

  private _sourceMigrated: boolean
  private _sinkMigrated: boolean

  public destroyed: boolean
  public remoteAddr: MultiaddrConnection['remoteAddr']

  private sinkCreator: Promise<void>

  // Endpoint for libp2p
  public source: StreamSourceAsync

  // Underlying connection. Always points the connection that
  // is currently used.
  // At start, this is a relayed connection. Once WebRTC connection
  // is ready, it points to the WebRTC instance
  public conn: RelayConnection | SimplePeer

  private _id: string

  public tags: PeerConnectionType[]

  // Set magic *close* property to end connection
  // @dev this is done using meta programming in libp2p
  public timeline: MultiaddrConnection['timeline']

  constructor(
    private relayConn: RelayConnection,
    private testingOptions: HoprConnectTestingOptions,
    private options?: DialOptions
  ) {
    this.conn = relayConn

    this.destroyed = false
    this._switchPromise = defer<WebRTCInitFinishedEvent>()
    this._sinkSourceAttachedPromise = defer<SinkSourceAttachedEvent>()
    this._webRTCHandshakeFinished = false

    // Sink and source get migrated individually
    this._sourceMigrated = false
    this._sinkMigrated = false

    this.remoteAddr = this.conn.remoteAddr

    this.timeline = {
      open: Date.now()
    }

    // Initial state + fallback if WebRTC failed
    this.tags = [PeerConnectionType.WEBRTC_RELAYED]

    // Give each WebRTC connection instance a unique identifier
    this._id = u8aToHex(randomBytes(4), false)

    // @TODO fail if now WebRTC
    this.relayConn.state.channel?.on(
      'error',
      // not supposed to produce any errors
      this.onWebRTCError.bind(this)
    )
    this.relayConn.state.channel?.once(
      'connect',
      // not supposed to produce any errors
      this.onWebRTCConnect.bind(this)
    )

    this.relayConn.state.channel?.once('close', () => {
      this.destroyed = true
      this.timeline.close ??= Date.now()
    })

    // Attach a listener to WebRTC to cleanup state
    // and remove stale connection from internal libp2p state
    // once there is a disconnect, set magic *close* property in
    // timeline object
    this.relayConn.state.channel?.on('iceStateChange', (iceConnectionState: string, iceGatheringState: string) => {
      if (iceConnectionState === 'disconnected' && iceGatheringState === 'complete') {
        this.destroyed = true
        this.timeline.close ??= Date.now()
      }
    })

    this.source = getAbortableSource(this.createSource(), this.options?.signal) as AsyncIterable<StreamType>

    // Starts the sink and stores the handle to attach an error listener
    this.sinkCreator = this.sinkFunction()
    // Attaches the error listener in case of early failures
    this.sinkCreator.catch((err) => this.error('sink error thrown before sink attach', err.message))

    // Sink is passed as function handle, so we need to explicitly bind
    // an environment to it.
    this.sink = this.sink.bind(this)
  }

  /**
   * Takes a stream with messages to be sent to counterparty
   * @dev Resolves the sinkSourceAttached promise to start forwarding
   *      incoming messages
   * @param source stream with messages to be sent to counterparty
   * @returns
   */
  public sink(source: StreamSource) {
    setTimeout(this.onWebRTCError.bind(this), WEBRTC_UPGRADE_TIMEOUT).unref()

    let deferred = defer<void>()
    this.sinkCreator.catch(deferred.reject)
    this._sinkSourceAttachedPromise.resolve({
      type: ConnectionEventTypes.SINK_SOURCE_ATTACHED,
      value: async function* (this: WebRTCConnection) {
        try {
          yield* getAbortableSource(toU8aStream(source), this.options?.signal)
          deferred.resolve()
        } catch (err: any) {
          if (err.type === 'aborted' || err.code === 'ABORT_ERR') {
            // We can safely ignore abort errors
            deferred.resolve()
          } else {
            this.error(`sink error thrown`, err.message)
            deferred.reject(err)
          }
        }
      }.call(this)
    })

    return deferred.promise
  }

  /**
   * Log messages and add identity tag to distinguish multiple instances
   */
  private log(..._: any[]) {
    _log(`WRTC [${this._id}]`, ...arguments)
  }

  /**
   * Log verbose messages and add identity tag to distinguish multiple instances
   */
  // @ts-ignore temporarily unused
  private verbose(..._: any[]) {
    _verbose(`WRTC [${this._id}]`, ...arguments)
  }

  /**
   * Log errors and add identity tag to distinguish multiple instances
   */
  private error(..._: any[]) {
    _error(`WRTC [${this._id}]`, ...arguments)
  }

  private flow(..._: any[]) {
    _flow(`WRTC [${this._id}]`, ...arguments)
  }

  /**
   * Called once WebRTC is finished
   * @param err pass error during WebRTC upgrade
   */
  private onWebRTCError(err?: any) {
    if (this._webRTCHandshakeFinished) {
      // Already handled, so nothing to do
      return
    }
    this._webRTCHandshakeFinished = true

    if (err) {
      this.error(`ending WebRTC upgrade due error: ${err}`)
    }

    this._switchPromise.resolve({
      type: ConnectionEventTypes.WEBRTC_INIT_FINISHED,
      value: WebRTCResult.UNAVAILABLE
    })

    // @TODO fail if no WebRTC instance
    if (this.relayConn.state.channel) {
      setImmediate(this.relayConn.state.channel.destroy.bind(this.relayConn.state.channel))
    }
  }

  /**
   * Called once WebRTC was able to connect *directly* to counterparty
   */
  private async onWebRTCConnect() {
    if (this._webRTCHandshakeFinished) {
      // Already handled, so nothing to do
      return
    }
    this._webRTCHandshakeFinished = true

    // For testing, disable WebRTC upgrade
    // to test fallback connection in case of e.g.
    // bidirectional NATs
    if (this.testingOptions.__noWebRTCUpgrade) {
      this._switchPromise.resolve({
        type: ConnectionEventTypes.WEBRTC_INIT_FINISHED,
        value: WebRTCResult.UNAVAILABLE
      })
    } else {
      this._switchPromise.resolve({
        type: ConnectionEventTypes.WEBRTC_INIT_FINISHED,
        value: WebRTCResult.AVAILABLE
      })
    }
  }

  /**
   * Starts the communication with the counterparty through the
   * relayed connection. Passes messages through relayed connection
   * until WebRTC connection is available.
   */
  private async sinkFunction(): Promise<void> {
    let source: AsyncIterator<StreamType> | undefined
    let sourcePromise: Promise<PayloadEvent> | undefined

    this.flow(`FLOW: webrtc sink 1`)

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
      this.relayConn
        .sink(
          // start sinking status messages even if no source got
          // attached yet
          // this is important for sending webrtc signalling messages
          // even before payload messages are ready to send
          eagerIterator(
            async function* (this: WebRTCConnection): StreamSource {
              let webRTCFinished = false

              let leave = false
              let reasonToLeave: MigrationEvent | StreamEndedEvent | undefined
              this.flow(`FLOW: webrtc sink: loop started`)

              while (!leave) {
                this.flow(`FLOW: webrtc sink: loop iteration`)
                const promises: Promise<SinkEvent>[] = []

                // No source available, need to wait for it
                if (source == undefined) {
                  promises.push(this._sinkSourceAttachedPromise.promise)
                }

                // WebRTC handshake is not completed yet
                if (!webRTCFinished) {
                  promises.push(this._switchPromise.promise)
                }

                // Source already attached, wait for incoming messages
                if (source != undefined) {
                  if (sourcePromise == undefined) {
                    advanceIterator()
                  }
                  promises.push(sourcePromise as Promise<PayloadEvent>)
                }

                this.flow(`FLOW: webrtc sink: awaiting promises`)
                const relayConnResult = await Promise.race(promises)

                let toYield: Uint8Array | undefined

                switch (relayConnResult.type) {
                  case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
                    this.flow(`FLOW: webrtc sink: source attached, continue`)
                    source = relayConnResult.value[Symbol.asyncIterator]()
                    break
                  case ConnectionEventTypes.WEBRTC_INIT_FINISHED:
                    this.flow(`FLOW: webrtc sink: webrtc finished, handle`)
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
                        this.flow(`FLOW: webrtc sink: WebRTC upgrade finished but no connection, continue`)
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
                      this.flow(`FLOW: webrtc sink: received.done, break`)
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
              this.flow(`FLOW: webrtc sink: loop ended`)
              resolve(reasonToLeave as MigrationEvent | StreamEndedEvent)
            }.call(this)
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
        this.flow(`FLOW: sending UPGRADED to relay`)
        this.relayConn.sendUpgraded()

        // WebRTC handshake was successful, now using direct connection
        this._sinkMigrated = true
        if (this._sourceMigrated) {
          // Update state object once source *and* sink are migrated
          this.conn = this.relayConn.state.channel as SimplePeer
          if (!this.tags.includes(PeerConnectionType.WEBRTC_DIRECT)) {
            this.tags.push(PeerConnectionType.WEBRTC_DIRECT)
          }
        }
        try {
          await toIterable.sink(this.relayConn.state.channel as SimplePeer)(
            async function* (this: WebRTCConnection): StreamSource {
              let webRTCresult: PayloadEvent | SinkSourceAttachedEvent
              let toYield: Uint8Array | undefined
              let leave = false

              while (!leave) {
                // If no source attached, wait until there is one,
                // otherwise wait for messages
                if (source == undefined) {
                  webRTCresult = await this._sinkSourceAttachedPromise.promise
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
                    if (received.done || this.destroyed || this.relayConn.state.channel?.destroyed) {
                      leave = true

                      // WebRTC uses UDP, so we need to explicitly end the connection
                      toYield = encodeWithLengthPrefix(Uint8Array.of(MigrationStatus.DONE))
                      break
                    }

                    // this.log(
                    //   `sinking ${received.value.slice().length} bytes into webrtc[${
                    //     (this.relayConn.state.channel as any)._id
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
            }.call(this)
          )

          // End the stream
          this.relayConn.state.channel?.end()
        } catch (err) {
          this.error(`WebRTC sink err`, err)
          // Initiates Connection object teardown
          // by using meta programming
          this.timeline.close ??= Date.now()
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
  private async *createSource(): StreamSource {
    let migrated = false

    for await (const msg of this.relayConn.source) {
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
    const result = await this._switchPromise.promise

    switch (result.value) {
      // Anything can happen
      case WebRTCResult.UNAVAILABLE:
        throw Error(`Fatal error: Counterparty migrated stream but WebRTC is not avaialable`)
      // Forward messages from WebRTC instance
      case WebRTCResult.AVAILABLE:
        this._sourceMigrated = true

        if (this._sinkMigrated) {
          // Update state object once sink *and* source are migrated
          this.conn = this.relayConn.state.channel as SimplePeer
          if (!this.tags.includes(PeerConnectionType.WEBRTC_DIRECT)) {
            this.tags.push(PeerConnectionType.WEBRTC_DIRECT)
          }
        }

        this.log(`webRTC source handover done. Using direct connection to peer ${this.remoteAddr.getPeerId()}`)

        let done = false
        for await (const msg of this.relayConn.state.channel as SimplePeer) {
          // WebRTC tends to bundle multiple message into one chunk,
          // so we need to encode messages and decode them before passing
          // to libp2p
          const decoded = decodeWithLengthPrefix(msg.subarray())

          for (const decodedMsg of decoded) {
            const [finished, payload] = [decodedMsg.subarray(0, 1), decodedMsg.subarray(1)]

            // WebRTC is based on UDP, so we need to explicitly end the connection
            if (finished[0] == MigrationStatus.DONE) {
              this.log(`received DONE from WebRTC - ending stream`)
              done = true
              break
            }

            // this.log(`Getting NOT_DONE from WebRTC - ${msg.length} bytes`)
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

  /**
   * Closes the connection by closing WebRTC instance and closing
   * relayed connection. Log errors if any.
   * @param err
   * @returns
   */
  async close(err?: Error): Promise<void> {
    if (err) {
      this.error(`Error while attempting to close stream to ${this.remoteAddr}: ${err}`)
    }
    if (this.destroyed) {
      return
    }

    // Tell libp2p that connection is closed
    this.timeline.close = Date.now()
    this.destroyed = true

    try {
      // @TODO check if already closed
      this.relayConn.state.channel?.destroy()
    } catch (e) {
      this.error(`Error while destroying WebRTC instance to ${this.remoteAddr}: ${e}`)
    }

    try {
      await this.relayConn.close()
    } catch (e) {
      this.error(`Error while destroying relay connection to ${this.remoteAddr}: ${e}`)
    }

    this.log(`Connection to ${this.remoteAddr} has been destroyed`)
  }
}

export { WebRTCConnection }
