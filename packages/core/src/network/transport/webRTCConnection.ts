import { MultiaddrConnection, Stream } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import Multiaddr from 'multiaddr'
import type PeerId from 'peer-id'
import { durations } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'

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

      let graceFullyMigrated = false

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

                console.log(`sinking into relay connection`, new TextDecoder().decode(sourceMsg.slice()))

                console.log(
                  `this._webRTCAvailable`,
                  this._webRTCAvailable,
                  `this._webRTCStateKnown`,
                  this._webRTCStateKnown
                )

                if (!this._webRTCAvailable) {
                  yield sourceMsg

                  sourcePromise = source.next().then(sourceFunction)
                  graceFullyMigrated = true
                }
              }
            } else {
              console.log(`fallback branch`)
              await sourcePromise

              if (sourceDone) {
                break
              }

              yield sourceMsg
              yield* source
            }
          }
          defer.resolve()
          console.log(`sink returned`)
        }.call(this)
      )

      if (!this._webRTCStateKnown || this._webRTCAvailable) {
        await this._switchPromise.promise

        clearTimeout(this._webRTCTimeout)
        if (this._webRTCAvailable) {
          const sink = toIterable.sink(this.channel)

          sink(
            (async function* () {
              console.log(`before defer.promise`, graceFullyMigrated)
              // await defer.promise

              if (!graceFullyMigrated) {
                console.log(`!graceFullyMigrated`)
                // await sourcePromise

                if (sourceMsg != null) {
                  yield sourceMsg
                }
              }
              console.log(`start sinking into WebRTC`)

              for await (const msg of source) {
                console.log(`sinking into webrtc`, msg)
                yield msg.slice()
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

      console.log(`before await switchPromise`)

      if (this._webRTCAvailable || !this._webRTCStateKnown) {
        clearTimeout(this._webRTCTimeout)

        await this._switchPromise.promise

        console.log(`source_ migrated`)

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
      err(`WebRTC error while destroying: ${err}`)
    }

    this.conn.close()

    this._destroyed = true
  }
}

export { WebRTCConnection }
