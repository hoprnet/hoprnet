import type { StreamType } from '../types.js'
import type { Instance as SimplePeerInstance } from 'simple-peer'
import type { RelayConnection } from '../relay/connection.js'

import { handshake } from 'it-handshake'
import { pair } from 'it-pair'
import { duplexPair } from 'it-pair/duplex'
import { Multiaddr } from '@multiformats/multiaddr'
import { pushable } from 'it-pushable'

import { WebRTCConnection, MigrationStatus } from './connection.js'
import { encodeWithLengthPrefix } from '../utils/index.js'
import { privKeyToPeerId, stringToU8a, u8aEquals, defer } from '@hoprnet/hopr-utils'

import { EventEmitter } from 'events'
import assert from 'assert'

// const Alice = privKeyToPeerId(stringToU8a(`0xf8860ccb336f4aad751f55765b4adbefc538f8560c21eed6fbc9940d0584eeca`))
const Bob = privKeyToPeerId(stringToU8a(`0xf8860ccb336f4aad751f55765b4adbefc538f8560c21eed6fbc9940d0584eeca`))

describe('test webrtc connection', function () {
  it('exchange messages without upgrade', async function () {
    const AliceBob = pair<StreamType>()
    const BobAlice = pair<StreamType>()

    const fakedWebRTCInstance = new EventEmitter() as SimplePeerInstance

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const conn = new WebRTCConnection(
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: () => {},
        getWebRTCInstance: () => fakedWebRTCInstance
      } as RelayConnection,
      {}
    )

    const AliceShaker = handshake(conn)
    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    const ATTEMPTS = 5

    for (let i = 0; i < ATTEMPTS; i++) {
      const firstMessage = new TextEncoder().encode(`first message`)
      AliceShaker.write(firstMessage)

      assert(
        u8aEquals(
          ((await BobShaker.read()) as Uint8Array).slice(),
          Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])
        )
      )

      const secondMessage = new TextEncoder().encode(`second message`)
      BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

      assert(u8aEquals(((await AliceShaker.read()) as Uint8Array).slice(), secondMessage))
    }
  })

  it('sends UPGRADED to the relayed connection', async function () {
    const AliceBob = pair<StreamType>()
    const BobAlice = pair<StreamType>()

    const webRTCInstance = new EventEmitter() as SimplePeerInstance

    let upgradeCalls = 0
    new WebRTCConnection(
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: () => {
          upgradeCalls++
        },
        getWebRTCInstance: () => webRTCInstance
      } as RelayConnection,
      {}
    )

    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    webRTCInstance.emit(`connect`)

    assert(u8aEquals(((await BobShaker.read()) as Uint8Array).slice(), Uint8Array.of(MigrationStatus.DONE)))

    assert(upgradeCalls == 1)
  })

  it('send DONE after webRTC connect event', async function () {
    const AliceBob = pair<StreamType>()
    const BobAlice = pair<StreamType>()

    const webRTCInstance = new EventEmitter() as SimplePeerInstance

    new WebRTCConnection(
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: () => {},
        getWebRTCInstance: () => webRTCInstance
      } as RelayConnection,
      {}
    )

    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    webRTCInstance.emit(`connect`)

    assert(u8aEquals(((await BobShaker.read()) as Uint8Array).slice(), Uint8Array.of(MigrationStatus.DONE)))
  })

  it('sending messages after webRTC error event', async function () {
    const AliceBob = pair<StreamType>()
    const BobAlice = pair<StreamType>()

    const webRTCInstance = new EventEmitter() as SimplePeerInstance

    Object.assign(webRTCInstance, {
      destroy: () => {}
    })

    const conn = new WebRTCConnection(
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        getWebRTCInstance: () => webRTCInstance
      } as RelayConnection,
      {}
    )

    const AliceShaker = handshake(conn)
    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    webRTCInstance.emit(`error`)

    const firstMessage = new TextEncoder().encode(`first message`)
    AliceShaker.write(firstMessage)

    assert(
      u8aEquals(
        ((await BobShaker.read()) as Uint8Array).slice(),
        Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])
      )
    )

    const secondMessage = new TextEncoder().encode(`second message`)
    BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

    assert(u8aEquals(((await AliceShaker.read()) as Uint8Array).slice(), secondMessage))
  })

  it('exchange messages and send DONE after webRTC connect event', async function () {
    const [AliceBob, BobAlice] = duplexPair<StreamType>()

    const webRTCInstance = new EventEmitter() as SimplePeerInstance
    Object.assign(webRTCInstance, {
      _id: 'testing'
    })

    const conn = new WebRTCConnection(
      {
        ...AliceBob,
        sendUpgraded: () => {},
        getWebRTCInstance: () => webRTCInstance
      } as RelayConnection,
      {}
    )

    const AliceShaker = handshake(conn)
    const BobShaker = handshake(BobAlice)

    const firstMessage = new TextEncoder().encode(`first message`)
    AliceShaker.write(firstMessage)

    assert(
      u8aEquals(
        ((await BobShaker.read()) as Uint8Array).slice(),
        Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])
      )
    )

    const secondMessage = new TextEncoder().encode(`second message`)
    BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

    assert(u8aEquals(((await AliceShaker.read()) as Uint8Array).slice(), secondMessage))

    webRTCInstance.emit(`connect`)

    assert(u8aEquals(((await BobShaker.read()) as Uint8Array).slice(), Uint8Array.of(MigrationStatus.DONE)))
  })

  it('exchange messages through webRTC', async function () {
    const [AliceBob, BobAlice] = duplexPair<StreamType>()

    const BobAliceWebRTC = pushable()
    const AliceBobWebRTC = pushable()

    const webRTCInstance = new EventEmitter() as SimplePeerInstance

    // Turn faked WebRTC instance into an async iterator (read) and writable stream (write)
    Object.assign(webRTCInstance, {
      [Symbol.asyncIterator]() {
        return (async function* () {
          for await (const msg of BobAliceWebRTC) {
            yield msg
          }
        })()
      },
      write(msg: Uint8Array) {
        AliceBobWebRTC.push(msg)
      },
      writable: true,
      destroy() {
        AliceBobWebRTC.end()
      }
    })

    const conn = new WebRTCConnection(
      {
        ...AliceBob,
        remoteAddr: new Multiaddr(`/p2p/${Bob.toString()}`),
        sendUpgraded: () => {},
        getWebRTCInstance: () => webRTCInstance
      } as RelayConnection,
      {}
    )

    const AliceShaker = handshake(conn)
    const BobShaker = handshake(BobAlice)

    const firstMessage = new TextEncoder().encode(`first message`)
    AliceShaker.write(firstMessage)

    assert(
      u8aEquals(
        ((await BobShaker.read()) as Uint8Array).slice(),
        Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])
      )
    )

    const secondMessage = new TextEncoder().encode(`second message`)
    BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

    assert(u8aEquals(((await AliceShaker.read()) as Uint8Array).slice(), secondMessage))

    webRTCInstance.emit(`connect`)

    assert(u8aEquals(((await BobShaker.read()) as Uint8Array).slice(), Uint8Array.of(MigrationStatus.DONE)))

    BobShaker.write(Uint8Array.of(MigrationStatus.DONE))

    const msgSentThroughWebRTC = new TextEncoder().encode(`message that is sent through faked WebRTC`)
    BobAliceWebRTC.push(encodeWithLengthPrefix(Uint8Array.from([MigrationStatus.NOT_DONE, ...msgSentThroughWebRTC])))

    assert(u8aEquals(((await AliceShaker.read()) as Uint8Array).slice(), msgSentThroughWebRTC))

    const msgSentBackThroughWebRTC = new TextEncoder().encode(`message that is sent back through faked WebRTC`)

    AliceShaker.write(msgSentBackThroughWebRTC)

    assert(
      u8aEquals(
        (await AliceBobWebRTC[Symbol.asyncIterator]().next()).value,
        encodeWithLengthPrefix(Uint8Array.from([MigrationStatus.NOT_DONE, ...msgSentBackThroughWebRTC]))
      )
    )
  })

  it('use abortController to end stream', async function () {
    const [AliceBob, _BobAlice] = duplexPair<StreamType>()

    const webRTCInstance = new EventEmitter() as SimplePeerInstance

    Object.assign(webRTCInstance, {
      destroy: () => {}
    })

    const abort = new AbortController()

    const conn = new WebRTCConnection(
      {
        ...AliceBob,
        getWebRTCInstance: () => webRTCInstance
      } as RelayConnection,
      {},
      {
        signal: abort.signal,
        upgrader: undefined as any
      }
    )

    await assert.doesNotReject(
      async () =>
        await conn.sink(
          (async function* () {
            abort.abort()
            yield new TextEncoder().encode('dummy message')
          })()
        )
    )
  })
})

