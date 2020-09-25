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
    this._stream = stream

    this.source = async function* (this: RelayConnection) {
      let msgReceived = false
      let itDone = false

      let msg: Promise<IteratorResult<Uint8Array, Uint8Array | void>>

      this._defer.promise.then(() => {
        itDone = true
      })

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

        if (msgReceived) {
          msgReceived = false

          const _received = (await msg).value

          if (_received == null) {
            // change this to continue to accept **empty** messages
            return
          }

          const received = (_received as Uint8Array).slice()

          if (u8aEquals(received.slice(0, 1), RELAY_PAYLOAD_PREFIX)) {
            if (itDone) {
              return received.slice(1)
            } else {
              yield received.slice(1)
            }
          } else if (u8aEquals(received.slice(0, 1), RELAY_STATUS_PREFIX)) {
            if (u8aEquals(received.slice(1), STOP)) {
              return
            } else {
                error(`Received invalid status message ${received.slice(1)}. Dropping message.`)
            }
          } else {
            error(`Received invalid prefix <${received.slice(1)}. Dropping message.`)
          }
        }

        if (itDone) {
          return
        }
      }
    }.call(this)
  }

  sink(source: AsyncIterable<Uint8Array>): Promise<void> {
    return this._stream.sink(
      (async function* () {
        for await (const msg of source) {
          yield (new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (msg as unknown) as BL]) as unknown) as Uint8Array
        }

        yield (new BL([(RELAY_STATUS_PREFIX as unknown) as BL, (STOP as unknown) as BL]) as unknown) as Uint8Array
      })()
    )
  }

  async close(err?: Error) {
    this._defer.resolve()

    this.timeline.close = Date.now()
  }
}

export { RelayConnection }
