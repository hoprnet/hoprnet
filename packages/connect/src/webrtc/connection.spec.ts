import handshake from 'it-handshake'
import Pair from 'it-pair'
import { Multiaddr } from 'multiaddr'

import { WebRTCConnection, MigrationStatus } from './connection'
import { encodeWithLengthPrefix } from '../utils'
import { privKeyToPeerId, stringToU8a, u8aEquals, defer } from '@hoprnet/hopr-utils'
import pushable from 'it-pushable'

import { EventEmitter } from 'events'
import assert from 'assert'
import type { StreamType } from '../types'
import chai, { expect } from 'chai'
import spies from 'chai-spies'

chai.use(spies)

// const Alice = privKeyToPeerId(stringToU8a(`0xf8860ccb336f4aad751f55765b4adbefc538f8560c21eed6fbc9940d0584eeca`))
const Bob = privKeyToPeerId(stringToU8a(`0xf8860ccb336f4aad751f55765b4adbefc538f8560c21eed6fbc9940d0584eeca`))

describe('test webrtc connection', function () {
  it('exchange messages without upgrade', async function () {
    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const fakedWebRTCInstance = new EventEmitter() as any

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const conn = new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: () => {}
      } as any,
      fakedWebRTCInstance
    )

    const AliceShaker = handshake<StreamType>(conn as any)
    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    const ATTEMPTS = 5

    for (let i = 0; i < ATTEMPTS; i++) {
      const firstMessage = new TextEncoder().encode(`first message`)
      AliceShaker.write(firstMessage)

      assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])))

      const secondMessage = new TextEncoder().encode(`second message`)
      BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

      assert(u8aEquals((await AliceShaker.read()).slice(), secondMessage))
    }
  })

  it('sends UPGRADED to the relayed connection', async function () {
    const sendUpgradedSpy = chai.spy()

    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const webRTCInstance = new EventEmitter()

    new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: sendUpgradedSpy
      } as any,
      webRTCInstance as any
    )

    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    webRTCInstance.emit(`connect`)

    assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.of(MigrationStatus.DONE)))

    expect(sendUpgradedSpy).to.have.been.called.once
  })

  it('send DONE after webRTC connect event', async function () {
    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const webRTCInstance = new EventEmitter()

    new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: () => {}
      } as any,
      webRTCInstance as any
    )

    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    webRTCInstance.emit(`connect`)

    assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.of(MigrationStatus.DONE)))
  })

  it('sending messages after webRTC error event', async function () {
    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const webRTCInstance = new EventEmitter()

    Object.assign(webRTCInstance, {
      destroy: () => {}
    })

    const conn = new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink
      } as any,
      webRTCInstance as any
    )

    const AliceShaker = handshake<StreamType>(conn as any)
    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    webRTCInstance.emit(`error`)

    const firstMessage = new TextEncoder().encode(`first message`)
    AliceShaker.write(firstMessage)

    assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])))

    const secondMessage = new TextEncoder().encode(`second message`)
    BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

    assert(u8aEquals((await AliceShaker.read()).slice(), secondMessage))
  })

  it('exchange messages and send DONE after webRTC connect event', async function () {
    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const webRTCInstance = new EventEmitter()

    const conn = new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: () => {}
      } as any,
      webRTCInstance as any
    )

    const AliceShaker = handshake<StreamType>(conn as any)
    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    const firstMessage = new TextEncoder().encode(`first message`)
    AliceShaker.write(firstMessage)

    assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])))

    const secondMessage = new TextEncoder().encode(`second message`)
    BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

    assert(u8aEquals((await AliceShaker.read()).slice(), secondMessage))

    webRTCInstance.emit(`connect`)

    assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.of(MigrationStatus.DONE)))
  })

  it('exchange messages through webRTC', async function () {
    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const BobAliceWebRTC = pushable<StreamType>()
    const AliceBobWebRTC = pushable<StreamType>()

    const webRTCInstance = new EventEmitter()

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
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        remoteAddr: new Multiaddr(`/p2p/${Bob.toB58String()}`),
        sendUpgraded: () => {}
      } as any,
      webRTCInstance as any
    )

    const AliceShaker = handshake<StreamType>(conn as any)
    const BobShaker = handshake({
      source: AliceBob.source,
      sink: BobAlice.sink
    })

    const firstMessage = new TextEncoder().encode(`first message`)
    AliceShaker.write(firstMessage)

    assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.from([MigrationStatus.NOT_DONE, ...firstMessage])))

    const secondMessage = new TextEncoder().encode(`second message`)
    BobShaker.write(Uint8Array.from([MigrationStatus.NOT_DONE, ...secondMessage]))

    assert(u8aEquals((await AliceShaker.read()).slice(), secondMessage))

    webRTCInstance.emit(`connect`)

    assert(u8aEquals((await BobShaker.read()).slice(), Uint8Array.of(MigrationStatus.DONE)))

    BobShaker.write(Uint8Array.of(MigrationStatus.DONE))

    const msgSentThroughWebRTC = new TextEncoder().encode(`message that is sent through faked WebRTC`)
    BobAliceWebRTC.push(encodeWithLengthPrefix(Uint8Array.from([MigrationStatus.NOT_DONE, ...msgSentThroughWebRTC])))

    assert(u8aEquals((await AliceShaker.read()).slice(), msgSentThroughWebRTC))

    const msgSentBackThroughWebRTC = new TextEncoder().encode(`message that is sent back through faked WebRTC`)

    AliceShaker.write(msgSentBackThroughWebRTC)

    assert(
      u8aEquals(
        (await (AliceBobWebRTC as any).next()).value,
        encodeWithLengthPrefix(Uint8Array.from([MigrationStatus.NOT_DONE, ...msgSentBackThroughWebRTC]))
      )
    )
  })

  it('use abortController to end stream', async function () {
    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const webRTCInstance = new EventEmitter()

    Object.assign(webRTCInstance, {
      destroy: () => {}
    })

    const abort = new AbortController()

    const conn = new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink
      } as any,
      webRTCInstance as any,
      {
        signal: abort.signal
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
    const BobAlice = Pair<StreamType>()

    const falsySinkError = 'falsy sink error'

    const waitForSinkAttach = defer<Uint8Array>()

    const fakedWebRTCInstance = new EventEmitter() as any

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const conn = new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: () => waitForSinkAttach.promise,
        sendUpgraded: () => {}
      } as any,
      fakedWebRTCInstance
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
    const BobAlice = Pair<StreamType>()

    const falsySinkError = 'falsy sink error'

    const waitForError = defer<void>()

    const fakedWebRTCInstance = new EventEmitter() as any

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: () => {
          waitForError.resolve()
          return Promise.reject(Error(falsySinkError))
        },
        sendUpgraded: () => {}
      } as any,
      fakedWebRTCInstance
    )

    await waitForError.promise
    await new Promise((resolve) => setTimeout(resolve))
  })

  it('falsy sink source', async function () {
    const AliceBob = Pair<StreamType>()
    const BobAlice = Pair<StreamType>()

    const fakedWebRTCInstance = new EventEmitter() as any

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const errorInSinkSource = 'error in sink source'
    const conn = new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: BobAlice.source,
        sink: AliceBob.sink,
        sendUpgraded: () => {}
      } as any,
      fakedWebRTCInstance
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
    const AliceBob = Pair<StreamType>()

    const fakedWebRTCInstance = new EventEmitter() as any

    Object.assign(fakedWebRTCInstance, {
      destroy: () => {}
    })

    const errorInSource = 'error in source'
    const conn = new WebRTCConnection(
      Bob,
      { connections: new Map() } as any,
      {
        source: (async function* () {
          throw Error(errorInSource)
        })(),
        sink: AliceBob.sink,
        sendUpgraded: () => {}
      } as any,
      fakedWebRTCInstance
    )

    await assert.rejects(
      (conn.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]().next(),
      Error(errorInSource)
    )
  })
})
