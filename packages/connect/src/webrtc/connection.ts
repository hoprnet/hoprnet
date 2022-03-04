import type { MultiaddrConnection } from 'libp2p-interfaces/src/transport/types'
import type { Instance as SimplePeer } from 'simple-peer'
import type PeerId from 'peer-id'
import { durations, u8aToHex, defer, type DeferType } from '@hoprnet/hopr-utils'

import toIterable from 'stream-to-it'
import Debug from 'debug'
import type { RelayConnection } from '../relay/connection'
import type Libp2p from 'libp2p'
import { randomBytes } from 'crypto'
import { toU8aStream, encodeWithLengthPrefix, decodeWithLengthPrefix, eagerIterator } from '../utils'
import abortable from 'abortable-iterator'
import type { Stream, StreamResult, StreamType, HoprConnectDialOptions } from '../types'
import assert from 'assert'

const DEBUG_PREFIX = `hopr-connect`

const _log = Debug(DEBUG_PREFIX)
const _verbose = Debug(`${DEBUG_PREFIX}:verbose`)
const _flow = Debug(`flow:${DEBUG_PREFIX}:error`)
const _error = Debug(`${DEBUG_PREFIX}:error`)

export const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(3)

export enum MigrationStatus {
  NOT_DONE,
  DONE
}

function getAbortableSource(source: Stream['source'], signal?: AbortSignal) {
  if (signal != undefined) {
    source = abortable(source, signal) as Stream['source']
  }

  return source
}

/**
 * Encapsulate state management and upgrade from relayed connection to
 * WebRTC connection
 */
