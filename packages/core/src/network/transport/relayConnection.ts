import {Stream} from 'libp2p'
import Multiaddr from 'multiaddr'
import BL from 'bl'
import {MultiaddrConnection} from 'libp2p'
import Defer, {DeferredPromise} from 'p-defer'
import {RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, RELAY_WEBRTC_PREFIX, STOP} from './constants'
import {u8aEquals} from '@hoprnet/hopr-utils'

import type {Instance as SimplePeer} from 'simple-peer'

import type PeerId from 'peer-id'

import Debug from 'debug'

const error = Debug('hopr-core:transport:error')

class RelayConnection implements MultiaddrConnection {
  private _defer: DeferredPromise<void>
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean
  private webRTC: SimplePeer
  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  public source: AsyncIterable<Uint8Array>
  public sink: (source: AsyncIterable<Uint8Array>) => Promise<void>

  public conn: Stream

  public timeline: {
    open: number
    close?: number
  }

  constructor({
    stream,
    self,
    counterparty,
    webRTC
  }: {
    stream: Stream
    self: PeerId
    counterparty: PeerId
    webRTC?: SimplePeer
  }) {
    this.timeline = {
      open: Date.now()
    }

    this._defer = Defer()

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = stream

    this.localAddr = Multiaddr(`/p2p/${self.toB58String()}`)
    this.remoteAddr = Multiaddr(`/p2p/${counterparty.toB58String()}`)

    this.webRTC = webRTC

    this.source = async function* (this: RelayConnection) {
      const promise = this._defer.promise.then(() => ({done: true}))

      while (true) {
        let result: {
          done: boolean
          value: BL
        } = await Promise.race([
          // @ts-ignore
          this._stream.source.next(),
          promise
        ])

        if (result.value != null) {
          const received = result.value.slice()

          if (u8aEquals(received.slice(0, 1), RELAY_PAYLOAD_PREFIX)) {
            if (result.done) {
              this._destroyed = true
              return received.slice(1)
            } else {
              yield received.slice(1)
            }
          } else if (u8aEquals(received.slice(0, 1), RELAY_STATUS_PREFIX)) {
            if (u8aEquals(received.slice(1), STOP) || result.done) {
              this._destroyed = true
              return
            } else {
              error(`Received invalid status message ${received.slice(1)}. Dropping message.`)
            }
          } else if (u8aEquals(received.slice(0, 1), RELAY_WEBRTC_PREFIX)) {
            // console.log(`Receiving fancy WebRTC message`, JSON.parse(new TextDecoder().decode(received.slice(1))))
            this.webRTC?.signal(JSON.parse(new TextDecoder().decode(received.slice(1))))
          } else {
            error(`Received invalid prefix <${received.slice(0, 1)}. Dropping message.`)
          }
        } else if (result.done) {
          if (!this._destroyed) {
            if (!this._sinkTriggered) {
              this._stream.sink(
                (async function* () {
                  yield (new BL([
                    (RELAY_STATUS_PREFIX as unknown) as BL,
                    (STOP as unknown) as BL
                  ]) as unknown) as Uint8Array
                })()
              )
            }
            this._destroyed = true
          }
          return
        }
      }
    }.call(this)

    this.sink = (source: AsyncIterable<Uint8Array>): Promise<void> => {
      this._sinkTriggered = true

      return this._stream.sink(
        async function* (this: RelayConnection) {
          let promiseDone = false

          let streamResolved = false
          let streamDone = false
          let streamMsg: BL

          let webRTCresolved = false
          let webRTCdone = this.webRTC == null
          let webRTCmsg: Uint8Array | void

          function streamSourceFunction({done, value}: {done: boolean; value?: BL}) {
            streamResolved = true
            streamMsg = value

            if (done) {
              streamDone = true
            }
          }

          // @ts-ignore
          let streamPromise = source.next().then(streamSourceFunction)

          function webRTCSourceFunction({done, value}: {done?: boolean; value: Uint8Array | void}) {
            webRTCresolved = true
            webRTCmsg = value

            if (done) {
              webRTCdone = true
            }
          }

          let webRTCstream: AsyncGenerator<Uint8Array, void, unknown>
          if (this.webRTC != null) {
            webRTCstream = async function* (this: RelayConnection) {
              let defer = Defer<DeferredPromise<any>>()
              let waiting = false
              const webRTCmessages: Uint8Array[] = []
              let done = false
              function onSignal(msg: any) {
                webRTCmessages.push(new TextEncoder().encode(JSON.stringify(msg)))
                if (waiting) {
                  waiting = false
                  defer.resolve(Defer<DeferredPromise<any>>())
                }
              }
              this.webRTC.on('signal', onSignal)

              this.webRTC.once('connect', () => {
                done = true
                this.webRTC.removeListener('signal', onSignal)
                defer.resolve()
              })

              while (!done) {
                while (webRTCmessages.length > 0) {
                  yield webRTCmessages.shift()
                }

                if (done) {
                  break
                }

                waiting = true

                defer = await defer.promise

                if (done) {
                  break
                }
              }
            }.call(this)
          }

          let webRTCPromise: Promise<void>

          const promise = this._defer.promise.then(() => {
            promiseDone = true
          })

          while (true) {
            if (!webRTCdone && this.webRTC != null) {
              if (webRTCPromise == null) {
                webRTCPromise = webRTCstream.next().then(webRTCSourceFunction)
              }
              await Promise.race([
                // @ts-ignore
                streamPromise,
                webRTCPromise,
                promise
              ])
            } else {
              await Promise.race([
                // @ts-ignore
                streamPromise,
                promise
              ])
            }

            if (streamResolved && streamMsg != null) {
              streamResolved = false
              let _received = streamMsg.slice()

              if (promiseDone || (streamDone && webRTCdone)) {
                if (_received != null) {
                  yield new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (_received as unknown) as BL])
                }

                this._destroyed = true

                return (new BL([
                  (RELAY_STATUS_PREFIX as unknown) as BL,
                  (STOP as unknown) as BL
                ]) as unknown) as Uint8Array
              } else {
                if (_received == null) {
                  // @TODO change this to `return` to end the stream
                  // once we receive an empty message
                  continue
                }

                yield new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (_received as unknown) as BL])

                // @ts-ignore
                streamPromise = source.next().then(streamSourceFunction)
              }
            } else if (webRTCresolved && webRTCmsg != null) {
              webRTCresolved = false
              // @ts-ignore
              yield new BL([RELAY_WEBRTC_PREFIX, webRTCmsg])

              webRTCPromise = webRTCstream.next().then(webRTCSourceFunction)
            } else if (promiseDone || (streamDone && webRTCdone)) {
              if (!this._destroyed) {
                this._destroyed = true

                return (new BL([
                  (RELAY_STATUS_PREFIX as unknown) as BL,
                  (STOP as unknown) as BL
                ]) as unknown) as Uint8Array
              }

              return
            }
          }
        }.call(this)
      )
    }
  }

  get destroyed(): boolean {
    return this._destroyed
  }

  close(): Promise<void> {
    this._defer.resolve()
    this.timeline.close = Date.now()
    return Promise.resolve()
  }
}

export {RelayConnection}
