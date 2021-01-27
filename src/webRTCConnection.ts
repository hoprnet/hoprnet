import { DialOptions, MultiaddrConnection, Stream, StreamResult } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import Multiaddr from 'multiaddr'
import type PeerId from 'peer-id'
import { durations, u8aToHex } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'
import Debug from 'debug'
import { RelayConnection } from './relayConnection'
import { randomBytes } from 'crypto'
import { toU8aStream } from './utils'
import abortable from 'abortable-iterator'
import LibP2P from 'libp2p'

const _log = Debug('hopr-connect')
const _error = Debug('hopr-connect:error')
// const _verbose = Debug('hopr-connect:verbose')

export const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(3)

const DONE = Uint8Array.from([1])
const NOT_DONE = Uint8Array.from([0])

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

    this.conn.once('restart', () => {
      this._destroyed = true
      try {
        this.channel.destroy()
      } catch {}
    })
    this._destroyed = false
    this._switchPromise = Defer<void>()
    this._webRTCStateKnown = false
    this._webRTCAvailable = false

    this._sourceMigrated = false
    this._sinkMigrated = false

    this._libp2p = opts.libp2p

    this._counterparty = opts.counterparty

    this.remoteAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)
    this.localAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)

    this._signal = options?.signal

    this.timeline = {
      open: Date.now()
    }

    this._id = u8aToHex(randomBytes(4), false)

    // used for testing
    this.__noWebRTCUpgrade = options?.__noWebRTCUpgrade

    this.channel.once('connect', this.onConnect.bind(this))

    this.channel.once('error', this.endWebRTCUpgrade.bind(this))

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

    this._webRTCTimeout = setTimeout(this.endWebRTCUpgrade.bind(this), WEBRTC_UPGRADE_TIMEOUT)

    this.sink = this._sink.bind(this)
  }

  private async *createSource(this: WebRTCConnection): Stream['source'] {
    while (true) {
      const result = await (this.conn as RelayConnection).source.next()

      if (result.done) {
        break
      }

      const [finished, payload] = [result.value.slice(0, 1), result.value.slice(1)]

      if (finished[0] == DONE[0]) {
        return
      } else {
        this.log(`getting ${result.value.slice().length} bytes from relayed connecton`)
        yield payload
      }
    }

    this.log(`webrtc source migrated but this._sinkMigrated`, this._sinkMigrated)

    this._sourceMigrated = true
    if (this._sinkMigrated) {
      this.conn = this.channel
    }

    if (!this._webRTCStateKnown) {
      await this._switchPromise.promise
    }

    if (this._webRTCAvailable) {
      this.log(`webRTC source handover done. Using direct connection to peer ${this.remoteAddr.getPeerId()}`)

      const iterator = this.channel[Symbol.asyncIterator]() as Stream['source']

      // @TODO `for await` does NOT work here -
      // even if it SHOULD be the SAME functionality
      // @dev `for await` produces a hanging promise
      while (true) {
        const result = await iterator.next()

        if (result.done) {
          break
        }

        const [finished, payload] = [result.value.slice(0, 1), result.value.slice(1)]

        if (finished[0] == DONE[0]) {
          this.log(`received DONE from WebRTC - ending stream`)
          break
        }

        this.log(`Getting NOT_DONE from WebRTC - ${result.value.slice().length} bytes`)

        yield payload
      }
    }
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
    this._webRTCAvailable = !this.__noWebRTCUpgrade

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

    let defer = Defer<void>()

    let streamSwitched = false

    let switchPromise = this._switchPromise.promise.then(() => {
      streamSwitched = true
    })

    ;(this.conn as RelayConnection).sink(
      async function* (this: WebRTCConnection): Stream['source'] {
        let result: SinkType

        while (!(this._webRTCAvailable || this._webRTCStateKnown)) {
          // 1. Handle stream handover
          // 2. Handle stream messages
          result = await Promise.race([
            // prettier-ignore
            switchPromise,
            sourcePromise
          ])

          if (streamSwitched) {
            streamSwitched = false
            break
          }

          const received = result as StreamResult

          if (received == undefined || received.done) {
            yield DONE
            break
          }

          this.log(`sinking ${received.value.slice().length} bytes into relayed connecton`)

          yield Uint8Array.from([...NOT_DONE, ...received.value.slice()])

          sourcePromise = source.next()
        }

        if (this._webRTCStateKnown && !this._webRTCAvailable) {
          this.log(
            `WebRTC connection upgrade failed. Continue using relayed connection with peer ${this._counterparty.toB58String()}.`
          )

          result = await sourcePromise

          if (result == undefined || result.done) {
            return
          }

          this.log(`sinking NOT_DONE into WebRTC - ${result.value.slice().length} bytes`)
          yield Uint8Array.from([...NOT_DONE, ...result.value.slice()])

          for await (const msg of source) {
            this.log(`sinking NOT_DONE into WebRTC - ${msg.slice().length} bytes`)
            yield Uint8Array.from([...NOT_DONE, ...msg.slice()])
          }

          this.log(`sinking DONE into WebRTC`)
          yield DONE
        }

        defer.resolve()
      }.call(this)
    )

    await defer.promise

    this.log(`webrtc sinkMigrated but this._sourceMigrated`, this._sourceMigrated)

    this._sinkMigrated = true
    if (this._sourceMigrated) {
      this.conn = this.channel
    }

    this.log(`sourcePromise`, sourcePromise)

    if (this._webRTCAvailable) {
      toIterable.sink(this.channel)(
        async function* (this: WebRTCConnection): Stream['source'] {
          let result: SinkType

          result = await sourcePromise

          if (result == undefined || result.done) {
            yield DONE
            return
          }

          if (this._destroyed || this.channel.destroyed) {
            yield DONE
            return
          }

          this.log(`yielding into webrtc ${this._id}`, result.value.slice().length)
          yield Uint8Array.from([...NOT_DONE, ...result.value.slice()])

          for await (const msg of source) {
            if (this._destroyed || this.channel.destroyed) {
              break
            }

            this.log(`yielding into webrtc ${this._id}`, msg.slice().length)
            yield Uint8Array.from([...NOT_DONE, ...msg.slice()])
          }

          yield DONE
        }.call(this)
      )
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
