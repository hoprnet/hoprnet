/// <reference path="./@types/it-pair.ts" />
/// <reference path="./@types/libp2p.ts" />

// @ts-nocheck
import { durations, u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'
import { PING, RELAY_PAYLOAD_PREFIX } from './constants'
import { RelayContext } from './relayContext'
import { RelayConnection } from './relayConnection'
import type { Stream } from 'libp2p'
import assert from 'assert'

import Pair from 'it-pair'

import PeerId from 'peer-id'

interface DebugMessage {
  messageId: number
  iteration: number
}

describe('test overwritable connection', function () {
  let iteration = 0

  /**
   * Starts a duplex stream that sends numbered messages and checks
   * whether the received messages came from the same sender and in
   * expected order.
   * @param arg options
   */
  // function getStream(arg: { usePrefix: boolean; designatedReceiverId?: number }): Stream {
  //   let _iteration = iteration

  //   let lastId = -1
  //   let receiver = arg.designatedReceiverId

  //   return {
  //     source: (async function* () {
  //       let i = 0
  //       let msg: Uint8Array

  //       for (; i < 15; i++) {
  //         msg = new TextEncoder().encode(
  //           JSON.stringify({
  //             iteration: _iteration,
  //             messageId: i
  //           })
  //         )

  //         if (arg.usePrefix) {
  //           yield u8aConcat(RELAY_PAYLOAD_PREFIX, msg)
  //         } else {
  //           yield msg
  //         }

  //         await new Promise((resolve) => setTimeout(resolve, 10))
  //       }
  //     })(),
  //     sink: async (source: Stream['source']) => {
  //       let msg: Uint8Array

  //       for await (const _msg of source) {
  //         if (_msg != null) {
  //           if (arg.usePrefix) {
  //             if (!u8aEquals(_msg.slice(0, 1), RELAY_PAYLOAD_PREFIX)) {
  //               continue
  //             }

  //             if (_msg.slice(1).length == 0) {
  //               continue
  //             }

  //             msg = _msg.slice(1)
  //           } else {
  //             msg = _msg.slice()
  //           }

  //           let decoded = JSON.parse(new TextDecoder().decode(msg)) as DebugMessage

  //           assert(decoded.messageId == lastId + 1)

  //           if (receiver == null) {
  //             receiver = decoded.iteration
  //           } else {
  //             assert(receiver == decoded.iteration)
  //           }

  //           lastId = decoded.messageId
  //         } else {
  //           assert.fail(`received empty message`)
  //         }
  //       }
  //     }
  //   }
  // }

  // it('should simulate a reconnect', async function () {
  //   this.timeout(durations.seconds(3))

  //   // Sample two IDs
  //   const [self, counterparty] = await Promise.all(
  //     Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
  //   )

  //   // Get low-level connections between A, B
  //   const connectionA = Pair()
  //   const connectionB = Pair()

  //   let newSenderId = -1

  //   // Generate sender-side of relay
  //   const ctxSender = new RelayContext({
  //     sink: connectionA.sink,
  //     source: connectionB.source
  //   })

  //   // Generate counterparty-side of relay
  //   const ctxCounterparty = new RelayContext(getStream({ usePrefix: true }))

  //   // Internally wiring relay
  //   ctxSender.sink(ctxCounterparty.source)
  //   ctxCounterparty.sink(ctxSender.source)

  //   // Getting a demo stream
  //   iteration++
  //   const receiverStream = getStream({ usePrefix: false })

  //   const ctx = new RelayConnection({
  //     stream: {
  //       sink: connectionB.sink,
  //       source: connectionA.source
  //     },
  //     self,
  //     counterparty,
  //     onReconnect: async (newStream: RelayConnection) => {
  //       iteration++
  //       const demoStream = getStream({ usePrefix: false, designatedReceiverId: newSenderId })

  //       newStream.sink(demoStream.source)
  //       demoStream.sink(newStream.source)
  //     }
  //   })

  //   ctx.sink(receiverStream.source)
  //   receiverStream.sink(ctx.source)

  //   let pingPromise: Promise<number>

  //   // Trigger a reconnect after a timeout
  //   setTimeout(async () => {
  //     iteration++
  //     await new Promise((resolve) => setTimeout(resolve, 100))
  //     pingPromise = ctxSender.ping()
  //     newSenderId = iteration
  //     ctxCounterparty.update(getStream({ usePrefix: true }))
  //   }, 200)

  //   await new Promise((resolve) => setTimeout(resolve, 2500))
  //   console.log(`TIMEOUT done`)

  //   // Make sure that we the ping went through
  //   // Note that `result == -1` means timeout
  //   console.log(`PING PROMISE`, await pingPromise!)
  //   // assert((await pingPromise!) >= 0)

  //   await ctx.close()
  // })

  it('should perform a low-level ping', async function () {
    const [self, counterparty] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    // Get low-level connections between A, B
    const connectionA = Pair()
    const connectionB = Pair()
    new RelayConnection({
      stream: {
        sink: connectionB.sink,
        source: connectionA.source
      },
      self,
      counterparty,
      onReconnect: async () => {}
    })
    const ctxRelay = new RelayContext({
      sink: connectionA.sink,
      source: connectionB.source
    })

    // ctxSender.sink((async function * () {
    //   yield new Uint8Array([RELAY_PAYLOAD_PREFIX, 1])
    // })())

    const PING_ATTEMPTS = 4
    for (let i = 0; i < PING_ATTEMPTS; i++) {
      assert((await ctxRelay.ping()) >= 0, 'Ping must not timeout')
    }

    await new Promise((resolve) => setTimeout(resolve, 1000))
  })

  it('should echo messages a low-level ping', async function () {
    const [self, counterparty] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    // Get low-level connections between A, B
    const connectionA = Pair()
    const connectionB = Pair()
    new RelayConnection({
      stream: {
        sink: connectionB.sink,
        source: connectionA.source
      },
      self,
      counterparty,
      onReconnect: async () => {}
    })

    const ctxSender = new RelayContext({
      sink: connectionA.sink,
      source: connectionB.source
    })

    await new Promise((resolve) => setTimeout(resolve, 1000))
  })
})
