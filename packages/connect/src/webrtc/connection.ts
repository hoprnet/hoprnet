import type { MultiaddrConnection } from '@libp2p/interface-connection'
import type { Instance as SimplePeer } from 'simple-peer'
import { durations, u8aToHex, defer, type DeferType } from '@hoprnet/hopr-utils'

import toIterable from 'stream-to-it'
import Debug from 'debug'
import type { RelayConnection } from '../relay/connection.js'
import { randomBytes } from 'crypto'
import { toU8aStream, encodeWithLengthPrefix, decodeWithLengthPrefix, eagerIterator } from '../utils/index.js'
import { abortableSource } from 'abortable-iterator'
import type {
  StreamSink,
  StreamResult,
  StreamType,
  StreamSource,
  StreamSourceAsync,
  HoprConnectTestingOptions
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
 */
class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferType<WebRTCInitFinishedEvent>
  private _sinkSourceAttachedPromise: DeferType<SinkSourceAttachedEvent>
  private _webRTCHandshakeFinished: boolean

  private _sourceMigrated: boolean
  private _sinkMigrated: boolean

  public destroyed: boolean
  public remoteAddr: MultiaddrConnection['remoteAddr']

  private sinkCreator: Promise<void>

  // @ts-ignore
  public sink: StreamSink
  public source: StreamSourceAsync

  public conn: RelayConnection | SimplePeer

  private _id: string

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

    this._sourceMigrated = false
    this._sinkMigrated = false

    this.remoteAddr = this.conn.remoteAddr

    this.timeline = {
      open: Date.now()
    }

    // Give each WebRTC connection instance a unique identifier
    this._id = u8aToHex(randomBytes(4), false)

    this.relayConn.getWebRTCInstance().on(
      'error',
      // not supposed to produce any errors
      this.onWebRTCError.bind(this)
    )
    this.relayConn.getWebRTCInstance().once(
      'connect',
      // not supposed to produce any errors
      this.onWebRTCConnect.bind(this)
    )

    // Attach a listener to WebRTC to cleanup state
    // and remove stale connection from internal libp2p state
    this.relayConn.getWebRTCInstance().on('iceStateChange', (iceConnectionState: string, iceGatheringState: string) => {
      if (iceConnectionState === 'disconnected' && iceGatheringState === 'complete') {
        this.destroyed = true
        this.timeline.close = this.timeline.close ?? Date.now()
      }
    })

    this.source = getAbortableSource(this.createSource(), this.options?.signal) as AsyncIterable<StreamType>

    this.sinkCreator = this.sinkFunction()
    this.sinkCreator.catch((err) => this.error('sink error thrown before sink attach', err.message))

    this.sink = this._sink.bind(this)
  }

  public _sink(source: StreamSource) {
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

    setImmediate(this.relayConn.getWebRTCInstance().destroy.bind(this.relayConn.getWebRTCInstance()))
  }

  /**
   * Called once WebRTC was able to connect to counterparty
   */
  private async onWebRTCConnect() {
    if (this._webRTCHandshakeFinished) {
      // Already handled, so nothing to do
      return
    }
    this._webRTCHandshakeFinished = true

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

    const nextMessage = () => {
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
                    nextMessage()
                  }
                  promises.push(sourcePromise as Promise<PayloadEvent>)
                }

                // (0.) Handle stream source attach
                // 1. Handle stream handover
                // 2. Handle stream messages
                this.flow(`FLOW: webrtc sink: awaiting promises`)
                const result = await Promise.race(promises)

                let toYield: Uint8Array | undefined

                switch (result.type) {
                  case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
                    this.flow(`FLOW: webrtc sink: source attached, continue`)
                    source = result.value[Symbol.asyncIterator]()
                    break
                  case ConnectionEventTypes.WEBRTC_INIT_FINISHED:
                    this.flow(`FLOW: webrtc sink: webrtc finished, handle`)
                    webRTCFinished = true
                    switch (result.value) {
                      case WebRTCResult.AVAILABLE:
                        reasonToLeave = { type: ConnectionEventTypes.MIGRATED }
                        leave = true
                        toYield = Uint8Array.of(MigrationStatus.DONE)
                        break
                      case WebRTCResult.UNAVAILABLE:
                        this.flow(`FLOW: webrtc sink: WebRTC upgrade finished but no connection, continue`)
                        // WebRTC upgrade finished but no connection possible
                        break
                      default:
                        throw Error(`Invalid WebRTC result. Received ${JSON.stringify(result)}`)
                    }
                    break
                  case ConnectionEventTypes.PAYLOAD:
                    if (result.value.done) {
                      this.flow(`FLOW: webrtc sink: received.done, break`)
                      reasonToLeave = { type: ConnectionEventTypes.STREAM_ENDED }
                      leave = true
                      break
                    }
                    toYield = Uint8Array.from([MigrationStatus.NOT_DONE, ...result.value.value])
                    nextMessage()
                    break
                  default:
                    throw Error(`Invalid result ${JSON.stringify(result)}`)
                }

                if (toYield != undefined) {
                  this.log(`sinking ${toYield.length} bytes into relayed connection`)
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

    switch (result.type) {
      case ConnectionEventTypes.STREAM_ENDED:
        // nothing to do
        return
      case ConnectionEventTypes.MIGRATED:
        // WebRTC is available, let's attach sink source to it
        this.flow(`FLOW: sending UPGRADED to relay`)
        this.relayConn.sendUpgraded()

        // WebRTC handshaerrorke was successful, now using direct connection
        this._sinkMigrated = true
        if (this._sourceMigrated) {
          // Update state object once source *and* sink are migrated
          this.conn = this.relayConn.getWebRTCInstance()
        }
        await toIterable.sink(this.relayConn.getWebRTCInstance())(
          async function* (this: WebRTCConnection): StreamSource {
            let result: PayloadEvent | SinkSourceAttachedEvent
            let toYield: Uint8Array | undefined
            let leave = false

            while (!leave) {
              // If no source attached, wait until there is one,
              // otherwise wait for messages
              if (source == undefined) {
                result = await this._sinkSourceAttachedPromise.promise
              } else {
                if (sourcePromise == undefined) {
                  nextMessage()
                }
                result = await (sourcePromise as Promise<PayloadEvent>)
              }

              switch (result.type) {
                case ConnectionEventTypes.SINK_SOURCE_ATTACHED:
                  source = result.value[Symbol.asyncIterator]()
                  break
                case ConnectionEventTypes.PAYLOAD:
                  const received = result.value

                  if (received.done || this.destroyed || this.relayConn.getWebRTCInstance().destroyed) {
                    leave = true
                    toYield = encodeWithLengthPrefix(Uint8Array.of(MigrationStatus.DONE))
                    break
                  }

                  this.log(
                    `sinking ${received.value.slice().length} bytes into webrtc[${
                      (this.relayConn.getWebRTCInstance() as any)._id
                    }]`
                  )

                  toYield = encodeWithLengthPrefix(
                    Uint8Array.from([MigrationStatus.NOT_DONE, ...received.value.slice()])
                  )

                  nextMessage()

                  break
              }

              if (toYield != undefined) {
                yield toYield
              }
            }
          }.call(this)
        )
    }
  }

  /**
   * Creates a source that yields messages from relayed connection
   * until a DONE is received. If a direct WebRTC connection is
   * available, yield messages from WebRTC instance
   */
  private async *createSource(): StreamSource {
    let migrated = false

    for await (const msg of this.relayConn.source) {
      if (msg.length == 0) {
        continue
      }

      const [migrationStatus, payload] = [msg.slice(0, 1), msg.slice(1)]

      // Handle sub-protocol
      switch (migrationStatus[0] as MigrationStatus) {
        case MigrationStatus.DONE:
          migrated = true
          break
        case MigrationStatus.NOT_DONE:
          this.log(`getting ${payload.length} bytes from relayed connection`)
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
      return
    }

    // Wait for finish of WebRTC handshake
    const result = await this._switchPromise.promise
    switch (result.value) {
      case WebRTCResult.UNAVAILABLE:
        throw Error(`Fatal error: Counterparty migrated stream but WebRTC is not avaialable`)
      case WebRTCResult.AVAILABLE:
        this._sourceMigrated = true

        if (this._sinkMigrated) {
          // Update state object once sink *and* source are migrated
          this.conn = this.relayConn.getWebRTCInstance()
        }

        this.log(`webRTC source handover done. Using direct connection to peer ${this.remoteAddr.getPeerId()}`)

        let done = false
        for await (const msg of this.relayConn.getWebRTCInstance()) {
          const decoded = decodeWithLengthPrefix(msg.slice())

          for (const decodedMsg of decoded) {
            const [finished, payload] = [decodedMsg.slice(0, 1), decodedMsg.slice(1)]

            if (finished[0] == MigrationStatus.DONE) {
              this.log(`received DONE from WebRTC - ending stream`)
              done = true
              break
            }

            this.log(`Getting NOT_DONE from WebRTC - ${msg.length} bytes`)
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
      this.relayConn.getWebRTCInstance().destroy()
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
