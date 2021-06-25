/// <reference path="../@types/it-pair.ts" />
/// <reference path="../@types/it-handshake.ts" />

import { RelayConnection, statusMessagesCompare } from './connection'
import type { Stream, StreamResult } from 'libp2p'
import assert from 'assert'
import { randomInteger, u8aEquals } from '@hoprnet/hopr-utils'
import pipe from 'it-pipe'

import PeerId from 'peer-id'
import { EventEmitter, once } from 'events'
import type { Instance as SimplePeer } from 'simple-peer'
import Pair from 'it-pair'
import { ConnectionStatusMessages, RelayPrefix, RELAY_STATUS_PREFIX, RESTART, StatusMessages } from '../constants'
import Defer from 'p-defer'
import handshake from 'it-handshake'
import { green } from 'chalk'

const TIMEOUT_LOWER_BOUND = 450
const TIMEOUT_UPPER_BOUND = 650

function createPeers(amount: number): Promise<PeerId[]> {
  return Promise.all(Array.from({ length: amount }, (_) => PeerId.create({ keyType: 'secp256k1' })))
}

describe('test status message sorting', function () {
  it('sort status messages', function () {
    const arr = [
      Uint8Array.of(RelayPrefix.PAYLOAD),
      Uint8Array.of(RelayPrefix.WEBRTC_SIGNALLING),
      Uint8Array.of(RelayPrefix.STATUS_MESSAGE),
      Uint8Array.of(RelayPrefix.CONNECTION_STATUS)
    ]

    const sortedArr = [...arr].sort(statusMessagesCompare)

    assert(sortedArr.every((value: Uint8Array, index: number) => value == arr[arr.length - index - 1]))
  })
})

describe('relay connection', function () {
  let Alice: PeerId, Relay: PeerId, Bob: PeerId

  before(async function () {
    ;[Alice, Relay, Bob] = await Promise.all(Array.from({ length: 3 }, (_) => PeerId.create({ keyType: 'secp256k1' })))
  })

  it('ping message', async function () {
    const AliceRelay = Pair()
    const RelayAlice = Pair()

    new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async () => {}
    })

    const relayShaker = handshake<Uint8Array>({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    relayShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING))

    assert(
      u8aEquals((await relayShaker.read()).slice(), Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))
    )
  })

  it('forward payload', async function () {
    const AliceRelay = Pair()
    const RelayAlice = Pair()

    const alice = new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async () => {}
    })

    const relayShaker = handshake<Uint8Array>({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    const aliceShaker = handshake<Uint8Array>({
      source: alice.source,
      sink: alice.sink
    })

    const AMOUNT = 5
    for (let i = 0; i < AMOUNT; i++) {
      const relayHello = new TextEncoder().encode('Hello from Relay')
      relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHello]))

      assert(u8aEquals((await aliceShaker.read()).slice(), relayHello))

      const aliceHello = new TextEncoder().encode('Hello from Alice')
      aliceShaker.write(aliceHello)

      assert(u8aEquals((await relayShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHello])))
    }
  })

  it('stop a relayed connection from the relay', async function () {
    const AliceRelay = Pair()
    const RelayAlice = Pair()

    const alice = new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async () => {}
    })

    const relayShaker = handshake<Uint8Array>({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

    relayShaker.rest()

    for await (const _msg of alice.source) {
      assert.fail(`Stream should be closed`)
    }

    for await (const _msg of relayShaker.stream.source) {
      assert.fail(`Stream should be closed`)
    }

    assert(alice.destroyed, `Stream must be destroyed`)

    assert(
      alice.timeline.close != undefined && Date.now() >= alice.timeline.close,
      `Timeline object must have been populated`
    )
  })

  it('stop a relayed connection from the client', async function () {
    const AliceRelay = Pair()
    const RelayAlice = Pair()

    const alice = new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async () => {}
    })

    const relayShaker = handshake<Uint8Array>({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    alice.close()

    assert(
      u8aEquals(
        (await relayShaker.read()).slice(),
        Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
      )
    )

    relayShaker.rest()

    for await (const msg of relayShaker.stream.source) {
      assert.fail(`Stream must have ended`)
    }

    for await (const msg of alice.source) {
      assert.fail(`Stream must have ended`)
    }

    assert(alice.destroyed, `Stream must be destroyed`)

    assert(
      alice.timeline.close != undefined && Date.now() >= alice.timeline.close,
      `Timeline object must have been populated`
    )
  })

  it('reconnect before using stream and use new stream', async function () {
    const AliceRelay = Pair()
    const RelayAlice = Pair()

    let aliceAfterReconnect: RelayConnection | undefined

    const alice = new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async (newStream: RelayConnection) => {
        aliceAfterReconnect = newStream
      }
    })

    const aliceShakerBeforeReconnect = handshake<Uint8Array>({
      source: alice.source,
      sink: alice.sink
    })

    const relayShaker = handshake<Uint8Array>({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    // try to read something
    aliceShakerBeforeReconnect.read()

    relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART))

    await once(alice, 'restart')

    aliceShakerBeforeReconnect.write(new TextEncoder().encode('a'))

    assert(aliceAfterReconnect != undefined)

    const aliceShaker = handshake<Uint8Array>({
      sink: aliceAfterReconnect.sink,
      source: aliceAfterReconnect.source
    })

    const relayHelloAfterReconnect = new TextEncoder().encode('Hello after reconnect!')
    relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHelloAfterReconnect]))

    assert(u8aEquals((await aliceShaker.read()).slice(), relayHelloAfterReconnect))

    const aliceHelloAfterReconnect = new TextEncoder().encode('Hello from Alice after reconnect!')

    aliceShaker.write(aliceHelloAfterReconnect)
    assert(
      u8aEquals((await relayShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHelloAfterReconnect]))
    )
  })

  it('reconnect before using stream and use new stream', async function () {
    const AliceRelay = Pair()
    const RelayAlice = Pair()

    let aliceAfterReconnect: RelayConnection | undefined

    const alice = new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async (newStream: RelayConnection) => {
        aliceAfterReconnect = newStream
      }
    })

    const aliceShakerBeforeReconnect = handshake<Uint8Array>({
      source: alice.source,
      sink: alice.sink
    })

    const relayShaker = handshake<Uint8Array>({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    let aliceHelloBeforeReconnect = new TextEncoder().encode(`Hello from Alice before reconnecting`)
    aliceShakerBeforeReconnect.write(aliceHelloBeforeReconnect)

    assert(
      u8aEquals(
        (await relayShaker.read()).slice(),
        Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHelloBeforeReconnect])
      )
    )

    let relayHelloBeforeReconnect = new TextEncoder().encode(`Hello from relay before reconnecting`)
    relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHelloBeforeReconnect]))

    assert(u8aEquals((await aliceShakerBeforeReconnect.read()).slice(), relayHelloBeforeReconnect))

    relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART))

    await once(alice, 'restart')

    aliceShakerBeforeReconnect.write(new TextEncoder().encode('a'))

    assert(aliceAfterReconnect != undefined)

    const aliceShaker = handshake<Uint8Array>({
      sink: aliceAfterReconnect.sink,
      source: aliceAfterReconnect.source
    })

    const relayHelloAfterReconnect = new TextEncoder().encode('Hello after reconnect!')
    relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHelloAfterReconnect]))

    assert(u8aEquals((await aliceShaker.read()).slice(), relayHelloAfterReconnect))

    const aliceHelloAfterReconnect = new TextEncoder().encode('Hello from Alice after reconnect!')

    aliceShaker.write(aliceHelloAfterReconnect)
    assert(
      u8aEquals((await relayShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHelloAfterReconnect]))
    )
  })
})

