import { Connection, Stream } from './types'
import Multiaddr from 'multiaddr'
import BL from 'bl'
import { MultiaddrConnection } from './types'
import Defer, { DeferredPromise } from 'p-defer'
import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, STOP } from './constants'

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

      let msg: Promise<IteratorResult<Uint8Array, void>>

      this._defer.promise.then(() => {
        itDone = true
      })

      while (true) {
        msg = (this._stream.source as AsyncGenerator<Uint8Array, void>).next()

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

          if (itDone) {
            return (await msg).value
          } else {
            yield (await msg).value
          }
        }

        if (itDone) {
          return
        }
      }
    }.call(this)
  }

  async sink(source: AsyncIterable<Uint8Array>) {
    return (async function* () {
      for await (const msg of source) {
        yield new BL([(RELAY_PAYLOAD_PREFIX as unknown) as BL, (msg as unknown) as BL]) as unknown as Uint8Array
      }

      yield new BL([(RELAY_STATUS_PREFIX as unknown) as BL, (STOP as unknown) as BL]) as unknown as Uint8Array
    })()
  }

  async close(err?: Error) {
    this._defer.resolve()

    this.timeline.close = Date.now()
  }
}

export { RelayConnection }
