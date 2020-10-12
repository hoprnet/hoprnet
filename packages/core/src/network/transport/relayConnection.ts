import Multiaddr from 'multiaddr'
import BL from 'bl'
import { MultiaddrConnection, Stream } from './types'
import Defer, { DeferredPromise } from 'p-defer'
import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, RELAY_WEBRTC_PREFIX, RESTART, STOP } from './constants'
import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'

import type { Instance as SimplePeer } from 'simple-peer'

import type PeerId from 'peer-id'

import Debug from 'debug'

const log = Debug('hopr-core:transport')
const error = Debug('hopr-core:transport:error')

class RelayConnection implements MultiaddrConnection {
  private _defer: DeferredPromise<void>
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean
  private _onReconnect: () => void

  private webRTC: SimplePeer
  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  public source: Stream['source']
  public sink: Stream['sink']

  public conn: Stream

  public timeline: {
    open: number
    close?: number
  }

  constructor({
    stream,
    self,
    counterparty,
    webRTC,
    onReconnect
  }: {
    stream: Stream
    self: PeerId
    counterparty: PeerId
    onReconnect: () => void
    webRTC?: SimplePeer
  }) {
    this.timeline = {
      open: Date.now()
    }

    this._defer = Defer()

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = stream

    this._onReconnect = onReconnect

    this.localAddr = Multiaddr(`/p2p/${self.toB58String()}`)
    this.remoteAddr = Multiaddr(`/p2p/${counterparty.toB58String()}`)

    this.webRTC = webRTC

    this.source = this._createSource.call(this)

    this.sink = this._createSink.bind(this)
  }

  private async *_createSource() {
    let promiseDone = false

    const promise = this._defer.promise.then(() => {
      promiseDone = true
    })

    let streamResolved = false
    let streamMsg: Uint8Array | void
    let streamDone = false

    function streamSourceFunction({ done, value }: { done?: boolean; value?: Uint8Array | void }) {
      streamResolved = true
      streamMsg = value

      if (done) {
        streamDone = done
      }
    }

    let streamPromise = this._stream.source.next().then(streamSourceFunction)

    while (true) {
      await Promise.race([
        // prettier-ignore
        streamPromise,
        promise
      ])

      if (streamResolved) {
        streamResolved = false

        if (streamMsg != null) {
          const received = (streamMsg as Uint8Array).slice()

          const [PREFIX, SUFFIX] = [received.subarray(0, 1), received.subarray(1)]

          if (u8aEquals(PREFIX, RELAY_PAYLOAD_PREFIX)) {
            if (streamDone || promiseDone) {
              this._destroyed = true
              return SUFFIX
            } else {
              yield SUFFIX
            }
          } else if (u8aEquals(PREFIX, RELAY_STATUS_PREFIX)) {
            if (u8aEquals(SUFFIX, STOP) || streamDone || promiseDone) {
              this._destroyed = true
              return
            } else if (u8aEquals(SUFFIX, RESTART)) {
              this._onReconnect()
              log(`RESTART received`)

            } else {
              error(`Received invalid status message ${received.slice(1)}. Dropping message.`)
            }
          } else if (u8aEquals(PREFIX, RELAY_WEBRTC_PREFIX)) {
            // console.log(`Receiving fancy WebRTC message`, JSON.parse(new TextDecoder().decode(received.slice(1))))
            this.webRTC?.signal(JSON.parse(new TextDecoder().decode(received.slice(1))))
          } else {
            error(`Received invalid prefix <${u8aToHex(PREFIX)}. Dropping message.`)
          }

          streamPromise = this._stream.source.next().then(streamSourceFunction)
        }
      } else if (promiseDone || streamDone) {
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
  }

  private async _createSink(source: Stream['source']) {
    this._sinkTriggered = true

    return this._stream.sink(
      async function* (this: RelayConnection) {
        let promiseDone = false

        let streamResolved = false
        let streamDone = false
        let streamMsg: Uint8Array | void

        let webRTCresolved = false
        let webRTCdone = this.webRTC == null
        let webRTCmsg: Uint8Array | void

        function streamSourceFunction({ done, value }: { done?: boolean; value?: Uint8Array | void }) {
          streamResolved = true
          streamMsg = value

          if (done) {
            streamDone = true
          }
        }

        let streamPromise = source.next().then(streamSourceFunction)

        function webRTCSourceFunction({ done, value }: { done?: boolean; value: Uint8Array | void }) {
          webRTCresolved = true
          webRTCmsg = value

          if (done) {
            webRTCdone = true
          }
        }

        let webRTCstream: Stream['source']

        if (this.webRTC != null) {
          webRTCstream = async function* (this: RelayConnection) {
            let defer = Defer<void>()
            let waiting = false
            const webRTCmessages: Uint8Array[] = []
            let done = false
            function onSignal(msg: any) {
              webRTCmessages.push(new TextEncoder().encode(JSON.stringify(msg)))
              if (waiting) {
                waiting = false
                let tmpPromise = defer
                defer = Defer<void>()
                tmpPromise.resolve()
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

              await defer.promise

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
              // prettier-ignore
              streamPromise,
              webRTCPromise,
              promise
            ])
          } else {
            await Promise.race([
              // prettier-ignore
              streamPromise,
              promise
            ])
          }

          if (streamResolved && streamMsg != null) {
            streamResolved = false

            if (promiseDone || (streamDone && webRTCdone)) {
              if (streamMsg != null) {
                yield new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (streamMsg as unknown) as BL])
              }

              this._destroyed = true

              return (new BL([
                (RELAY_STATUS_PREFIX as unknown) as BL,
                (STOP as unknown) as BL
              ]) as unknown) as Uint8Array
            } else {
              // Drop empty messages
              if (streamMsg != null) {
                yield new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (streamMsg as unknown) as BL])
              }

              streamPromise = source.next().then(streamSourceFunction)
            }
          } else if (webRTCresolved && webRTCmsg != null) {
            webRTCresolved = false
            yield new BL([(RELAY_WEBRTC_PREFIX as unknown) as BL, (webRTCmsg as unknown) as BL])

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

  get destroyed(): boolean {
    return this._destroyed
  }

  close(err?: Error): Promise<void> {
    this._defer.resolve()

    this.timeline.close = Date.now()

    return Promise.resolve()
  }
}

export { RelayConnection }
