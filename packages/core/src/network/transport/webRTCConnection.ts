import { MultiaddrConnection, Stream } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import Multiaddr from 'multiaddr'
import type PeerId from 'peer-id'
import { durations } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'

const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(7)

class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferredPromise<void>
  private _webRTCStateKnown: boolean
  private _webRTCAvailable: boolean
  private _migrated: boolean
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

    this.channel.on('connect', () => {
      clearTimeout(this._webRTCTimeout)

      console.log(`available after connect`)

      this._webRTCStateKnown = true
      this._webRTCAvailable = true
      this._switchPromise.resolve()
    })

    const endWebRTCUpgrade = () => {
      clearTimeout(this._webRTCTimeout)
      console.log(`error thrown`)
      this._webRTCStateKnown = true
      this._webRTCAvailable = false
      this._switchPromise.resolve()
      setImmediate(() => {
        this.channel.destroy()
      })
    }

    this.channel.on('iceTimeout', endWebRTCUpgrade)
    this.channel.on('error', endWebRTCUpgrade)

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

      this.conn.sink(
        async function* (this: WebRTCConnection): Stream['source'] {
          if (this._webRTCTimeout == null) {
            this._webRTCTimeout = setTimeout(endWebRTCUpgrade, WEBRTC_UPGRADE_TIMEOUT)
          }

          while (!this._webRTCAvailable) {
            if (!this._webRTCStateKnown) {
              await Promise.race([
                // prettier-ignore
                sourcePromise
                // this._switchPromise.promise
              ])

              if (sourceReceived) {
                sourceReceived = false

                if (sourceDone) {
                  break
                }

                // console.log(`sinking into relay connection`, new TextDecoder().decode(sourceMsg))

                yield sourceMsg

                console.log(
                  `this._webRTCAvailable`,
                  this._webRTCAvailable,
                  `this._webRTCStateKnown`,
                  this._webRTCStateKnown
                )

                if (!this._webRTCAvailable) {
                  sourcePromise = source.next().then(sourceFunction)
                } else {
                  defer.resolve()
                }
              }
            } else {
              await sourcePromise

              if (sourceDone) {
                break
              }

              yield sourceMsg
              yield* source
            }
          }
          console.log(`sink returned`)
        }.call(this)
      )

      if (!this._webRTCStateKnown || this._webRTCAvailable) {
        await this._switchPromise.promise

        clearTimeout(this._webRTCTimeout)
        if (this._webRTCAvailable) {
          const sink = toIterable.sink(this.channel)
          this._migrated = true

          sink(
            (async function* () {
              await defer.promise
              console.log(`start sinking into WebRTC`)

              for await (const msg of source) {
                yield Buffer.from(msg.slice())
              }
            })()
          )
        }
      }
    }

    this.source = async function* (this: WebRTCConnection): Stream['source'] {
      if (this._webRTCTimeout == null) {
        this._webRTCTimeout = setTimeout(endWebRTCUpgrade, WEBRTC_UPGRADE_TIMEOUT)
      }

      yield* this.conn.source

      if (this._webRTCAvailable || !this._webRTCStateKnown) {
        clearTimeout(this._webRTCTimeout)
        await this._switchPromise.promise

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

    if (this.timeline == null) {
      this.timeline = {
        open: Date.now(),
        closed: Date.now()
      }
    } else {
      this.timeline.closed = Date.now()
    }

    if (this._migrated) {
      return Promise.resolve(this.channel.destroy())
    } else {
      this.channel.destroy()
      return this.conn.close()
    }
  }
}

export { WebRTCConnection }
