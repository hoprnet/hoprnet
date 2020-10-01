// import { u8aEquals } from '@hoprnet/hopr-utils'
// import Defer, { DeferredPromise } from 'p-defer'

// import { RELAY_PAYLOAD_PREFIX, RELAY_STATUS_PREFIX, STOP } from './constants'

// class RelayContext {
//   private _defer: DeferredPromise<AsyncGenerator<Uint8Array>>

//   public source: AsyncIterable<Uint8Array>

//   constructor(private _source: AsyncGenerator<Uint8Array>) {
//     this._defer = Defer<AsyncGenerator<Uint8Array>>()

//     this.source = async function* (this: RelayContext) {
//       let itDone = false

//       let msgReceived = false
//       let streamReceived = false

//       let msg: Promise<IteratorResult<Uint8Array, Uint8Array>>

//       this._defer.promise.then(() => {
//         streamReceived = true
//       })

//       while (true) {
//         msg = this._source.next()

//         await Promise.race([
//           msg.then(({ done }) => {
//             if (done) {
//               itDone = true
//             }

//             msgReceived = true
//           }),
//           this._defer.promise,
//         ])

//         if (itDone || streamReceived) {
//           console.log(`waiting for resolve`)
//           this._source = await this._defer.promise

//           this._defer = Defer()

//           streamReceived = false

//           this._defer.promise.then(() => {
//             console.log(`stream resolved`)
//             streamReceived = true
//           })

//           itDone = false
//           continue
//         }

//         if (msgReceived) {
//           const received = (await msg).value?.slice()

//           if (u8aEquals(received.slice(0, 1), RELAY_STATUS_PREFIX)) {
//             if (u8aEquals(received.slice(1), STOP)) {
//               console.log(`STOP received`)
//               break
//             } else {
//               throw Error(`Invalid status message. Got <${received.slice(1)}>`)
//             }
//           }
          
//           yield (await msg).value

//           msgReceived = false
//         }
//       }
//       console.log(`after relay context return `)
//     }.call(this)
//   }

//   update(newStream: AsyncGenerator<Uint8Array>) {
//     this._defer.resolve(newStream)
//   }
// }

// export { RelayContext }
