/// <reference path="../@types/it-pair.ts" />
/// <reference path="../@types/it-handshake.ts" />

import { RelayConnection, statusMessagesCompare } from './connection'
import assert from 'assert'
import { u8aEquals } from '@hoprnet/hopr-utils'

import PeerId from 'peer-id'
import { EventEmitter, once } from 'events'
import Pair from 'it-pair'
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants'
import handshake from 'it-handshake'
import { StreamType } from '../types'

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
    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

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

    const relayShaker = handshake({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    relayShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING))

    assert(
      u8aEquals((await relayShaker.read()).slice(), Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))
    )
  })

  it('forward payload', async function () {
    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

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

    const relayShaker = handshake({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    const aliceShaker = handshake({
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
    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

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

    const relayShaker = handshake({
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
    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

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

    const relayShaker = handshake({
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

    for await (const _msg of relayShaker.stream.source) {
      assert.fail(`Stream must have ended`)
    }

    for await (const _msg of alice.source) {
      assert.fail(`Stream must have ended`)
    }

    assert(alice.destroyed, `Stream must be destroyed`)

    assert(
      alice.timeline.close != undefined && Date.now() >= alice.timeline.close,
      `Timeline object must have been populated`
    )
  })

  it('reconnect before using stream and use new stream', async function () {
    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

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

    const aliceShakerBeforeReconnect = handshake({
      source: alice.source,
      sink: alice.sink
    })

    const relayShaker = handshake({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    // try to read something
    aliceShakerBeforeReconnect.read()

    relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART))

    await once(alice, 'restart')

    aliceShakerBeforeReconnect.write(new TextEncoder().encode('Hello from Alice before reconnect'))

    assert(aliceAfterReconnect != undefined)

    const aliceShaker = handshake({
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
    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

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

    const aliceShakerBeforeReconnect = handshake({
      source: alice.source,
      sink: alice.sink
    })

    const relayShaker = handshake({
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

      aliceShakerBeforeReconnect.write(new TextEncoder().encode('Hello from Alice before reconnect'))

      assert(aliceAfterReconnect != undefined)

      const aliceShaker = handshake({
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

  it('forward and prefix WebRTC messages', async function () {
    class WebRTC extends EventEmitter {
      signal(args: any) {
        this.emit('incoming msg', args)
      }
    }

    const webRTC = new WebRTC()

    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

    new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async () => {},
      webRTC: {
        channel: webRTC as any,
        upgradeInbound: (): any => {}
      }
    })

    const relayShaker = handshake({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    const webRTCHello = 'WebRTC Hello'

    relayShaker.write(
      Uint8Array.from([
        RelayPrefix.WEBRTC_SIGNALLING,
        ...new TextEncoder().encode(
          JSON.stringify({
            message: webRTCHello
          })
        )
      ])
    )

    const result = await once(webRTC, 'incoming msg')
    assert(result != undefined && result.length > 0 && result[0]?.message === webRTCHello)

    const webRTCResponse = 'webRTC hello back'
    webRTC.emit('signal', {
      message: webRTCResponse
    })

    assert(JSON.parse(new TextDecoder().decode((await relayShaker.read()).slice(1))).message === webRTCResponse)
  })

  it('forward and prefix WebRTC messages after reconnect', async function () {
    class WebRTC extends EventEmitter {
      signal(args: any) {
        this.emit('incoming msg', args)
      }
    }

    const webRTC = new WebRTC()

    const AliceRelay = Pair<StreamType>()
    const RelayAlice = Pair<StreamType>()

    let webRTCAfterReconnect: WebRTC | undefined

    const alice = new RelayConnection({
      stream: {
        sink: AliceRelay.sink,
        source: RelayAlice.source
      },
      self: Alice,
      relay: Relay,
      counterparty: Bob,
      onReconnect: async () => {},
      webRTC: {
        channel: webRTC as any,
        upgradeInbound: (): any => {
          webRTCAfterReconnect = new WebRTC()
          return webRTCAfterReconnect
        }
      }
    })

    const relayShaker = handshake({
      sink: RelayAlice.sink,
      source: AliceRelay.source
    })

    const webRTCHello = 'WebRTC Hello'

    relayShaker.write(
      Uint8Array.from([
        RelayPrefix.WEBRTC_SIGNALLING,
        ...new TextEncoder().encode(
          JSON.stringify({
            message: webRTCHello
          })
        )
      ])
    )

    const result = await once(webRTC, 'incoming msg')
    assert(result != undefined && result.length > 0 && result[0]?.message === webRTCHello)

    const webRTCResponse = 'webRTC hello back'
    webRTC.emit('signal', {
      message: webRTCResponse
    })

    assert(JSON.parse(new TextDecoder().decode((await relayShaker.read()).slice(1))).message === webRTCResponse)

    const ATTEMPTS = 5
    for (let i = 0; i < ATTEMPTS; i++) {
      relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART))

      await once(alice, 'restart')

      webRTC.emit('signal', {
        message: webRTCResponse
      })

      const correctWebRTCResponseAfterReconnect = 'webRTC hello back after response'

      assert(webRTCAfterReconnect != undefined)

      // Emitting unnecessary event that must not come through
      webRTCAfterReconnect.emit('signal', {
        message: correctWebRTCResponseAfterReconnect
      })

      assert(
        JSON.parse(new TextDecoder().decode((await relayShaker.read()).slice(1))).message ===
          correctWebRTCResponseAfterReconnect
      )
    }
  })
})
