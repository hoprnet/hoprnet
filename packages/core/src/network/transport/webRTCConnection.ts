import { MultiaddrConnection, Stream } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import Multiaddr from 'multiaddr'
import type PeerId from 'peer-id'
import { durations } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'
import { blue, red } from 'chalk'
import Debug from 'debug'

const log = Debug('hopr-core:transport')

const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(1)

class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferredPromise<void>
  private _webRTCStateKnown: boolean
  private _webRTCAvailable: boolean
  private _destroyed: boolean
  private _webRTCTimeout?: NodeJS.Timeout

  public source: Stream['source']

  public remoteAddr: Multiaddr
  public localAddr: Multiaddr

  public sink: Stream['sink']

  private channel: SimplePeer

  public conn: MultiaddrConnection

  public timeline: {
    open: number
    closed?: number
  }

  constructor(opts: { conn: MultiaddrConnection; channel: SimplePeer; self: PeerId; counterparty: PeerId }) {
    this.channel = opts.channel
    this.conn = opts.conn
    this._destroyed = false
    this._switchPromise = Defer<void>()
    this._webRTCStateKnown = false
    this._webRTCAvailable = false

    this.remoteAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)
    this.localAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)

    this.timeline = {
      open: Date.now()
    }

    this.channel.once('connect', () => {
      clearTimeout(this._webRTCTimeout)

      this._webRTCStateKnown = true
      this._webRTCAvailable = true
      this._switchPromise.resolve()
    })

    const endWebRTCUpgrade = (err?: any) => {
      clearTimeout(this._webRTCTimeout)
      log(`ending WebRTC upgrade due error: ${err}`)
      this._webRTCStateKnown = true
      this._webRTCAvailable = false
      this._switchPromise.resolve()
      setImmediate(() => {
        this.channel.destroy()
      })
    }

    this.channel.once('iceTimeout', endWebRTCUpgrade)
    this.channel.once('error', endWebRTCUpgrade)

    this.sink = async (source: Stream['source']): Promise<void> => {
      let sourceReceived = false
      let sourceMsg: Uint8Array
      let sourceDone = false

      function sourceFunction(arg: IteratorResult<Uint8Array, void>) {
        sourceReceived = true
        sourceDone = arg.done

        if (!arg.done) {
          sourceMsg = arg.value as Uint8Array
        }
      }

      let sourcePromise = source.next().then(sourceFunction)

      let defer = Defer<void>()

      let promiseTriggered = true

      let streamSwitched = false
      let switchPromise = this._switchPromise.promise.then(() => {
        streamSwitched = false
      })

      this.conn.sink(
        async function* (this: WebRTCConnection): Stream['source'] {
          if (this._webRTCTimeout == null) {
            this._webRTCTimeout = setTimeout(endWebRTCUpgrade, WEBRTC_UPGRADE_TIMEOUT)
          }

          while (!this._webRTCAvailable) {
            if (!this._webRTCStateKnown) {
              await Promise.race([
                // prettier-ignore
                sourcePromise,
                switchPromise
              ])

              if (streamSwitched) {
                break
              } else if (sourceReceived) {
                promiseTriggered = false
                sourceReceived = false

                if (sourceDone) {
                  break
                }

                yield sourceMsg.slice()

                if (!this._webRTCAvailable) {
                  sourcePromise = source.next().then(sourceFunction)
                  promiseTriggered = true
                }
              }
            } else {
              log(`fallback to relayed connection`)
              await sourcePromise

              if (sourceDone) {
                break
              }

              yield sourceMsg
              yield* source
            }
          }
          defer.resolve()
        }.call(this)
      )

      await defer.promise

      if (this._webRTCAvailable) {
        clearTimeout(this._webRTCTimeout)
        if (this._webRTCAvailable) {
          const sink = toIterable.sink(this.channel)

          sink(
            async function* () {
              if (promiseTriggered && !sourceReceived) {
                await sourcePromise

                yield sourceMsg.slice()
              } else if (promiseTriggered && sourceReceived) {
                yield sourceMsg.slice()
              }

              log(`switching to direct WebRTC connection with peer ${blue(this.remoteAddr.getPeerId())}`)

              for await (const msg of source) {
                yield msg.slice()
              }
            }.call(this)
          )
        }
      }
    }

    this.source = async function* (this: WebRTCConnection): Stream['source'] {
      if (this._webRTCTimeout == null) {
        this._webRTCTimeout = setTimeout(endWebRTCUpgrade, WEBRTC_UPGRADE_TIMEOUT)
      }

      yield* this.conn.source

      if (!this._webRTCStateKnown || this._webRTCAvailable) {
        await this._switchPromise.promise
      }

      if (this._webRTCAvailable || !this._webRTCStateKnown) {
        clearTimeout(this._webRTCTimeout)
        log(`webRTC handover done. Using direct connection to peer ${blue(this.remoteAddr.getPeerId())}`)
        yield* this.channel[Symbol.asyncIterator]() as Stream['source']
      }
    }.call(this)
  }

  get destroyed(): boolean {
    return this._destroyed
  }

  async close(_err?: Error): Promise<void> {
    if (this.destroyed) {
      return Promise.resolve()
    }

    this.timeline.closed = Date.now()

    try {
      this.channel.destroy()
    } catch (err) {
      err(`WebRTC error while destroying connection to peer ${blue(this.remoteAddr.getPeerId())}. Error: ${red(err)}`)
    }

    this.conn.close()

    this._destroyed = true
  }
}

export { WebRTCConnection }
