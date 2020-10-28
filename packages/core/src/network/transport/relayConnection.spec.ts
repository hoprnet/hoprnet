import { RelayConnection } from './relayConnection'
import assert from 'assert'
import { randomInteger } from '@hoprnet/hopr-utils'

import PeerId from 'peer-id'
import { EventEmitter } from 'events'
import { Instance as SimplePeer } from 'simple-peer'

interface PairType<T> {
  sink(source: AsyncIterable<T>): Promise<void>
  source: AsyncIterable<T>
}

const TIMEOUT_LOWER_BOUND = 450
const TIMEOUT_UPPER_BOUND = 650

// @ts-ignore
const Pair: <T>() => PairType<T> = require('it-pair')

describe('test relay connection', function () {
  it('should initiate a relayConnection and let the receiver close the connection prematurely', async function () {
    const AliceBob = Pair<Uint8Array>()
    const BobAlice = Pair<Uint8Array>()
    const Alice = await PeerId.create({ keyType: 'secp256k1' })
    const Bob = await PeerId.create({ keyType: 'secp256k1' })
    const a = new RelayConnection({
      stream: {
        sink: AliceBob.sink,
        source: BobAlice.source
      },
      self: Alice,
      counterparty: Bob
    })

    const b = new RelayConnection({
      stream: {
        sink: BobAlice.sink,
        source: AliceBob.source
      },
      self: Bob,
      counterparty: Alice
    })

    a.sink(
      (async function* () {
        let i = 0
        while (true) {
          yield new TextEncoder().encode(`message ${i++}`)
          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    setTimeout(() => setImmediate(() => b.close()), randomInteger(TIMEOUT_LOWER_BOUND, TIMEOUT_UPPER_BOUND))

    for await (const msg of b.source) {
      console.log(new TextDecoder().decode(msg.slice()))
    }

    for await (const _msg of a.source) {
      throw Error(`there should be no message`)
    }

    await new Promise((resolve) => setTimeout(resolve, 50))

    assert(
      (
        await Promise.all([
          // @ts-ignore
          a.source.next(),
          // @ts-ignore
          b.source.next()
        ])
      ).every(({ done }) => done),
      `Streams must have ended.`
    )
    assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
  })

  it('should initiate a relayConnection and close the connection by the sender prematurely', async function () {
    const AliceBob = Pair<Uint8Array>()
    const BobAlice = Pair<Uint8Array>()
    const Alice = await PeerId.create({ keyType: 'secp256k1' })
    const Bob = await PeerId.create({ keyType: 'secp256k1' })
    const a = new RelayConnection({
      stream: {
        sink: AliceBob.sink,
        source: BobAlice.source
      },
      self: Alice,
      counterparty: Bob
    })

    const b = new RelayConnection({
      stream: {
        sink: BobAlice.sink,
        source: AliceBob.source
      },
      self: Bob,
      counterparty: Alice
    })

    a.sink(
      (async function* () {
        let i = 0
        while (true) {
          yield new TextEncoder().encode(`message ${i++}`)

          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )
    setTimeout(() => setImmediate(() => a.close()), randomInteger(TIMEOUT_LOWER_BOUND, TIMEOUT_UPPER_BOUND))

    for await (const msg of b.source) {
      console.log(new TextDecoder().decode(msg.slice()))
    }

    for await (const _msg of a.source) {
      throw Error(`there should be no message`)
    }

    await new Promise((resolve) => setTimeout(resolve, 50))

    assert(
      // @ts-ignore
      (await Promise.all([a.source.next(), b.source.next()])).every(({ done }) => done),
      `Streams must have ended.`
    )
    assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
  })

  it('should initiate a relayConnection and exchange messages and destroy the connection after a random timeout', async function () {
    const AliceBob = Pair<Uint8Array>()
    const BobAlice = Pair<Uint8Array>()

    const Alice = await PeerId.create({ keyType: 'secp256k1' })
    const Bob = await PeerId.create({ keyType: 'secp256k1' })

    const FakeWebRTCAlice = new EventEmitter()
    // @ts-ignore
    FakeWebRTCAlice.signal = (msg: string) => console.log(`received fancy WebRTC message`, msg)

    const FakeWebRTCBob = new EventEmitter()
    // @ts-ignore
    FakeWebRTCBob.signal = (msg: string) => console.log(`received fancy WebRTC message`, msg)

    const interval = setInterval(() => FakeWebRTCAlice.emit(`signal`, { msg: 'Fake signal' }), 50)
    setTimeout(() => {
      clearInterval(interval)
      FakeWebRTCAlice.emit('connect')
    }, 200)

    const a = new RelayConnection({
      stream: {
        sink: AliceBob.sink,
        source: BobAlice.source
      },
      self: Alice,
      counterparty: Bob,
      webRTC: FakeWebRTCAlice as SimplePeer
    })

    const b = new RelayConnection({
      stream: {
        sink: BobAlice.sink,
        source: AliceBob.source
      },
      self: Bob,
      counterparty: Alice,
      webRTC: FakeWebRTCBob as SimplePeer
    })

    a.sink(
      (async function* () {
        let i = 0
        while (true) {
          yield new TextEncoder().encode(`message from a ${i++}`)

          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    b.sink(
      (async function* () {
        let i = 0
        await new Promise((resolve) => setTimeout(resolve, 50))

        while (true) {
          yield new TextEncoder().encode(`message from b ${i++}`)

          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    setTimeout(() => setImmediate(() => a.close()), randomInteger(TIMEOUT_LOWER_BOUND, TIMEOUT_UPPER_BOUND))

    let msgAReceived = false
    let msgBReceived = false

    let aDone = false
    let bDone = false

    function aFunction({ done, value }) {
      msgAReceived = true
      if (done) {
        aDone = true
      }
      return { done, value }
    }

    function bFunction({ done, value }) {
      msgBReceived = true
      if (done) {
        bDone = true
      }
      return { done, value }
    }

    // @ts-ignore
    let msgA = a.source.next().then(aFunction)

    // @ts-ignore
    let msgB = b.source.next().then(bFunction)

    while (true) {
      if (!aDone && !bDone) {
        await Promise.race([msgA, msgB])
      } else if (aDone) {
        await msgB
      } else if (bDone) {
        await msgA
      } else {
        break
      }

      if (msgAReceived || bDone) {
        msgAReceived = false

        if (aDone && bDone) {
          break
        } else {
          console.log(new TextDecoder().decode((await msgA).value))
        }

        //@ts-ignore
        msgA = a.source.next().then(aFunction)
      }

      if (msgBReceived || aDone) {
        msgBReceived = false

        if (aDone && bDone) {
          break
        } else {
          console.log(new TextDecoder().decode((await msgB).value))
        }
        //@ts-ignore
        msgB = b.source.next().then(bFunction)
      }
    }

    assert(
      // @ts-ignore
      (await Promise.all([a.source.next(), b.source.next()])).every(({ done }) => done),
      `both stream should have ended`
    )

    assert(b.destroyed && a.destroyed, `both parties must have marked the connection as destroyed`)
  })
})
