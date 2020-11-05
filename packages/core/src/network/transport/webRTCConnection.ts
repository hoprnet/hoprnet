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

    this.remoteAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)
    this.localAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)

    this.channel.on('connect', () => {
      if (this._webRTCTimeout != null) {
        clearTimeout(this._webRTCTimeout)
      }

      console.log(`available after connect`)
      this.timeline = {
        open: Date.now()
      }
      this._webRTCStateKnown = true
      this._webRTCAvailable = true
      this._switchPromise.resolve()
    })

    const endWebRTCUpgrade = () => {
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

    this.sink = async (source: Stream['source']) => {
      this.conn.sink(
        async function* (this: WebRTCConnection) {
          if (this._webRTCTimeout == null) {
            this._webRTCTimeout = setTimeout(endWebRTCUpgrade, WEBRTC_UPGRADE_TIMEOUT)
          }

          let sourceReceived = false
          let sourceMsg: Uint8Array | undefined
          let sourceDone = false

          function sourceFunction(arg: { value?: Uint8Array; done?: boolean }) {
            sourceReceived = true
            sourceMsg = arg.value

            if (arg.done) {
              sourceDone = true
            }
          }

          let sourcePromise = source.next().then(sourceFunction)

          while (!this._webRTCAvailable) {
            if (!this._webRTCStateKnown) {
              await Promise.race([
                // prettier-ignore
                sourcePromise,
                this._switchPromise.promise
              ])

              if (sourceReceived) {
                sourceReceived = false

                if (sourceDone && this._webRTCStateKnown && !this._webRTCAvailable) {
                  return sourceMsg
                } else if (sourceDone) {
                  yield sourceMsg
                  break
                } else {
                  sourcePromise = source.next().then(sourceFunction)
                  yield sourceMsg
                }
              }
            } else {
              await sourcePromise
              if (sourceDone) {
                return sourceMsg
              } else {
                yield sourceMsg
                yield* source
              }
            }
          }
        }.call(this)
      )

      this._switchPromise.promise.then(() => {
        if (this._webRTCAvailable) {
          const sink = toIterable.sink(this.channel)
          this._migrated = true

          sink(source)
          setImmediate(() => {
            this.conn.close().then(() => {
              console.log(`sink migrated`)
              this._migrated = true
            })
          })
        }
      })
    }

    this.source = async function* (this: WebRTCConnection) {
      if (this._webRTCTimeout == null) {
        this._webRTCTimeout = setTimeout(endWebRTCUpgrade, WEBRTC_UPGRADE_TIMEOUT)
      }

      let streamMsgReceived = false
      let streamMsg: Uint8Array | undefined
      let streamDone = false

      function streamSourceFunction(arg: { value?: Uint8Array; done?: boolean }) {
        streamMsgReceived = true
        streamMsg = arg.value

        if (arg.done) {
          streamDone = true
        }
      }

      let streamPromise = this.conn.source.next().then(streamSourceFunction)

      while (!this._webRTCAvailable) {
        if (!this._webRTCStateKnown) {
          await Promise.race([
            // prettier-ignore
            streamPromise,
            this._switchPromise.promise
          ])

          if (streamMsgReceived) {
            streamMsgReceived = false
            if (streamDone && this._webRTCStateKnown && !this._webRTCAvailable) {
              return streamMsg
            } else if (streamDone) {
              yield streamMsg
              break
            } else {
              streamPromise = this.conn.source.next().then(streamSourceFunction)
              yield streamMsg
            }
          }
        } else {
          await streamPromise

          if (streamDone) {
            return streamMsg
          } else {
            yield streamMsg
            yield* this.conn.source
          }
        }
      }

      await this._switchPromise.promise

      if (this._webRTCAvailable) {
        setImmediate(() => {
          this.conn.close().then(() => {
            this._migrated = true
            console.log(`source migrated`)
          })
        })
        yield* this.channel[Symbol.asyncIterator]()
      } else {
        return
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
