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

    this._defer.promise.then(() => {
      console.log(`promise resolved`)
    })

    this._destroyed = false
    this._sinkTriggered = false

    this._stream = stream

    this.source = async function* (this: RelayConnection) {
      let msgReceived = false
      let itDone = false

      this._defer.promise.then(() => {
        console.log(`source promise resolved`)

        itDone = true
      })

      let msg: Promise<IteratorResult<Uint8Array, Uint8Array | void>>

      while (true) {
        msg = (this._stream.source as AsyncGenerator<Uint8Array, Uint8Array | void>).next()

        await Promise.race([
          msg.then(({ done }) => {
            msgReceived = true

            if (done) {
              itDone = true
            }
          }),
          this._defer.promise,
        ])

        console.log(`itDone`, itDone, `msgReceived`, msgReceived)

        if (msgReceived) {
          msgReceived = false

          const _received = (await msg).value

          if (_received == null) {
            // change this to `return` to end the stream
            // once we receive an empty message
            continue
          }

          const received = (_received as Uint8Array).slice()

          if (u8aEquals(received.slice(0, 1), RELAY_PAYLOAD_PREFIX)) {
            if (itDone) {
              this._destroyed = true
              return received.slice(1)
            } else {
              yield received.slice(1)
            }
          } else if (u8aEquals(received.slice(0, 1), RELAY_STATUS_PREFIX)) {
            if (u8aEquals(received.slice(1), STOP) || itDone) {
              this._destroyed = true
              return
            } else {
              error(`Received invalid status message ${received.slice(1)}. Dropping message.`)
            }
          } else {
            error(`Received invalid prefix <${received.slice(1)}. Dropping message.`)
          }
        }

        if (itDone) {
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
          console.log(`inside last if statement`, `this._sinkTriggered`, this._sinkTriggered)
          return
        }
      }
    }.call(this)
  }

  get destroyed(): boolean {
    return this._destroyed
  }

  sink(source: AsyncIterable<Uint8Array>): Promise<void> {
    return this._stream.sink(
      async function* (this: RelayConnection) {
        this._sinkTriggered = true

        let msgReceived = false
        let itDone = false

        this._defer.promise.then(() => {
          console.log(`promise resolved`)
          itDone = true
        })

        let msg: Promise<IteratorResult<Uint8Array, Uint8Array | void>>

        while (true) {
          msg = (source as AsyncGenerator<Uint8Array, Uint8Array | void>).next()

          await Promise.race([
            msg.then(({ done }) => {
              msgReceived = true

              if (done) {
                itDone = true
              }
            }),
            this._defer.promise,
          ])

          console.log(`sink`, `itDone`, itDone, `msgReceived`, msgReceived)

          if (msgReceived) {
            msgReceived = false

            let _received = (await msg).value

            if (itDone) {
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
          }

          if (itDone && !this._destroyed) {
            this._destroyed = true

            return (new BL([(RELAY_STATUS_PREFIX as unknown) as BL, (STOP as unknown) as BL]) as unknown) as Uint8Array
          }
        }
      }.call(this)
    )
  }

  async close(err?: Error) {
    this._defer.resolve()

    this.timeline.close = Date.now()
  }
}

export { RelayConnection }