describe('webrtc connection - stream error propagation', function () {
  it('falsy sink', async function () {
    const BobAlice = pair<StreamType>()

    const falsySinkError = 'falsy sink error'

    const waitForSinkAttach = defer<void>()

    const fakedWebRTCInstance = new EventEmitter() as SimplePeerInstance

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const conn = new WebRTCConnection(
      {
        source: BobAlice.source,
        sink: (_source: AsyncIterable<Uint8Array>) => waitForSinkAttach.promise,
        sendUpgraded: () => {},
        getWebRTCInstance: () => fakedWebRTCInstance
      } as RelayConnection,
      {}
    )

    await assert.rejects(
      conn.sink(
        (async function* () {
          waitForSinkAttach.reject(Error(falsySinkError))
          yield new Uint8Array()
        })()
      ),
      Error(falsySinkError)
    )
  })

  it('falsy sink before sink source attach', async function () {
    const BobAlice = pair<StreamType>()

    const falsySinkError = 'falsy sink error'

    const waitForError = defer<void>()

    const fakedWebRTCInstance = new EventEmitter() as SimplePeerInstance

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    new WebRTCConnection(
      {
        source: BobAlice.source,
        sink: (_source: AsyncIterable<Uint8Array>) => {
          waitForError.resolve()
          return Promise.reject(Error(falsySinkError))
        },
        sendUpgraded: () => {},
        getWebRTCInstance: () => fakedWebRTCInstance
      } as RelayConnection,
      {}
    )

    await waitForError.promise
    await new Promise((resolve) => setTimeout(resolve))
  })

  it('falsy sink source', async function () {
    const [AliceBob, _BobAlice] = duplexPair<StreamType>()

    const fakedWebRTCInstance = new EventEmitter() as SimplePeerInstance

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const errorInSinkSource = 'error in sink source'
    const conn = new WebRTCConnection(
      {
        ...AliceBob,
        sendUpgraded: () => {},
        getWebRTCInstance: () => fakedWebRTCInstance
      } as RelayConnection,
      {}
    )

    await assert.rejects(
      conn.sink(
        (async function* () {
          throw Error(errorInSinkSource)
        })()
      ),
      Error(errorInSinkSource)
    )
  })

  it('falsy source', async function () {
    const AliceBob = pair<StreamType>()

    const fakedWebRTCInstance = new EventEmitter() as SimplePeerInstance

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const errorInSource = 'error in source'
    const conn = new WebRTCConnection(
      {
        source: (async function* () {
          throw Error(errorInSource)
        })() as AsyncIterable<Uint8Array>,
        sink: AliceBob.sink,
        sendUpgraded: () => {},
        getWebRTCInstance: () => fakedWebRTCInstance
      } as RelayConnection,
      {}
    )

    await assert.rejects(
      (conn.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]().next(),
      Error(errorInSource)
    )
  })
})