// describe.skip('test relay connection', function () {
//   it('should initiate a relayConnection and let the receiver close the connection prematurely', async function () {
//     const AliceBob = Pair()
//     const BobAlice = Pair()

//     const [Alice, DummyRelay, Bob] = await createPeers(3)

//     const a = new RelayConnection({
//       stream: {
//         sink: AliceBob.sink,
//         source: BobAlice.source
//       },
//       self: Alice,
//       relay: DummyRelay,
//       counterparty: Bob,
//       onReconnect: async () => {}
//     })

//     const b = new RelayConnection({
//       stream: {
//         sink: BobAlice.sink,
//         source: AliceBob.source
//       },
//       self: Bob,
//       relay: DummyRelay,
//       counterparty: Alice,
//       onReconnect: async () => {}
//     })

//     a.sink(
//       (async function* () {
//         let i = 0
//         while (i < 17) {
//           yield new TextEncoder().encode(`message ${i++}`)
//           await new Promise((resolve) => setTimeout(resolve, 100))
//         }
//       })()
//     )

//     setTimeout(() => setImmediate(() => b.close()), randomInteger(TIMEOUT_LOWER_BOUND, TIMEOUT_UPPER_BOUND))

//     pipe(
//       // prettier-ignore
//       b,
//       async function (source: Stream['source']) {
//         for await (const msg of source) {
//           console.log(new TextDecoder().decode(msg.slice()))
//         }
//       }
//     )

//     for await (const _msg of a.source) {
//       throw Error(`there should be no message`)
//     }

//     console.log(a._id, a.source.next(), b._id, b.source.next())
//     assert(
//       (
//         await Promise.all([
//           // prettier-ignore
//           a.source.next(),
//           b.source.next()
//         ])
//       ).every(({ done }) => done),
//       `Streams must have ended.`
//     )
//     assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
//   })