class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferType<void>
  private _sinkSourceAttached: boolean
  private _sinkSourceAttachedPromise: DeferType<Stream['source']>
  private _webRTCHandshakeFinished: boolean
  private _webRTCAvailable: boolean
  private webRTCHandshakeTimeout?: NodeJS.Timeout

  private _sourceMigrated: boolean
  private _sinkMigrated: boolean

  public destroyed: boolean
  public remoteAddr: MultiaddrConnection['remoteAddr']
  public localAddr: MultiaddrConnection['localAddr']

  // @ts-ignore
  public sink: Stream['sink']
  public source: Stream['source']

  public conn: RelayConnection | SimplePeer

  private _id: string

  public timeline: MultiaddrConnection['timeline']

  constructor(
    private counterparty: PeerId,
    private connectionManager: Pick<Libp2p['connectionManager'], 'connections'>,
    private relayConn: RelayConnection,
    private channel: SimplePeer,
    private options?: HoprConnectDialOptions & { __noWebRTCUpgrade?: boolean }
  ) {
    this.conn = relayConn

    this.destroyed = false
    this._switchPromise = defer<void>()
    this._sinkSourceAttached = false
    this._sinkSourceAttachedPromise = defer<Stream['source']>()
    this._webRTCHandshakeFinished = false
    this._webRTCAvailable = false

    this._sourceMigrated = false
    this._sinkMigrated = false

    this.localAddr = this.conn.localAddr
    this.remoteAddr = this.conn.remoteAddr

    this.timeline = {
      open: Date.now()
    }

    this._id = u8aToHex(randomBytes(4), false)
    const errListener = this.afterWebRTCUpgrade.bind(this)

    this.channel.once('error', errListener)
    this.channel.once('connect', () => {
      this.channel.removeListener('error', errListener)
      this.onConnect()
    })

    // Attach a listener to WebRTC to cleanup state
    // and remove stale connection from internal libp2p state
    this.channel.on('iceStateChange', (iceConnectionState: string, iceGatheringState: string) => {
      if (iceConnectionState === 'disconnected' && iceGatheringState === 'complete') {
        this.timeline.close = Date.now()
        this.destroyed = true
        // HACK, @TODO remove this
        this.connectionManager.connections.delete(this.counterparty.toB58String())
      }
    })

    this.source = getAbortableSource(this.createSource(), this.options?.signal)

    let sinkCreator: Promise<void>
    this.sink = (source: Stream['source']) => {
      let deferred = defer<void>()
      sinkCreator.catch(deferred.reject)
      this._sinkSourceAttached = true
      this._sinkSourceAttachedPromise.resolve(
        async function* (this: WebRTCConnection) {
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
      )

      return deferred.promise
    }

    sinkCreator = this.sinkFunction()

    sinkCreator.catch((err) => this.error('sink error thrown before sink attach', err.message))
    this.verbose(`!!! sinkFunction`)

    this.webRTCHandshakeTimeout = setTimeout(this.afterWebRTCUpgrade.bind(this), WEBRTC_UPGRADE_TIMEOUT)
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
  private afterWebRTCUpgrade(err?: any) {
    if (this.webRTCHandshakeTimeout != undefined) {
      clearTimeout(this.webRTCHandshakeTimeout)
    }

    this.error(`ending WebRTC upgrade due error: ${err}`)
    this._webRTCHandshakeFinished = true
    this._webRTCAvailable = false
    this._switchPromise.resolve()

    setImmediate(() => {
      this.channel.destroy()
    })
  }

  /**
   * Called once WebRTC was able to connect to counterparty
   */
  private async onConnect() {
    if (this.webRTCHandshakeTimeout != undefined) {
      clearTimeout(this.webRTCHandshakeTimeout)
    }

    this._webRTCHandshakeFinished = true

    if (this.options?.__noWebRTCUpgrade) {
      this._webRTCAvailable = false
    } else {
      this._webRTCAvailable = true
    }

    this._switchPromise.resolve()
  }

  /**
   * Starts the communication with the counterparty through the
   * relayed connection. Passes messages through relayed connection
   * until WebRTC connection is available.
   */
  private async sinkFunction(): Promise<void> {
    type SinkType = Stream['source'] | StreamResult | void

    let source: AsyncIterator<StreamType> | undefined
    let sourcePromise: Promise<StreamResult> | void

    let sourceAttached = false

    this.flow(`FLOW: webrtc sink 1`)

    // handle sink stream of relay connection until it
    // either ends or webrtc becomes available
    await new Promise<void>((resolve, reject) =>
      this.relayConn
        .sink(
          // start sinking status messages even if no source got
          // attached yet
          // this is important for sending webrtc signalling messages
          // even before payload messages are ready to send
          eagerIterator(
            async function* (this: WebRTCConnection): Stream['source'] {
              let webRTCFinished = false

              let result: SinkType

              const next = () => {
                assert(source != undefined)
                sourcePromise = source.next()
                result = undefined
              }

              this.flow(`FLOW: webrtc sink: loop started`)

              while (true) {
                this.flow(`FLOW: webrtc sink: loop iteration`)
                const promises: Promise<SinkType>[] = []

                let resolvedPromiseName

                const pushPromise = (promise: Promise<SinkType>, name: string) => {
                  promises.push(
                    promise.then((res: any) => {
                      resolvedPromiseName = name
                      return res
                    })
                  )
                }

                // No source available, need to wait for it
                if (!sourceAttached) {
                  pushPromise(this._sinkSourceAttachedPromise.promise, 'sourceAttached')
                }

                // WebRTC handshake is not completed yet
                if (!webRTCFinished) {
                  pushPromise(this._switchPromise.promise, 'switch')
                }

                // Source already attached, wait for incoming messages
                if (sourceAttached) {
                  assert(source != undefined)
                  sourcePromise ??= source.next()
                  pushPromise(sourcePromise, 'source')
                }

                // (0.) Handle stream source attach
                // 1. Handle stream handover
                // 2. Handle stream messages
                this.flow(`FLOW: webrtc sink: awaiting promises`)
                result = await Promise.race(promises)
                this.flow(`FLOW: webrtc sink: promise resolved ${resolvedPromiseName}`)

                // Source got attached
                if (!sourceAttached && this._sinkSourceAttached) {
                  sourceAttached = true
                  source = (result as AsyncIterable<StreamType>)[Symbol.asyncIterator]()
                  this.flow(`FLOW: webrtc sink: source attached, continue`)
                  continue
                }

                // WebRTC is finished, now handle result
                if (!webRTCFinished && this._webRTCHandshakeFinished) {
                  webRTCFinished = true

                  if (this._webRTCAvailable) {
                    // Send DONE and migrate to direct WebRTC connection
                    this.flow(`FLOW: webrtc sink: webrtc finished, handle`)
                    // this.flow(`FLOW: switched to webrtc, will try to close relayed connection`)

                    yield Uint8Array.of(MigrationStatus.DONE)
                    break
                  } else {
                    // WebRTC upgrade finished but no connection
                    // possible
                    this.flow(`FLOW: webrtc sink: WebRTC upgrade finished but no connection, continue`)
                    continue
                  }
                }

                const received = result as StreamResult

                if (received.done) {
                  this.flow(`FLOW: webrtc sink: received.done, break`)
                  break
                }

                next()

                this.log(`sinking ${received.value.slice().length} bytes into relayed connection`)
                this.flow(`FLOW: webrtc sink: loop iteration ended`)
                yield Uint8Array.from([MigrationStatus.NOT_DONE, ...received.value.slice()])
              }
              this.flow(`FLOW: webrtc sink: loop ended`)
              resolve()
            }.call(this)
          )
        )
        // catch stream errors and forward error
        .catch(reject)
    )

    // Either stream is finished or WebRTC is available
    if (this._webRTCAvailable) {
      // WebRTC is available, let's attach sink source to it
      this.flow(`FLOW: sending UPGRADED to relay`)
      this.relayConn.sendUpgraded()

      // WebRTC handshake was successful, now using direct connection
      this._sinkMigrated = true
      if (this._sourceMigrated) {
        this.conn = this.channel
      }

      await toIterable.sink(this.channel)(
        async function* (this: WebRTCConnection): Stream['source'] {
          let result: SinkType

          while (true) {
            // If no source attached, wait until there is one,
            // otherwise wait for messages
            if (!sourceAttached) {
              result = await this._sinkSourceAttachedPromise.promise
            } else {
              assert(source != undefined)
              sourcePromise ??= source.next()
              result = await sourcePromise
            }

            // Handle attached source
            if (!sourceAttached && this._sinkSourceAttached) {
              sourceAttached = true
              source = (result as AsyncIterable<StreamType>)[Symbol.asyncIterator]()
              continue
            }

            const received = result as StreamResult

            if (received.done || this.destroyed || this.channel.destroyed) {
              yield encodeWithLengthPrefix(Uint8Array.of(MigrationStatus.DONE))
              break
            }

            assert(source != undefined)
            sourcePromise = source.next()

            this.log(`sinking ${received.value.slice().length} bytes into webrtc[${(this.channel as any)._id}]`)

            yield encodeWithLengthPrefix(Uint8Array.from([MigrationStatus.NOT_DONE, ...received.value.slice()]))
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
  private async *createSource(): Stream['source'] {
    for await (const msg of this.relayConn.source) {
      if (msg.length == 0) {
        continue
      }

      const [migrationStatus, payload] = [msg.slice(0, 1), msg.slice(1)]

      // Handle sub-protocol
      let done = false
      switch (migrationStatus[0] as MigrationStatus) {
        case MigrationStatus.DONE:
          done = true
          break
        case MigrationStatus.NOT_DONE:
          this.log(`getting ${payload.length} bytes from relayed connection`)
          yield payload
          break
        default:
          throw Error(`Invalid WebRTC migration status prefix. Got ${JSON.stringify(migrationStatus)}`)
      }

      if (done) {
        break
      }
    }

    // Wait for finish of WebRTC handshake
    if (!this._webRTCHandshakeFinished) {
      await this._switchPromise.promise
    }

    // If direct connection with WebRTC is possible, use it,
    // otherwise end stream
    if (this._webRTCAvailable) {
      this._sourceMigrated = true

      if (this._sinkMigrated) {
        this.conn = this.channel
      }

      this.log(`webRTC source handover done. Using direct connection to peer ${this.remoteAddr.getPeerId()}`)

      let done = false
      for await (const msg of this.channel) {
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
          this.flow(`FLOW: `)
          this.relayConn.sendUpgraded()
          break
        }
      }
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
      this.error(`Error while attempting to close stream: ${err}`)
    }
    if (this.destroyed) {
      return
    }

    // Tell libp2p that connection is closed
    this.timeline.close = Date.now()
    this.destroyed = true

    try {
      this.channel.destroy()
    } catch (err) {
      this.error(`Error while destroying WebRTC instance: ${err}`)
    }

    try {
      await this.relayConn.close()
    } catch (err) {
      this.error(`Error while destroying relay connection: ${err}`)
    }
  }
}

export { WebRTCConnection }
