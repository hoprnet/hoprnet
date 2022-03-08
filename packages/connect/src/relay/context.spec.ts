import { RelayContext, DEFAULT_PING_TIMEOUT } from './context'
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants'
import { u8aEquals, defer } from '@hoprnet/hopr-utils'
import Pair from 'it-pair'
import DuplexPair from 'it-pair/duplex'
import handshake from 'it-handshake'

import type { StreamType } from '../types'
import assert from 'assert'

describe('relay swtich context', function () {
  it('forward payload messages', async function () {
    const [relayToNode, nodeToRelay] = DuplexPair<StreamType>()

    const ctx = new RelayContext(nodeToRelay)

    const nodeShaker = handshake(relayToNode)
    const destinationShaker = handshake(ctx)

    const firstMessage = new TextEncoder().encode('first message')
    nodeShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage]))

    assert(u8aEquals((await destinationShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage])))

    const secondMessage = new TextEncoder().encode('second message')
    destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage]))

    assert(u8aEquals((await nodeShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage])))
  })

  it('ping comes back in time', async function () {
    const [relayToNode, nodeToRelay] = DuplexPair<StreamType>()

    const ctx = new RelayContext(nodeToRelay)

    const nodeShaker = handshake(relayToNode)

    const pingPromise = ctx.ping()

    assert(u8aEquals((await nodeShaker.read()).slice(), Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING)))
    nodeShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))

    const pingResponse = await pingPromise

    assert(pingResponse >= 0 && pingResponse <= DEFAULT_PING_TIMEOUT)
  })

  it('ping timeout', async function () {
    this.timeout(DEFAULT_PING_TIMEOUT + 2e3)

    const [relayToNode, nodeToRelay] = DuplexPair<StreamType>()

    const ctx = new RelayContext(nodeToRelay)

    const nodeShaker = handshake(relayToNode)

    const pingPromise = ctx.ping()

    assert(u8aEquals((await nodeShaker.read()).slice(), Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING)))

    const pingResponse = await pingPromise

    nodeShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))

    assert(pingResponse == -1)

    // Let async operations happen
    await new Promise((resolve) => setTimeout(resolve))

    const secondPingResult = await ctx.ping()

    assert(secondPingResult == -1)
  })

  it('stop a stream', async function () {
    const [relayToNode, nodeToRelay] = DuplexPair<StreamType>()

    const ctx = new RelayContext(nodeToRelay)

    const nodeShaker = handshake(relayToNode)

    nodeShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

    let msgReceived = false
    for await (const msg of ctx.source) {
      if (msgReceived) {
        assert.fail(`Stream must end after STOP message`)
      }
      assert(u8aEquals(msg.slice(), Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)))
      msgReceived = true
    }

    assert(msgReceived, `Other end must receive message`)
  })

  it('update stream', async function () {
    const [relayToNode, nodeToRelay] = DuplexPair<StreamType>()

    const ctx = new RelayContext(nodeToRelay)

    const nodeShaker = handshake(relayToNode)
    const destinationShaker = handshake(ctx)

    // Sending messages should work before stream switching
    const firstMessage = new TextEncoder().encode('first message')
    nodeShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage]))

    assert(u8aEquals((await destinationShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage])))

    const secondMessage = new TextEncoder().encode('second message')
    destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage]))

    assert(u8aEquals((await nodeShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage])))

    // Try to read something
    nodeShaker.read()

    const UPDATE_ATTEMPTS = 5

    for (let i = 0; i < UPDATE_ATTEMPTS; i++) {
      const [relayToNodeAfterUpdate, nodeToRelayAfterUpdate] = DuplexPair<StreamType>()

      ctx.update(nodeToRelayAfterUpdate)

      const nodeShakerAfterUpdate = handshake(relayToNodeAfterUpdate)

      const firstMessageAfterUpdate = new TextEncoder().encode('first message after update')
      nodeShakerAfterUpdate.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessageAfterUpdate]))

      assert(
        u8aEquals(
          (await destinationShaker.read()).slice(),
          Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)
        )
      )

      assert(
        u8aEquals(
          (await destinationShaker.read()).slice(),
          Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessageAfterUpdate])
        )
      )

      await new Promise((resolve) => setTimeout(resolve))
      const secondMessageAfterUpdate = new TextEncoder().encode('second message after update')
      destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessageAfterUpdate]))

      assert(
        u8aEquals(
          (await nodeShakerAfterUpdate.read()).slice(),
          Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessageAfterUpdate])
        )
      )
    }
  })
})

describe('relay switch context - falsy streams', function () {
  it('falsy sink source', async function () {
    const [relayToNode, nodeToRelay] = DuplexPair<StreamType>()

    const errorInSource = 'error in source'
    const ctx = new RelayContext(nodeToRelay)

    const sinkPromise = ctx.sink(
      (async function* () {
        throw Error(errorInSource)
      })()
    )

    const sourcePromise = (relayToNode.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]().next()

    await assert.rejects(sinkPromise, Error(errorInSource))

    await sourcePromise
  })

  it('falsy sink', async function () {
    const nodeToRelay = Pair<StreamType>()

    const falsySinkError = 'falsy sink error'

    const ctx = new RelayContext({
      source: nodeToRelay.source,
      sink: () => Promise.reject(Error(falsySinkError))
    })

    await assert.rejects(
      ctx.sink(
        (async function* () {
          yield new Uint8Array()
        })()
      ),
      Error(falsySinkError)
    )
  })

  it('falsy sink before attaching source', async function () {
    const nodeToRelay = Pair<StreamType>()

    const falsySinkError = 'falsy sink error'

    const waitForError = defer<void>()

    new RelayContext({
      source: nodeToRelay.source,
      sink: () => {
        waitForError.resolve()
        return Promise.reject(Error(falsySinkError))
      }
    })

    await waitForError.promise
    await new Promise((resolve) => setTimeout(resolve))
  })

  it('falsy sink', async function () {
    const relayToNode = Pair<StreamType>()

    const falsySourceError = 'falsy source error'

    const ctx = new RelayContext({
      source: (async function* () {
        throw new Error(falsySourceError)
      })(),
      sink: relayToNode.sink
    })

    await assert.rejects(
      (ctx.source as AsyncIterable<StreamType>)[Symbol.asyncIterator]().next(),
      Error(falsySourceError)
    )
  })
})
