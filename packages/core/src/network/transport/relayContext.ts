import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import Defer, { DeferredPromise } from 'p-defer'

import Debug from 'debug'
const log = Debug(`hopr-core:transport`)
const error = Debug(`hopr-core:transport:error`)

import { RELAY_STATUS_PREFIX, STOP } from './constants'

class RelayContext {
  private _defer: DeferredPromise<AsyncGenerator<Uint8Array>>

  public source: AsyncIterable<Uint8Array>

  constructor(private _source: AsyncGenerator<Uint8Array>) {
    this._defer = Defer<AsyncGenerator<Uint8Array>>()

    this.source = async function* (this: RelayContext) {
      let itDone = false

      let msgReceived = false
      let streamReceived = false

      let msg: Promise<IteratorResult<Uint8Array, Uint8Array>>

      this._defer.promise.then(() => {
        streamReceived = true
      })

      while (true) {
        log(`relay iteration`)
        msg = this._source.next()

        await Promise.race([
          msg.then(({ done }) => {
            if (done) {
              itDone = true
            }

            msgReceived = true
          }),
          this._defer.promise
        ])

        if (itDone || streamReceived) {
          log(`waiting for resolve streamReceived ${streamReceived} itDone ${itDone}`)
          this._source = await this._defer.promise

          this._defer = Defer()

          streamReceived = false

          this._defer.promise.then(() => {
            log(`stream resolved`)
            streamReceived = true
          })

          itDone = false
          continue
        }

        if (msgReceived) {
          const received = (await msg).value?.slice()

          if (u8aEquals(received.slice(0, 1), RELAY_STATUS_PREFIX)) {
            if (u8aEquals(received.slice(1), STOP)) {
              log(`STOP received`)
              break
            } else {
              error(`Invalid status message. Got <${received.slice(1)}>`)
            }
          }

          log(`relaying ${(await msg).value.toString()}`, u8aToHex((await msg).value))
          yield (await msg).value

          msgReceived = false
        }
      }
      log(`after relay context return `)
    }.call(this)
  }

  update(newStream: AsyncGenerator<Uint8Array>) {
    this._defer.resolve(newStream)
  }
}

export { RelayContext }
