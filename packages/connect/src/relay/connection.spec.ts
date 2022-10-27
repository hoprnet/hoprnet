import { RelayConnection, RelayConnectionInterface, statusMessagesCompare } from './connection.js'
import assert from 'assert'
import { defer, u8aEquals } from '@hoprnet/hopr-utils'

import type { PeerId } from '@libp2p/interface-peer-id'
import { EventEmitter, once } from 'events'
import { pair } from 'it-pair'
import { duplexPair } from 'it-pair/duplex'
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants.js'
import { handshake } from 'it-handshake'
import type { StreamType } from '../types.js'
import { createPeerId } from '../base/utils.spec.js'
import type { ConnectComponents } from '../components.js'

class WebRTC extends EventEmitter {
  signal(args: any) {
    this.emit('incoming msg', args)
  }

  destroy() {}
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
  const Relay: PeerId = createPeerId()
  const Bob: PeerId = createPeerId()

  it('ping message', async function () {
    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    )

    const relayShaker = handshake(RelayAlice)

    relayShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING))
    const msg = (await relayShaker.read()) as Uint8Array
    const expectedMsg = Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG)

    assert(u8aEquals(msg.slice(), expectedMsg))
  })

  it('forward payload', async function () {
    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    const alice = RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    )

    const relayShaker = handshake(RelayAlice)

    const aliceShaker = handshake({
      source: alice.source,
      sink: alice.sink
    })

    const AMOUNT = 5
    for (let i = 0; i < AMOUNT; i++) {
      const relayHello = new TextEncoder().encode('Hello from Relay')
      relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHello]))

      assert(u8aEquals(((await aliceShaker.read()) as Uint8Array).slice(), relayHello))

      const aliceHello = new TextEncoder().encode('Hello from Alice')
      aliceShaker.write(aliceHello)

      assert(
        u8aEquals(
          ((await relayShaker.read()) as Uint8Array).slice(),
          Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHello])
        )
      )
    }
  })

  it('stop a relayed connection from the relay', async function () {
    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    const alice = RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    )

    const relayShaker = handshake(RelayAlice)

    relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

    relayShaker.rest()

    for await (const _msg of alice.source) {
      assert.fail(`Stream should be closed`)
    }

    for await (const _msg of relayShaker.stream.source as AsyncIterable<StreamType>) {
      assert.fail(`Stream should be closed`)
    }

    assert(alice.isDestroyed(), `Stream must be destroyed`)

    assert(
      alice.timeline.close != undefined && Date.now() >= alice.timeline.close,
      `Timeline object must have been populated`
    )
  })

  it('stop a relayed connection from the client', async function () {
    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    const alice = RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    )

    const relayShaker = handshake(RelayAlice)

    alice.close()

    assert(
      u8aEquals(
        ((await relayShaker.read()) as Uint8Array).slice(),
        Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
      )
    )

    relayShaker.rest()

    for await (const _msg of relayShaker.stream.source as AsyncIterable<StreamType>) {
      assert.fail(`Stream must have ended`)
    }

    for await (const _msg of alice.source) {
      assert.fail(`Stream must have ended`)
    }

    assert(alice.isDestroyed(), `Stream must be destroyed`)

    assert(
      alice.timeline.close != undefined && Date.now() >= alice.timeline.close,
      `Timeline object must have been populated`
    )
  })

  it('reconnect before using stream and use new stream', async function () {
    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    let aliceAfterReconnect: RelayConnectionInterface | undefined

    const restarted = defer<void>()

    const alice = RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async (newStream: RelayConnectionInterface) => {
        aliceAfterReconnect = newStream
        restarted.resolve()
      }
    )

    const aliceShakerBeforeReconnect = handshake({
      source: alice.source,
      sink: alice.sink
    })

    const relayShaker = handshake(RelayAlice)

    // try to read something
    aliceShakerBeforeReconnect.read()

    relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART))

    await restarted.promise

    aliceShakerBeforeReconnect.write(new TextEncoder().encode('Hello from Alice before reconnect'))

    assert(aliceAfterReconnect != undefined)

    const aliceShaker = handshake({
      sink: aliceAfterReconnect.sink,
      source: aliceAfterReconnect.source
    })

    const relayHelloAfterReconnect = new TextEncoder().encode('Hello after reconnect!')
    relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHelloAfterReconnect]))

    assert(u8aEquals(((await aliceShaker.read()) as Uint8Array).slice(), relayHelloAfterReconnect))

    const aliceHelloAfterReconnect = new TextEncoder().encode('Hello from Alice after reconnect!')

    aliceShaker.write(aliceHelloAfterReconnect)
    assert(
      u8aEquals(
        ((await relayShaker.read()) as Uint8Array).slice(),
        Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHelloAfterReconnect])
      )
    )
  })

  it('reconnect before using stream and use new stream', async function () {
    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    let aliceAfterReconnect: RelayConnectionInterface | undefined

    let restarted = defer<void>()

    const alice = RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async (newStream: RelayConnectionInterface) => {
        restarted.resolve()
        restarted = defer<void>()
        aliceAfterReconnect = newStream
      }
    )

    const aliceShakerBeforeReconnect = handshake({
      source: alice.source,
      sink: alice.sink
    })

    const relayShaker = handshake(RelayAlice)

    let aliceHelloBeforeReconnect = new TextEncoder().encode(`Hello from Alice before reconnecting`)
    aliceShakerBeforeReconnect.write(aliceHelloBeforeReconnect)

    assert(
      u8aEquals(
        ((await relayShaker.read()) as Uint8Array).slice(),
        Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHelloBeforeReconnect])
      )
    )

    let relayHelloBeforeReconnect = new TextEncoder().encode(`Hello from relay before reconnecting`)
    relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHelloBeforeReconnect]))

    assert(u8aEquals(((await aliceShakerBeforeReconnect.read()) as Uint8Array).slice(), relayHelloBeforeReconnect))

    const ATTEMPTS = 5
    for (let i = 0; i < ATTEMPTS; i++) {
      relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART))

      await restarted.promise

      aliceShakerBeforeReconnect.write(new TextEncoder().encode('Hello from Alice before reconnect'))

      assert(aliceAfterReconnect != undefined)

      const aliceShaker = handshake({
        sink: aliceAfterReconnect.sink,
        source: aliceAfterReconnect.source
      })

      const relayHelloAfterReconnect = new TextEncoder().encode('Hello after reconnect!')
      relayShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...relayHelloAfterReconnect]))

      assert(u8aEquals(((await aliceShaker.read()) as Uint8Array).slice(), relayHelloAfterReconnect))

      const aliceHelloAfterReconnect = new TextEncoder().encode('Hello from Alice after reconnect!')

      aliceShaker.write(aliceHelloAfterReconnect)
      assert(
        u8aEquals(
          ((await relayShaker.read()) as Uint8Array).slice(),
          Uint8Array.from([RelayPrefix.PAYLOAD, ...aliceHelloAfterReconnect])
        )
      )
    }
  })

  it('forward and prefix WebRTC messages', async function () {
    const webRTC = new WebRTC()

    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {
        getWebRTCUpgrader() {
          return {
            upgradeOutbound() {
              return webRTC
            },
            upgradeInbound() {
              return webRTC
            }
          }
        }
      } as ConnectComponents,
      {
        __noWebRTCUpgrade: false
      },
      async () => {}
    )

    const relayShaker = handshake(RelayAlice)

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

    assert(
      JSON.parse(new TextDecoder().decode(((await relayShaker.read()) as Uint8Array).slice(1))).message ===
        webRTCResponse
    )
  })

  it('forward and prefix WebRTC messages after reconnect', async function () {
    const webRTC = new WebRTC()

    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    let webRTCAfterReconnect: WebRTC | undefined
    let secondAttempt = false

    let restarted = defer<void>()

    RelayConnection(
      AliceRelay,
      Relay,
      Bob,
      'outbound',
      () => {},
      {
        getWebRTCUpgrader() {
          return {
            upgradeOutbound() {
              if (!secondAttempt) {
                secondAttempt = true
                return webRTC
              } else {
                webRTCAfterReconnect ??= new WebRTC()
                return webRTCAfterReconnect
              }
            },
            upgradeInbound() {
              if (!secondAttempt) {
                secondAttempt = true
                return webRTC
              } else {
                webRTCAfterReconnect ??= new WebRTC()
                return webRTCAfterReconnect
              }
            }
          }
        }
      } as ConnectComponents,
      {
        __noWebRTCUpgrade: false
      },
      async () => {
        restarted.resolve()
        restarted = defer<void>()
      }
    )

    const relayShaker = handshake(RelayAlice)

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

    assert(
      JSON.parse(new TextDecoder().decode(((await relayShaker.read()) as Uint8Array).slice(1))).message ===
        webRTCResponse
    )

    const ATTEMPTS = 5
    for (let i = 0; i < ATTEMPTS; i++) {
      relayShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART))

      await restarted.promise

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
        JSON.parse(new TextDecoder().decode(((await relayShaker.read()) as Uint8Array).slice(1))).message ===
          correctWebRTCResponseAfterReconnect
      )
    }
  })
})

