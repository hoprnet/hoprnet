import { MultiaddrConnection, Stream } from 'libp2p'
import Defer, { DeferredPromise } from 'p-defer'

import type { Instance as SimplePeer } from 'simple-peer'
import Multiaddr from 'multiaddr'
import type PeerId from 'peer-id'
import { durations } from '@hoprnet/hopr-utils'
import toIterable from 'stream-to-it'
import Debug from 'debug'
import { RelayConnection } from './relayConnection'

const log = Debug('hopr-connect')
const verbose = Debug('hopr-connect:verbose')

export const WEBRTC_UPGRADE_TIMEOUT = durations.seconds(1)

class WebRTCConnection implements MultiaddrConnection {
  private _switchPromise: DeferredPromise<void>
  private _webRTCStateKnown: boolean
  private _webRTCAvailable: boolean
  private _destroyed: boolean
  private _webRTCTimeout?: NodeJS.Timeout

  private _iteration: number

  public source: Stream['source']

  public remoteAddr: Multiaddr
  public localAddr: Multiaddr

  public sink: Stream['sink']

  private channel: SimplePeer

  public conn: RelayConnection

  public timeline: {
    open: number
    closed?: number
  }

  constructor(opts: {
    conn: RelayConnection
    channel: SimplePeer
    self: PeerId
    counterparty: PeerId
    iteration: number
  }) {
    this.channel = opts.channel
    this.conn = opts.conn
    this._destroyed = false
    this._switchPromise = Defer<void>()
    this._webRTCStateKnown = false
    this._webRTCAvailable = false
    this._iteration = opts.iteration

    this.remoteAddr = Multiaddr(`/p2p/${opts.counterparty.toB58String()}`)
    this.localAddr = Multiaddr(`/p2p/${opts.self.toB58String()}`)

    this.timeline = {
      open: Date.now()
    }

    this.channel.once('connect', () => {
      if (this._webRTCTimeout != undefined) {
        clearTimeout(this._webRTCTimeout)
      }

      this._webRTCStateKnown = true
      this._webRTCAvailable = true
      this._switchPromise.resolve()
    })

    const endWebRTCUpgrade = (err?: any) => {
      if (this._webRTCTimeout != undefined) {
        clearTimeout(this._webRTCTimeout)
      }

      log(`ending WebRTC upgrade due error: ${err}`)
      this._webRTCStateKnown = true
      this._webRTCAvailable = false
      this._switchPromise.resolve()
      setImmediate(() => {
        this.channel.destroy()
      })
    }

    this.channel.once('error', endWebRTCUpgrade)

    this.sink = async (source: Stream['source']): Promise<void> => {
      let sourceResult: IteratorResult<Uint8Array, void> | undefined
      let sourceReceived = false

      function sourceFunction(arg: IteratorResult<Uint8Array, void>) {
        sourceReceived = true
        sourceResult = arg
      }

      let sourcePromise = source.next().then(sourceFunction)

      let defer = Defer<void>()

      let promiseTriggered = true

      let streamSwitched = false
      let switchPromise = this._switchPromise.promise.then(() => {
        streamSwitched = true
      })

      this.conn.sink(
        async function* (this: WebRTCConnection): Stream['source'] {
          if (this._webRTCTimeout == null) {
            this._webRTCTimeout = setTimeout(endWebRTCUpgrade, WEBRTC_UPGRADE_TIMEOUT)
          }

          while (!this._webRTCAvailable && !this._webRTCStateKnown) {
            await Promise.race([
              // prettier-ignore
              sourcePromise,
              switchPromise
            ])

            if (streamSwitched) {
              streamSwitched = false
              break
            }

            promiseTriggered = false
            sourceReceived = false

            if (sourceResult == undefined || sourceResult.done) {
              break
            }

            yield sourceResult.value.slice()

            if (!this._webRTCAvailable) {
              sourcePromise = source.next().then(sourceFunction)
              promiseTriggered = true
            } else {
              yield new Uint8Array()
            }
          }

          if (this._webRTCStateKnown && !this._webRTCAvailable) {
            log(
              `WebRTC connection upgrade failed. Continue using relayed connection with peer ${opts.counterparty.toB58String()}.`
            )

            await sourcePromise

            if (sourceResult == undefined || sourceResult.done) {
              return
            }

            yield sourceResult.value.slice()

            yield* source
          }

          defer.resolve()
        }.call(this)
      )

      await defer.promise

      if (this._webRTCAvailable) {
        if (this._webRTCTimeout != undefined) {
          clearTimeout(this._webRTCTimeout)
        }

        if (this._webRTCAvailable) {
          toIterable.sink(this.channel)(
            async function* (this: WebRTCConnection): Stream['source'] {
              console.log(`after defer`, promiseTriggered, sourceReceived)

              if (promiseTriggered && !sourceReceived) {
                if (!sourceReceived) {
                  await sourcePromise
                }

                console.log(`after defer`, promiseTriggered, sourceReceived, sourceResult)

                if (sourceResult != undefined && !sourceResult.done) {
                  yield sourceResult.value.slice()
                }
              }

              log(`switching to direct WebRTC connection with peer ${this.remoteAddr.getPeerId()}`)

              while (
                // @ts-ignore
                this.channel.connected &&
                this._iteration == this.conn._iteration
              ) {
                await new Promise((resolve) => setTimeout(resolve, 100))
                console.log(`sinking into WebRTC`)
                const result = await source.next()

                if (result.done) {
                  break
                }

                if (
                  // @ts-ignore
                  this.channel.connected &&
                  this._iteration == this.conn._iteration
                ) {
                  console.log(`sinking ${new TextDecoder().decode(result.value.slice())} into WebRTC`)
                  yield result.value.slice()
                } else {
                  break
                }
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

      // @TODO
      // end is not coming
      yield* this.conn.source

      console.log(`stream has ended`)
      if (!this._webRTCStateKnown || this._webRTCAvailable) {
        await this._switchPromise.promise
      }
      console.log(`after waiting for switch`)

      if (this._webRTCAvailable || !this._webRTCStateKnown) {
        clearTimeout(this._webRTCTimeout)
        log(`webRTC handover done. Using direct connection to peer ${this.remoteAddr.getPeerId()}`)

        for await (const msg of this.channel[Symbol.asyncIterator]() as Stream['source']) {
          console.log(`getting from WebRTC ${new TextDecoder().decode(msg)}`)
          yield msg
        }
        // yield* this.channel[Symbol.asyncIterator]() as Stream['source']
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
      err(`WebRTC error while destroying connection to peer ${this.remoteAddr.getPeerId()}. Error: ${err}`)
    }

    try {
      this.conn.close()
    } catch (err) {
      err(`Error while trying to close relayed connection. Increase log level to display error.`)
      verbose(err)
    }

    this._destroyed = true
  }
}

export { WebRTCConnection }
