import { u8aEquals } from '@hoprnet/hopr-utils'
import Defer, { DeferredPromise } from 'p-defer'

import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, STOP } from './constants'

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
        msg = this._source.next()

        await Promise.race([
          msg.then(({ done }) => {
            if (done) {
              itDone = true
            }

            msgReceived = true
          }),
          this._defer.promise,
        ])

        if (itDone || streamReceived) {
          console.log(`waiting for resolve`)
          this._source = await this._defer.promise

          this._defer = Defer()

          streamReceived = false

          this._defer.promise.then(() => {
            console.log(`stream resolved`)
            streamReceived = true
          })

          itDone = false
          continue
        }

        if (msgReceived) {
          const received = (await msg).value?.slice()

          if (u8aEquals(received.slice(0, 1), RELAY_STATUS_PREFIX)) {
            if (u8aEquals(received.slice(1), STOP)) {
              console.log(`STOP received`)
              break
            } else {
              throw Error(`Invalid status message. Got <${received.slice(1)}>`)
            }
          } else if (u8aEquals(received.slice(0, 1), RELAY_PAYLOAD_PREFIX)) {
            console.log(`relaying message <${new TextDecoder().decode((await msg).value.slice())}>`)
            yield (await msg).value.slice(1)
          }

          msgReceived = false
        }
      }
      console.log(`after relay context return `)
    }.call(this)
  }

  update(newStream: AsyncGenerator<Uint8Array>) {
    this._defer.resolve(newStream)
  }
}

export { RelayContext }

// +---------+
// |TEST CODE|
// +---------+
// let iteration = 0
// function getGenerator(): AsyncGenerator<Uint8Array> {
//   return (async function* () {
//     let i = 0
//     for (; i < 23; i++) {
//       yield new TextEncoder().encode(`iteration ${iteration} - msg no. ${i}`)
//       await new Promise((resolve) => setTimeout(resolve, 123))
//     }

//     return `iteration ${iteration} - msg no. ${i + 1}`
//   })()
// }

// async function main() {
//   const ctx = new RelayContext(getGenerator())

//   setInterval(() => {
//     ctx.update(getGenerator())
//     iteration++
//   }, 1234)

//   for await (const msg of ctx.source) {
//     console.log(new TextDecoder().decode(msg))
//   }
// }

// main()
