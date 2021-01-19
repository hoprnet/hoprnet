import { MultiaddrConnection, Stream } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import Multiaddr from 'multiaddr'
import type PeerId from 'peer-id'
import { durations, u8aToHex } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'
import Debug from 'debug'
import { RelayConnection } from './relayConnection'
import { randomBytes } from 'crypto'

const _log = Debug('hopr-connect')
const _error = Debug('hopr-connect:error')
const _verbose = Debug('hopr-connect:verbose')

export const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(2)

class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferredPromise<void>
  private _webRTCStateKnown: boolean
  private _webRTCAvailable: boolean
  private _destroyed: boolean
  private _webRTCTimeout?: NodeJS.Timeout

  private _counterparty: PeerId

  public source: Stream['source']

  public remoteAddr: Multiaddr
  public localAddr: Multiaddr

  private channel: SimplePeer

  public conn: RelayConnection

  private _id: string

  public timeline: {
    open: number
    closed?: number
  }

  constructor(opts: { conn: RelayConnection; channel: SimplePeer; self: PeerId; counterparty: PeerId }) {
    this.channel = opts.channel
    this.conn = opts.conn
    this._destroyed = false
    this._switchPromise = Defer<void>()
    this._webRTCStateKnown = false
    this._webRTCAvailable = false

    this._counterparty = opts.counterparty

    this.remoteAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)
    this.localAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)

    this.timeline = {
      open: Date.now()
    }

    this._id = u8aToHex(randomBytes(4), false)

    this.channel.once('connect', () => {
      if (this._webRTCTimeout != undefined) {
        clearTimeout(this._webRTCTimeout)
      }

      console.log(`CONNNNNECTED`, this._webRTCTimeout)

      this._webRTCStateKnown = true
      this._webRTCAvailable = true
      this._switchPromise.resolve()
    })

    this.channel.once('error', this.endWebRTCUpgrade.bind(this))

    this.source = async function* (this: WebRTCConnection): Stream['source'] {
      yield* this.conn.source

      if (!this._webRTCStateKnown || this._webRTCAvailable) {
        await this._switchPromise.promise
      }

      if (this._webRTCAvailable || !this._webRTCStateKnown) {
        this.log(`webRTC handover done. Using direct connection to peer ${this.remoteAddr.getPeerId()}`)

        yield* this.channel[Symbol.asyncIterator]() as Stream['source']
      }
    }.call(this)

    this._webRTCTimeout = setTimeout(this.endWebRTCUpgrade.bind(this), WEBRTC_UPGRADE_TIMEOUT)
  }

  private log(..._: any[]) {
    _log(`RX [${this._id}]`, ...arguments)
  }

  private verbose(..._: any[]) {
    _verbose(`RX [${this._id}]`, ...arguments)
  }

  private error(..._: any[]) {
    _error(`RX [${this._id}]`, ...arguments)
  }

  get destroyed(): boolean {
    return this._destroyed
  }

  private endWebRTCUpgrade(err?: any) {
    console.log(`END WEBRTC UPGRADE called`)
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

  public async sink(source: Stream['source']): Promise<void> {
    type SinkType = IteratorResult<Uint8Array, void> | void
    let sourcePromise = source.next()

    let defer = Defer<void>()

    let streamSwitched = false

    let switchPromise = this._switchPromise.promise.then(() => {
      streamSwitched = true
    })

    this.conn.sink(
      async function* (this: WebRTCConnection): Stream['source'] {
        let result: SinkType

        while (!(this._webRTCAvailable || this._webRTCStateKnown)) {
          result = await Promise.race([
            // prettier-ignore
            switchPromise,
            sourcePromise
          ])

          if (streamSwitched) {
            streamSwitched = false
            break
          }

          const received = result as IteratorResult<Uint8Array, void>

          if (received == undefined || received.done) {
            break
          }

          yield received.value.slice()

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

          yield result.value.slice()

          yield* source
        }

        defer.resolve()
        console.log(`defer resolved`)
      }.call(this)
    )

    await defer.promise

    if (this._webRTCAvailable) {
      toIterable.sink(this.channel)(
        async function* (this: WebRTCConnection): Stream['source'] {
          let result: SinkType

          result = await sourcePromise

          if (result == undefined || result.done) {
            return
          }

          yield result.value.slice()

          yield* source
        }.call(this)
      )
    }
  }

  async close(_err?: Error): Promise<void> {
    if (this.destroyed) {
      return Promise.resolve()
    }

    this.timeline.closed = Date.now()

    try {
      this.channel.destroy()
    } catch (err) {
      err(`WebRTC error while destroying connection to peer ${this.remoteAddr.getPeerId()}. Error: ${err}`)
    }

    try {
      this.conn.close()
    } catch (err) {
      this.error(`Error while trying to close relayed connection. Increase log level to display error.`)
      this.verbose(err)
    }

    this._destroyed = true
  }
}

export { WebRTCConnection }