//   it('should initiate a relayConnection and close the connection by the sender prematurely', async function () {
//     const AliceBob = Pair()
//     const BobAlice = Pair()
//     const [Alice, DummyRelay, Bob] = await createPeers(3)

//     const a = new RelayConnection({
//       stream: {
//         sink: AliceBob.sink,
//         source: BobAlice.source
//       },
//       self: Alice,
//       relay: DummyRelay,
//       onReconnect: async () => {},
//       counterparty: Bob
//     })

//     const b = new RelayConnection({
//       stream: {
//         sink: BobAlice.sink,
//         source: AliceBob.source
//       },
//       self: Bob,
//       relay: DummyRelay,
//       counterparty: Alice,
//       onReconnect: async () => {}
//     })

//     a.sink(
//       (async function* () {
//         let i = 0
//         while (true) {
//           yield new TextEncoder().encode(`message ${i++}`)

//           await new Promise((resolve) => setTimeout(resolve, 100))
//         }
//       })()
//     )
//     setTimeout(() => setImmediate(() => a.close()), randomInteger(TIMEOUT_LOWER_BOUND, TIMEOUT_UPPER_BOUND))

//     for await (const msg of b.source) {
//       console.log(new TextDecoder().decode(msg.slice()))
//     }

//     for await (const _msg of a.source) {
//       throw Error(`there should be no message`)
//     }

//     await new Promise((resolve) => setTimeout(resolve, 50))

//     assert(
//       (
//         await Promise.all([
//           // prettier-ignore
//           a.source.next(),
//           b.source.next()
//         ])
//       ).every(({ done }) => done),
//       `Streams must have ended.`
//     )
//     assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
//   })

//   it('should initiate a relayConnection and exchange messages and destroy the connection after a random timeout', async function () {
//     const AliceBob = Pair()
//     const BobAlice = Pair()

//     const [Alice, DummyRelay, Bob] = await createPeers(3)

//     const FakeWebRTCAlice = new EventEmitter() as SimplePeer
//     FakeWebRTCAlice.signal = (msg: string) => console.log(`received fancy WebRTC message`, msg)

//     const FakeWebRTCBob = new EventEmitter() as SimplePeer
//     FakeWebRTCBob.signal = (msg: string) => console.log(`received fancy WebRTC message`, msg)

//     const interval = setInterval(() => FakeWebRTCAlice.emit(`signal`, { msg: 'Fake signal' }), 50)
//     setTimeout(() => {
//       clearInterval(interval)
//       FakeWebRTCAlice.emit('connect')
//     }, 200)

//     const a = new RelayConnection({
//       stream: {
//         sink: AliceBob.sink,
//         source: BobAlice.source
//       },
//       self: Alice,
//       relay: DummyRelay,
//       counterparty: Bob,
//       onReconnect: async () => {},
//       webRTC: {
//         channel: FakeWebRTCAlice as SimplePeer,
//         upgradeInbound: () => FakeWebRTCAlice as SimplePeer
//       }
//     })

//     const b = new RelayConnection({
//       stream: {
//         sink: BobAlice.sink,
//         source: AliceBob.source
//       },
//       self: Bob,
//       relay: DummyRelay,
//       counterparty: Alice,
//       onReconnect: async () => {},
//       webRTC: {
//         channel: FakeWebRTCBob as SimplePeer,
//         upgradeInbound: () => FakeWebRTCBob as SimplePeer
//       }
//     })

//     a.sink(
//       (async function* () {
//         let i = 0
//         while (true) {
//           yield new TextEncoder().encode(
//             JSON.stringify({
//               text: `message from a ${i++}`
//             })
//           )

//           await new Promise((resolve) => setTimeout(resolve, 100))
//         }
//       })()
//     )

//     b.sink(
//       (async function* () {
//         let i = 0
//         await new Promise((resolve) => setTimeout(resolve, 50))

//         while (true) {
//           yield new TextEncoder().encode(
//             JSON.stringify({
//               text: `message from b ${i++}`
//             })
//           )

//           await new Promise((resolve) => setTimeout(resolve, 100))
//         }
//       })()
//     )

//     setTimeout(() => setImmediate(() => a.close()), randomInteger(TIMEOUT_LOWER_BOUND, TIMEOUT_UPPER_BOUND))

//     let msgAReceived = false
//     let msgBReceived = false

//     let aDone = false
//     let bDone = false

//     function aFunction(arg: StreamResult) {
//       msgAReceived = true
//       if (arg.done) {
//         aDone = true
//       }
//       return arg
//     }

//     function bFunction(arg: StreamResult) {
//       msgBReceived = true
//       if (arg.done) {
//         bDone = true
//       }
//       return arg
//     }

//     let msgA = a.source.next().then(aFunction)
//     let msgB = b.source.next().then(bFunction)