describe('relay connection - stream error propagation', function () {
  const Relay: PeerId = createPeerId()
  const Bob: PeerId = createPeerId()

  it('falsy sources in sinks', async function () {
    const [AliceRelay, RelayAlice] = duplexPair<StreamType>()

    const errorInSource = 'error in source'

    const alice = RelayConnection(
      RelayAlice,
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    )

    const sourcePromise = (AliceRelay.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]().next()

    await assert.rejects(
      alice.sink(
        (async function* () {
          throw Error(errorInSource)
        })()
      ),
      Error(errorInSource)
    )

    await sourcePromise
  })

  it('correct sources in falsy sinks', async function () {
    const RelayAlice = pair<StreamType>()

    const errorInSinkFunction = 'error in sink function'

    const alice = RelayConnection(
      {
        sink: () => Promise.reject(Error(errorInSinkFunction)),
        source: RelayAlice.source
      },
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    )

    // Should log exception but not throw it
    alice.sink(
      (async function* () {
        yield new Uint8Array()
      })()
    )
  })

  it('falsy sources', async function () {
    const AliceRelay = pair<StreamType>()

    const errorInSource = 'error in source'

    const alice = RelayConnection(
      {
        sink: AliceRelay.sink,
        source: (async function* () {
          throw Error(errorInSource)
        })()
      },
      Relay,
      Bob,
      'outbound',
      () => {},
      {} as ConnectComponents,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    )

    await assert.rejects(
      (alice.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]().next(),
      Error(errorInSource)
    )
  })
})
