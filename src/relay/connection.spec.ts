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

    const ATTEMPTS = 5
    for (let i = 0; i < ATTEMPTS; i++) {
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
        u8aEquals(
          (await relayShaker.read()).slice(),
          Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHelloAfterReconnect])
        )
      )
    }
  })
})