//     while (true) {
//       if (!aDone && !bDone) {
//         await Promise.race([
//           // prettier-ignore
//           msgA,
//           msgB
//         ])
//       } else if (aDone) {
//         await msgB
//       } else if (bDone) {
//         await msgA
//       } else {
//         break
//       }

//       if (msgAReceived || bDone) {
//         msgAReceived = false

//         if (aDone && bDone) {
//           break
//         } else {
//           console.log(new TextDecoder().decode(((await msgA).value as Uint8Array) || new Uint8Array()))
//         }

//         msgA = a.source.next().then(aFunction)
//       }

//       if (msgBReceived || aDone) {
//         msgBReceived = false

//         if (aDone && bDone) {
//           break
//         } else {
//           console.log(new TextDecoder().decode(((await msgB).value || new Uint8Array()).slice()))
//         }
//         msgB = b.source.next().then(bFunction)
//       }
//     }

//     assert(
//       (
//         await Promise.all([
//           // prettier-ignore
//           a.source.next(),
//           b.source.next()
//         ])
//       ).every(({ done }) => done),
//       `both stream should have ended`
//     )

//     assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
//   })

//   it('should trigger a reconnect before sending messages', async function () {
//     // Sample two IDs
//     const [self, relay, counterparty] = await createPeers(3)

//     let cutConnection = true

//     // Get low-level connections between A, B
//     const sideSelf = Pair()
//     const sideCounterparty = Pair()

//     let sideSelfRestarted = false
//     let sideCounterpartyRestarted = false

//     let selfSource = (async function* () {
//       if (cutConnection && !sideSelfRestarted) {
//         yield Uint8Array.from([...RELAY_STATUS_PREFIX, ...RESTART])
//         sideSelfRestarted = true
//       }
//       for await (const msg of sideCounterparty.source) {
//         if (cutConnection && !sideSelfRestarted) {
//           yield Uint8Array.from([...RELAY_STATUS_PREFIX, ...RESTART])
//           sideSelfRestarted = true
//         }

//         yield msg
//       }
//     })()

//     let counterpartySource = (async function* () {
//       if (cutConnection && !sideCounterpartyRestarted) {
//         yield Uint8Array.from([...RELAY_STATUS_PREFIX, ...RESTART])
//         sideCounterpartyRestarted = true
//       }

//       for await (const msg of sideSelf.source) {
//         if (cutConnection && !sideCounterpartyRestarted) {
//           yield Uint8Array.from([...RELAY_STATUS_PREFIX, ...RESTART])
//           sideCounterpartyRestarted = true
//         }

//         yield msg
//       }
//     })()

//     const TEST_MESSAGES = ['first', 'second', 'third'].map((x) => new TextEncoder().encode(x))

//     const selfReconnectTriggered = Defer<void>()

//     const ctxSelf = new RelayConnection({
//       stream: {
//         source: selfSource,
//         sink: sideSelf.sink
//       },
//       self,
//       relay,
//       counterparty,
//       onReconnect: async (newStream: RelayConnection, newCounterparty: PeerId) => {
//         assert(counterparty.equals(newCounterparty), `counterparty of new stream must match previous counterparty`)

//         newStream.sink(
//           (async function* () {
//             yield* TEST_MESSAGES
//           })()
//         )

//         selfReconnectTriggered.resolve()
//       }
//     })

//     const counterpartyReconnectTriggered = Defer<void>()
//     let counterpartyMessagesReceived = false

//     const ctxCounterparty = new RelayConnection({
//       stream: {
//         source: counterpartySource,
//         sink: sideCounterparty.sink
//       },
//       self: counterparty,
//       relay,
//       counterparty: self,
//       onReconnect: async (newStream: RelayConnection, newCounterparty: PeerId) => {
//         console.log(`in reconnect`)
//         assert(self.equals(newCounterparty))

//         let i = 0
//         for await (const msg of newStream.source) {
//           assert(u8aEquals(TEST_MESSAGES[i], msg.slice()))

//           if (i == TEST_MESSAGES.length - 1) {
//             counterpartyMessagesReceived = true
//           }
//           i++
//         }

//         await newStream.close()

//         // @TODO reconnected stream does not close properly
//         assert(newStream.destroyed)

//         counterpartyReconnectTriggered.resolve()
//       }
//     })

//     // Make sure that reconnect gets triggered
//     await Promise.all([selfReconnectTriggered.promise, counterpartyReconnectTriggered.promise])

//     assert(counterpartyMessagesReceived, `Counterparty must the receive all messages`)

//     await new Promise((resolve) => setTimeout(resolve, 100))

//     assert(ctxSelf.destroyed, `First instance must be destroyed`)
//     assert(ctxCounterparty.destroyed, `First instance must be destroyed`)
//   })
// })
