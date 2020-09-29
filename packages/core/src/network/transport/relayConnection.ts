import { Stream } from './types'
import Multiaddr from 'multiaddr'
import BL from 'bl'
import { MultiaddrConnection } from './types'
import Defer, { DeferredPromise } from 'p-defer'
import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, STOP } from './constants'
import { u8aEquals } from '@hoprnet/hopr-utils'

import Debug from 'debug'

const error = Debug('hopr-core:transport:error')

class RelayConnection implements MultiaddrConnection {
  private _defer: DeferredPromise<void>
  private _stream: Stream
  private _destroyed: boolean
  private _sinkTriggered: boolean
  public localAddr: Multiaddr
  public remoteAddr: Multiaddr

  public source: AsyncIterable<Uint8Array>

  public conn: Stream

  public timeline: {
    open: number
    close?: number
  }

  constructor({ stream }: { stream: Stream }) {
    this.timeline = {
      open: Date.now(),
    }

    this._defer = Defer()

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = stream

    this.source = async function* (this: RelayConnection) {
      const promise = this._defer.promise.then(() => ({ done: true, value: undefined }))

      while (true) {
        let { done, value } = await Promise.race([
          // prettier-ignore
          // @ts-ignore
          this._stream.source.next(),
          promise,
        ])

        if (value != null) {
          const received = (value as Uint8Array).slice()

          if (u8aEquals(received.slice(0, 1), RELAY_PAYLOAD_PREFIX)) {
            if (done) {
              this._destroyed = true
              return received.slice(1)
            } else {
              yield received.slice(1)
            }
          } else if (u8aEquals(received.slice(0, 1), RELAY_STATUS_PREFIX)) {
            if (u8aEquals(received.slice(1), STOP) || done) {
              this._destroyed = true
              return
            } else {
              error(`Received invalid status message ${received.slice(1)}. Dropping message.`)
            }
          } else {
            error(`Received invalid prefix <${received.slice(1)}. Dropping message.`)
          }
        } else if (done) {
          if (!this._destroyed) {
            if (!this._sinkTriggered) {
              this._stream.sink(
                (async function* () {
                  yield (new BL([
                    (RELAY_STATUS_PREFIX as unknown) as BL,
                    (STOP as unknown) as BL,
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
  }

  get destroyed(): boolean {
    return this._destroyed
  }

  sink(source: AsyncIterable<Uint8Array>): Promise<void> {
    this._sinkTriggered = true

    return this._stream.sink(
      async function* (this: RelayConnection) {
        const promise = this._defer.promise.then(() => {
          return { done: true, value: undefined }
        })

        while (true) {
          let { done, value } = await Promise.race([
            // prettier-ignore
            // @ts-ignore
            source.next(),
            promise,
          ])

          if (value != null) {
            let _received = value.slice()

            if (done) {
              if (_received != null) {
                yield new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (_received as unknown) as BL])
              }

              this._destroyed = true

              return (new BL([
                (RELAY_STATUS_PREFIX as unknown) as BL,
                (STOP as unknown) as BL,
              ]) as unknown) as Uint8Array
            } else {
              if (_received == null) {
                // @TODO change this to `return` to end the stream
                // once we receive an empty message
                continue
              }

              yield new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (_received as unknown) as BL])
            }
          } else if (done) {
            if (!this._destroyed) {
              this._destroyed = true

              return (new BL([
                (RELAY_STATUS_PREFIX as unknown) as BL,
                (STOP as unknown) as BL,
              ]) as unknown) as Uint8Array
            }

            return
          }
        }
      }.call(this)
    )
  }

  close(err?: Error): Promise<void> {
    this._defer.resolve()

    this.timeline.close = Date.now()

    return Promise.resolve()
  }
}

export { RelayConnection }
