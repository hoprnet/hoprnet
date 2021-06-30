/// <reference path="../@types/stream-to-it.ts" />

import { DialOptions, MultiaddrConnection, Stream, StreamResult } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import type PeerId from 'peer-id'
import { durations, u8aToHex } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'
import Debug from 'debug'
import type { RelayConnection } from '../relay/connection'
import { randomBytes } from 'crypto'
import { toU8aStream, encodeWithLengthPrefix, decodeWithLengthPrefix } from '../utils'
import abortable from 'abortable-iterator'
import LibP2P from 'libp2p'

const _log = Debug('hopr-connect')
const _error = Debug('hopr-connect:error')
// const _verbose = Debug('hopr-connect:verbose')

export const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(3)

enum MigrationStatus {
  NOT_DONE,
  DONE
}

class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferredPromise<void>
  private _webRTCStateKnown: boolean
  private _webRTCAvailable: boolean
  private _destroyed: boolean
  private _webRTCTimeout?: NodeJS.Timeout

  private _sourceMigrated: boolean
  private _sinkMigrated: boolean

  private _counterparty: PeerId

  public remoteAddr: MultiaddrConnection['remoteAddr']
  public localAddr: MultiaddrConnection['remoteAddr']

  private channel: SimplePeer

  public sink: Stream['sink']
  public source: Stream['source']

  public conn: RelayConnection | SimplePeer

  private _id: string
  private _signal?: AbortSignal

  private _libp2p: LibP2P

  public timeline: MultiaddrConnection['timeline']

  // used for testing
  private __noWebRTCUpgrade?: boolean

  constructor(
    opts: { conn: RelayConnection; channel: SimplePeer; self: PeerId; counterparty: PeerId; libp2p: LibP2P },
    options?: DialOptions & { __noWebRTCUpgrade?: boolean }
  ) {
    this.channel = opts.channel
    this.conn = opts.conn

    this._destroyed = false
    this._switchPromise = Defer<void>()
    this._webRTCStateKnown = false
    this._webRTCAvailable = false

    this._sourceMigrated = false
    this._sinkMigrated = false

    this._libp2p = opts.libp2p

    this._counterparty = opts.counterparty

    this.remoteAddr = opts.conn.remoteAddr
    this.localAddr = opts.conn.localAddr

    this._signal = options?.signal

    this.timeline = {
      open: Date.now()
    }

    this._id = u8aToHex(randomBytes(4), false)

    // used for testing
    this.__noWebRTCUpgrade = options?.__noWebRTCUpgrade

    const errListener = this.endWebRTCUpgrade.bind(this)

    this.channel.once('error', errListener)
    this.channel.once('connect', () => {
      this.channel.removeListener('error', errListener)
      this.onConnect.call(this)
    })

    this.channel.on('iceStateChange', (iceConnectionState: string, iceGatheringState: string) => {
      if (iceConnectionState === 'disconnected' && iceGatheringState === 'complete') {
        this.timeline.close = Date.now()
        this._destroyed = true
        // HACK, @TODO remove this
        this._libp2p.connectionManager.connections.delete(this._counterparty.toB58String())
      }
    })

    this.source =
      this._signal != undefined
        ? (abortable(this.createSource(), this._signal) as Stream['source'])
        : this.createSource()

    this.sink = this._sink.bind(this)

    this._webRTCTimeout = setTimeout(this.endWebRTCUpgrade.bind(this), WEBRTC_UPGRADE_TIMEOUT)
  }

  private log(..._: any[]) {
    _log(`WRTC [${this._id}]`, ...arguments)
  }

  // private verbose(..._: any[]) {
  //   _verbose(`RX [${this._id}]`, ...arguments)
  // }

  private error(..._: any[]) {
    _error(`WRTC [${this._id}]`, ...arguments)
  }

  get destroyed(): boolean {
    return this._destroyed
  }

  private endWebRTCUpgrade(err?: any) {
    if (this._webRTCTimeout != undefined) {
      clearTimeout(this._webRTCTimeout)
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
    if (this._webRTCTimeout != undefined) {
      clearTimeout(this._webRTCTimeout)
    }

    this._webRTCStateKnown = true

    if (this.__noWebRTCUpgrade) {
      this._webRTCAvailable = false
    } else {
      this._webRTCAvailable = true
    }

    // @TODO could be mixed up
    this._switchPromise.resolve()
  }

  private async _sink(_source: Stream['source']): Promise<void> {
    type SinkType = StreamResult | void
    let source =
      this._signal != undefined
        ? (abortable(toU8aStream(_source), this._signal) as Stream['source'])
        : toU8aStream(_source)

    let sourcePromise = source.next()

    const baseConn = this.conn as RelayConnection

    await new Promise<void>((resolve) =>
      baseConn.sink(
        async function* (this: WebRTCConnection): Stream['source'] {
          let result: SinkType

          while (true) {
            const promises: Promise<SinkType>[] = []

            if (!this._webRTCStateKnown) {
              promises.push(this._switchPromise.promise)
            }

            promises.push(sourcePromise)

            // 1. Handle stream handover
            // 2. Handle stream messages
            result = await Promise.race(promises)

            if (this._webRTCAvailable) {
              yield Uint8Array.of(MigrationStatus.DONE)

              break
            }

            const received = result as StreamResult

            if (received == undefined) {
              this.log(`received empty message. skipping`)
              continue
            }

            if (received.done) {
              break
            }

            this.log(`sinking ${received.value.slice().length} bytes into relayed connecton`)

            sourcePromise = source.next()

            yield Uint8Array.from([MigrationStatus.NOT_DONE, ...received.value.slice()])
          }

          resolve()
        }.call(this)
      )
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

            if (this._destroyed || this.channel.destroyed) {
              yield encodeWithLengthPrefix(Uint8Array.of(MigrationStatus.DONE))
              break
            }

            try {
              sourcePromise = source.next()
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
    const baseConn = this.conn as RelayConnection

    for await (const msg of baseConn.source) {
      const [migrationStatus, payload] = [msg.slice(0, 1), msg.slice(1)]

      if (migrationStatus[0] == MigrationStatus.DONE) {
        break
      } else if (migrationStatus[0] == MigrationStatus.NOT_DONE) {
        this.log(`getting ${msg.slice().length} bytes from relayed connecton`)
        yield payload
      } else {
        throw Error(`Invalid WebRTC migration status prefix. Got ${JSON.stringify(migrationStatus)}`)
      }
    }

    this.log(`webrtc source migrated but this._sinkMigrated`, this._sinkMigrated)

    this._sourceMigrated = true

    if (!this._webRTCStateKnown) {
      await this._switchPromise.promise
    }

    if (this._webRTCAvailable) {
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

          console.log(`Getting from webRTC`, msg.slice())

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

    this._destroyed = true
  }
}

export { WebRTCConnection }
