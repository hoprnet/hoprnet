/// <reference path="./@types/it-pair.ts" />
/// <reference path="./@types/libp2p.ts" />

import { durations, u8aEquals } from '@hoprnet/hopr-utils'
import { RELAY_PAYLOAD_PREFIX } from './constants'
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

function createPeers(amount: number): Promise<PeerId[]> {
  return Promise.all(Array.from({ length: amount }, (_) => PeerId.create({ keyType: 'secp256k1' })))
}

describe('test overwritable connection', function () {
  let iteration = 0

  /**
   * Starts a duplex stream that sends numbered messages and checks
   * whether the received messages came from the same sender and in
   * expected order.
   * @param arg options
   */
  function getStream(arg: { usePrefix: boolean; designatedReceiverId?: number }): Stream {
    let _iteration = iteration

    //let lastId = -1
    let receiver = arg.designatedReceiverId

    return {
      source: (async function* () {
        let i = 0
        let msg: Uint8Array

        for (; i < 10; i++) {
          msg = new TextEncoder().encode(
            JSON.stringify({
              iteration: _iteration,
              messageId: i
            })
          )

          if (arg.usePrefix) {
            yield Uint8Array.from([...RELAY_PAYLOAD_PREFIX, ...msg])
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
            if (arg.usePrefix) {
              if (!u8aEquals(_msg.slice(0, 1), RELAY_PAYLOAD_PREFIX)) {
                continue
              }

              if (_msg.slice(1).length == 0) {
                continue
              }

              msg = _msg.slice(1)
            } else {
              msg = _msg.slice()
            }

            let decoded = JSON.parse(new TextDecoder().decode(msg)) as DebugMessage

            // assert(decoded.messageId == lastId + 1)

            if (receiver == null) {
              receiver = decoded.iteration
            } else {
              assert(receiver == decoded.iteration)
            }

            // lastId = decoded.messageId
          } else {
            assert.fail(`received empty message`)
          }
        }
      }
    }
  }

  it('should simulate a reconnect', async function () {
    this.timeout(durations.seconds(3))

    // Sample two IDs
    const [self, relay, counterparty] = await createPeers(3)

    // Get low-level connections between A, B
    const connectionA = Pair()
    const connectionB = Pair()

    let newSenderId = -1

    // Generate sender-side of relay
    const ctxRelaySelf = new RelayContext({
      sink: connectionA.sink,
      source: connectionB.source
    })

    // Generate counterparty-side of relay
    const ctxRelayCounterparty = new RelayContext(getStream({ usePrefix: true }))

    // Internally wiring relay
    ctxRelaySelf.sink(ctxRelayCounterparty.source)
    ctxRelayCounterparty.sink(ctxRelaySelf.source)

    // Getting a demo stream
    iteration++
    const receiverStream = getStream({ usePrefix: false })

    const ctx = new RelayConnection({
      stream: {
        sink: connectionB.sink,
        source: connectionA.source
      },
      self,
      relay,
      counterparty,
      onReconnect: async (newStream: RelayConnection) => {
        iteration++
        // console.log(newStream)
        const demoStream = getStream({ usePrefix: false, designatedReceiverId: newSenderId })

        newStream.sink(demoStream.source)
        demoStream.sink(newStream.source)
      }
    })

    ctx.sink(receiverStream.source)
    receiverStream.sink(ctx.source)

    let pingPromise: Promise<number>

    // Trigger a reconnect after a timeout
    setTimeout(async () => {
      iteration++
      await new Promise((resolve) => setTimeout(resolve, 100))
      pingPromise = ctxRelaySelf.ping()
      newSenderId = iteration
      ctxRelayCounterparty.update(getStream({ usePrefix: true }))
    }, 100)

    await new Promise((resolve) => setTimeout(resolve, 1000))

    // Make sure that we the ping went through
    // Note that `result == -1` means timeout
    console.log(`PING PROMISE`, await pingPromise!)
    // assert((await pingPromise!) >= 0)

    await ctx.close()
  })

  it('should perform a low-level ping', async function () {
    const [self, relay, counterparty] = await createPeers(3)

    // Get low-level connections between A, B
    const connectionA = Pair()
    const connectionB = Pair()
    const ctxClient = new RelayConnection({
      stream: {
        sink: connectionB.sink,
        source: connectionA.source
      },
      self,
      relay,
      counterparty,
      onReconnect: async () => {}
    })
    const ctxRelay = new RelayContext({
      sink: connectionA.sink,
      source: connectionB.source
    })

    const PING_ATTEMPTS = 4
    for (let i = 0; i < PING_ATTEMPTS; i++) {
      assert((await ctxRelay.ping()) >= 0, 'Ping must not timeout')
    }

    await ctxClient.close()

    assert(ctxClient.destroyed, `Connection must be destroyed`)
  })

  it('should echo messages', async function () {
    const [self, relay, counterparty] = await createPeers(3)

    // Get low-level connections between A, B
    const connectionA = Pair()
    const connectionB = Pair()
    const ctxClient = new RelayConnection({
      stream: {
        sink: connectionB.sink,
        source: connectionA.source
      },
      self,
      relay,
      counterparty,
      onReconnect: async () => {}
    })

    const ctxRelay = new RelayContext({
      sink: connectionA.sink,
      source: connectionB.source
    })

    const TEST_MESSAGES = ['first', 'second', 'third'].map((x) => new TextEncoder().encode(x))

    let messagesReceived = false

    ctxRelay.sink(ctxRelay.source)

    ctxClient.sink(
      (async function* () {
        yield* TEST_MESSAGES
      })()
    )

    let i = 0
    for await (const msg of ctxClient.source) {
      assert(u8aEquals(msg.slice(), TEST_MESSAGES[i]))

      if (i == TEST_MESSAGES.length - 1) {
        messagesReceived = true
      }
      i++
    }

    await ctxClient.close()

    assert(ctxClient.destroyed, `Connection must be destroyed`)

    assert(messagesReceived, `Messages must be received.`)
  })

  it('should exchange messages over a relay', async function () {
    const [self, relay, counterparty] = await createPeers(3)

    const connectionSelf = [Pair(), Pair()]
    const connectionCounterparty = [Pair(), Pair()]

    const ctxSelf = new RelayConnection({
      stream: {
        source: connectionSelf[0].source,
        sink: connectionSelf[1].sink
      },
      self,
      relay,
      counterparty,
      onReconnect: async () => {}
    })

    const ctxCounterparty = new RelayConnection({
      stream: {
        source: connectionCounterparty[0].source,
        sink: connectionCounterparty[1].sink
      },
      self: counterparty,
      relay,
      counterparty: self,
      onReconnect: async () => {}
    })

    const ctxRelaySelf = new RelayContext({
      sink: connectionSelf[0].sink,
      source: connectionSelf[1].source
    })

    const ctxRelayCounterparty = new RelayContext({
      sink: connectionCounterparty[0].sink,
      source: connectionCounterparty[1].source
    })

    assert((await ctxRelaySelf.ping()) >= 0)
    assert((await ctxRelayCounterparty.ping()) >= 0)

    ctxRelaySelf.sink(ctxRelayCounterparty.source)
    ctxRelayCounterparty.sink(ctxRelaySelf.source)

    assert((await ctxRelaySelf.ping()) >= 0)
    assert((await ctxRelayCounterparty.ping()) >= 0)

    const TEST_MESSAGES = ['first', 'second', 'third'].map((x) => new TextEncoder().encode(x))

    // Loopback messages
    ctxCounterparty.sink(ctxCounterparty.source)

    ctxSelf.sink(
      (async function* () {
        yield* TEST_MESSAGES
      })()
    )

    let messagesReceived = false
    let i = 0
    for await (const msg of ctxSelf.source) {
      assert(u8aEquals(msg.slice(), TEST_MESSAGES[i]))

      if (i == TEST_MESSAGES.length - 1) {
        messagesReceived = true
      }
      i++
    }

    assert((await ctxRelaySelf.ping()) >= 0)
    assert((await ctxRelayCounterparty.ping()) >= 0)

    await ctxSelf.close()
    await ctxCounterparty.close()

    assert(messagesReceived)
  })
})
