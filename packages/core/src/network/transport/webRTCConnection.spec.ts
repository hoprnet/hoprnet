import { WebRTCConnection } from './webRTCConnection'
import Peer from 'simple-peer'
import { yellow } from 'chalk'

// @ts-ignore
import wrtc = require('wrtc')

import { u8aConcat } from '@hoprnet/hopr-utils'
import { RELAY_PAYLOAD_PREFIX } from './constants'
import { RelayContext } from './relayContext'
import { RelayConnection } from './relayConnection'
import type { Stream } from 'libp2p'
//import assert from 'assert'

import Pair from 'it-pair'

import PeerId from 'peer-id'

describe('test overwritable connection', function () {
  let iteration = 0

  function getStream({ usePrefix }: { usePrefix: boolean }): Stream {
    let _iteration = iteration

    return {
      source: (async function* () {
        let i = 0
        let msg: Uint8Array
        for (; i < 7; i++) {
          msg = new TextEncoder().encode(`iteration ${_iteration} - msg no. ${i}`)
          if (usePrefix) {
            yield u8aConcat(RELAY_PAYLOAD_PREFIX, msg)
          } else {
            yield msg
          }

          await new Promise((resolve) => setTimeout(resolve, 10))
        }
      })(),
      sink: async (source: Stream['source']) => {
        let msg: Uint8Array
        for await (const _msg of source) {
          if (_msg != null) {
            if (usePrefix) {
              msg = _msg.slice(1)
            } else {
              msg = _msg.slice()
            }

            console.log(yellow(`receiver #${_iteration}`, new TextDecoder().decode(msg)))
          } else {
            console.log(`received empty message`, _msg)
          }
        }
        console.log(`sinkDone`)
      }
    }
  }

  it('should simulate a reconnect', async function () {
    const [partyA, partyB] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    const connectionA = [Pair(), Pair()]
    const connectionB = [Pair(), Pair()]

    const relaySideA = new RelayContext({
      sink: connectionA[0].sink,
      source: connectionA[1].source
    })

    const relaySideB = new RelayContext({
      sink: connectionB[0].sink,
      source: connectionB[1].source
    })

    relaySideA.sink(relaySideB.source)
    relaySideB.sink(relaySideA.source)

    const PeerA = new Peer({ wrtc, initiator: true, trickle: true })
    const PeerB = new Peer({ wrtc, trickle: true })

    const ctxA = new WebRTCConnection({
      conn: new RelayConnection({
        stream: {
          sink: connectionA[1].sink,
          source: connectionA[0].source
        },
        self: partyA,
        counterparty: partyB,
        webRTC: PeerA,
        onReconnect: async () => {}
      }),
      self: partyA,
      counterparty: partyB,
      channel: PeerA
    })

    const ctxB = new WebRTCConnection({
      conn: new RelayConnection({
        stream: {
          sink: connectionB[1].sink,
          source: connectionB[0].source
        },
        self: partyB,
        counterparty: partyB,
        webRTC: PeerB,
        onReconnect: async () => {}
      }),
      self: partyB,
      counterparty: partyA,
      channel: PeerB
    })

    const streamA = getStream({ usePrefix: false })
    iteration++
    const streamB = getStream({ usePrefix: false })

    ctxA.sink(streamA.source)
    streamA.sink(ctxA.source)

    ctxB.sink(streamB.source)
    streamB.sink(ctxB.source)

    // let pingPromise: Promise<number>
    // setTimeout(() => {
    //   iteration++
    //   pingPromise = ctxSender.ping()
    //   ctxCounterparty.update(getStream({ usePrefix: true }))
    // }, 200)

    // await new Promise((resolve) => setTimeout(resolve, 1000))

    // assert((await pingPromise) > 0)

    // await ctx.close()

    await new Promise((resolve) => setTimeout(resolve, 2000))
  })
})

// async function main() {
//   const AliceBob = Pair()
//   const BobAlice = Pair()

//   const Alice = await PeerId.create({ keyType: 'secp256k1' })
//   const Bob = await PeerId.create({ keyType: 'secp256k1' })

//   const PeerAlice = new Peer({ wrtc, initiator: true, trickle: true })
//   const PeerBob = new Peer({ wrtc, trickle: true })

//   // await new Promise(resolve => {
//   // })

//   const a = new WebRTCConnection({
//     conn: new RelayConnection({
//       stream: {
//         sink: AliceBob.sink,
//         source: BobAlice.source
//       } as MultiaddrConnection,
//       counterparty: Bob,
//       self: Alice,
//       webRTC: PeerAlice,
//       onReconnect: async () => {}
//     }),
//     self: Alice,
//     counterparty: Bob,
//     channel: PeerAlice
//   })

//   const b = new WebRTCConnection({
//     conn: new RelayConnection({
//       stream: {
//         sink: BobAlice.sink,
//         source: AliceBob.source
//       } as MultiaddrConnection,
//       self: Bob,
//       counterparty: Alice,
//       webRTC: PeerBob,
//       onReconnect: async () => {}
//     }),
//     self: Bob,
//     counterparty: Alice,
//     channel: PeerBob
//   })

//   a.sink(
//     (async function* () {
//       while (true) {
//         await new Promise((resolve) => setTimeout(resolve, 1000))
//         yield new TextEncoder().encode(`fancy WebRTC message`)
//       }
//     })()
//   )

//   // b.sink(
//   //   (async function* () {
//   //     while (true) {
//   //       await new Promise((resolve) => setTimeout(resolve, 1000))
//   //       yield new TextEncoder().encode(`fancy WebRTC message`)
//   //     }
//   //   })()
//   // )

//   function foo({ done, value }: { done?: boolean | void; value?: Uint8Array | void }) {
//     if (value) {
//       console.log(new TextDecoder().decode(value))
//     }

//     if (!done) {
//       b.source.next().then(foo)
//     }
//   }

//   // function bar({ done, value }: { done?: boolean | void; value?: Uint8Array | void }) {
//   //   if (value) {
//   //     console.log(new TextDecoder().decode(value))
//   //   }

//   //   if (!done) {
//   //     b.source.next().then(foo)
//   //   }
//   // }

//   b.source.next().then(foo)
//   //a.source.next().then(bar)

//   //PeerBob.on('signal', (msg: any) => setTimeout(() => PeerAlice.signal(msg), 150))

//   //PeerAlice.on('signal', (msg: any) => setTimeout(() => PeerBob.signal(msg), 150))
// }

// main()
