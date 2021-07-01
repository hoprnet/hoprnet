/// <reference path="../@types/stream-to-it.ts" />
/// <reference path="../@types/libp2p.ts" />

import type { ConnectionManager, DialOptions, MultiaddrConnection, Stream, StreamResult } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import type PeerId from 'peer-id'
import { durations, u8aToHex } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'
import Debug from 'debug'
import type { RelayConnection } from '../relay/connection'
import { randomBytes } from 'crypto'
import { toU8aStream, encodeWithLengthPrefix, decodeWithLengthPrefix, eagerIterator } from '../utils'
import abortable from 'abortable-iterator'

const DEBUG_PREFIX = `hopr-connect`

const _log = Debug(DEBUG_PREFIX)
const _error = Debug(DEBUG_PREFIX.concat(`error`))

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

class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferredPromise<void>
  private _sinkSourceAttached: boolean
  private _sinkSourceAttachedPromise: DeferredPromise<Stream['source']>
  private _webRTCStateKnown: boolean
  private _webRTCAvailable: boolean
  private webRTCUpgradeTimeout?: NodeJS.Timeout

  private _sourceMigrated: boolean
  private _sinkMigrated: boolean

  public destroyed: boolean

  public remoteAddr: MultiaddrConnection['remoteAddr']
  public localAddr: MultiaddrConnection['localAddr']

  public sink: Stream['sink']
  public source: Stream['source']

  public conn: RelayConnection | SimplePeer

  private _id: string

  public timeline: MultiaddrConnection['timeline']

  constructor(
    private counterparty: PeerId,
    private connectionManager: ConnectionManager,
    private relayConn: RelayConnection,
    private channel: SimplePeer,
    private options?: DialOptions & { __noWebRTCUpgrade?: boolean }
  ) {
    this.conn = relayConn

    this.destroyed = false
    this._switchPromise = Defer<void>()
    this._sinkSourceAttached = false
    this._sinkSourceAttachedPromise = Defer<Stream['source']>()
    this._webRTCStateKnown = false
    this._webRTCAvailable = false

    this._sourceMigrated = false
    this._sinkMigrated = false

    this.remoteAddr = relayConn.remoteAddr
    this.localAddr = relayConn.localAddr

    this.timeline = {
      open: Date.now()
    }

    this._id = u8aToHex(randomBytes(4), false)
    const errListener = this.endWebRTCUpgrade.bind(this)

    this.channel.once('error', errListener)
    this.channel.once('connect', () => {
      this.channel.removeListener('error', errListener)
      this.onConnect()
    })

    this.channel.on('iceStateChange', (iceConnectionState: string, iceGatheringState: string) => {
      if (iceConnectionState === 'disconnected' && iceGatheringState === 'complete') {
        this.timeline.close = Date.now()
        this.destroyed = true
        // HACK, @TODO remove this
        this.connectionManager.connections.delete(this.counterparty.toB58String())
      }
    })

    this.source = getAbortableSource(this.createSource(), this.options?.signal)

    this.sink = this._sink.bind(this)

    this.sinkFunction()

    this.webRTCUpgradeTimeout = setTimeout(this.endWebRTCUpgrade.bind(this), WEBRTC_UPGRADE_TIMEOUT)
  }

  private log(..._: any[]) {
    _log(`WRTC [${this._id}]`, ...arguments)
  }

  private error(..._: any[]) {
    _error(`WRTC [${this._id}]`, ...arguments)
  }

  private endWebRTCUpgrade(err?: any) {
    if (this.webRTCUpgradeTimeout != undefined) {
      clearTimeout(this.webRTCUpgradeTimeout)
    }

    this.error(`ending WebRTC upgrade due error: ${err}`)
    this._webRTCStateKnown = true
    this._webRTCAvailable = false
    this._switchPromise.resolve()

    setImmediate(() => {
      this.channel.destroy()
    })
  }

  private async onConnect() {
    if (this.webRTCUpgradeTimeout != undefined) {
      clearTimeout(this.webRTCUpgradeTimeout)
    }

    this._webRTCStateKnown = true

    if (this.options?.__noWebRTCUpgrade) {
      this._webRTCAvailable = false
    } else {
      this._webRTCAvailable = true
    }

    console.log(`inside connect`)
    // @TODO could be mixed up
    this._switchPromise.resolve()
  }

  private async _sink(source: Stream['source']): Promise<void> {
    this._sinkSourceAttached = true
    this._sinkSourceAttachedPromise.resolve(getAbortableSource(toU8aStream(source), this.options?.signal))
  }

  private async sinkFunction(): Promise<void> {
    console.log(`sink called`)
    type SinkType = Stream['source'] | StreamResult | void

    let source: Stream['source'] | undefined
    let sourcePromise: Promise<StreamResult> | undefined

    let webRTCFinished = false

    await new Promise<void>((resolve) =>
      this.relayConn.sink(
        eagerIterator(async function* (this: WebRTCConnection): Stream['source'] {
          let result: SinkType

          if (this._webRTCAvailable) {
            yield Uint8Array.of(MigrationStatus.DONE)
            return
          }

          while (true) {
            console.log(`iteration`)
            const promises: Promise<SinkType>[] = []

            if (source == undefined) {
              console.log(`sinkSourceAttached`)
              promises.push(this._sinkSourceAttachedPromise.promise)
            }

            if (!webRTCFinished) {
              console.log(`webrtc state`)
              promises.push(this._switchPromise.promise)
            }

            if (source != undefined) {
              console.log(`source`)
              sourcePromise ??= source.next()
              promises.push(sourcePromise)
            }

            console.log(promises)
            // 1. Handle stream handover
            // 2. Handle stream messages
            result = await Promise.race(promises)

            if (this._sinkSourceAttached && source == undefined) {
              source = result as Stream['source']
              continue
            }

            if (this._webRTCAvailable) {
              webRTCFinished = true

              yield Uint8Array.of(MigrationStatus.DONE)

              break
            }

            if (result == undefined && this._webRTCStateKnown) {
              webRTCFinished = true
              // WebRTC upgrade finished but no connection
              // possible
              continue
            }

            const received = result as StreamResult

            if (received == undefined) {
              this.log(`received empty message. skipping`)
              continue
            }

            if (received.done) {
              break
            }

            this.log(`sinking ${received.value.slice().length} bytes into relayed connection`)

            sourcePromise = source!.next()

            yield Uint8Array.from([MigrationStatus.NOT_DONE, ...received.value.slice()])
          }

          resolve()
        }.call(this)
      ))
    )


    // Either stream is finished or WebRTC is available

    if (this._webRTCAvailable) {
      this._sinkMigrated = true
      if (this._sourceMigrated) {
        this.conn = this.channel
      }

      await toIterable.sink(this.channel)(
        async function* (this: WebRTCConnection): Stream['source'] {
          let result: SinkType

          while (true) {
            result = await sourcePromise

            if (result == undefined || result.done) {
              yield encodeWithLengthPrefix(Uint8Array.of(MigrationStatus.DONE))
              break
            }

            if (this.destroyed || this.channel.destroyed) {
              yield encodeWithLengthPrefix(Uint8Array.of(MigrationStatus.DONE))
              break
            }

            try {
              sourcePromise = source!.next()
            } catch (err) {
              this.error(err)
            }

            this.log(`sinking ${result.value.slice().length} bytes into webrtc[${(this.channel as any)._id}]`)

            yield encodeWithLengthPrefix(Uint8Array.from([MigrationStatus.NOT_DONE, ...result.value.slice()]))
          }
        }.call(this)
      )
    }
  }

  private async *createSource(this: WebRTCConnection): Stream['source'] {
    for await (const msg of this.relayConn.source) {
      const [migrationStatus, payload] = [msg.slice(0, 1), msg.slice(1)]

      if (migrationStatus[0] == MigrationStatus.DONE) {
        break
      } else if (migrationStatus[0] == MigrationStatus.NOT_DONE) {
        this.log(`getting ${payload.length} bytes from relayed connection`)
        yield payload
      } else {
        throw Error(`Invalid WebRTC migration status prefix. Got ${JSON.stringify(migrationStatus)}`)
      }
    }

    this._sourceMigrated = true

    if (!this._webRTCStateKnown) {
      await this._switchPromise.promise
    }

    if (this._webRTCAvailable) {
      console.log(`after`)

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
          break
        }
      }
    }
  }

  async close(_err?: Error): Promise<void> {
    if (this.destroyed) {
      return Promise.resolve()
    }

    this.timeline.close = Date.now()

    try {
      if (this._sinkMigrated || this._sourceMigrated) {
        ;(this.channel as SimplePeer).destroy()
      } else {
        await (this.conn as RelayConnection).close()
      }
    } catch (err) {
      this.error(err)
    }

    this.destroyed = true
  }
}

export { WebRTCConnection }
