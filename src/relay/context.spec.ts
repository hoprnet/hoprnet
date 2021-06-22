/// <reference path="../@types/it-handshake.ts" />
/// <reference path="../@types/it-pair.ts" />
/// <reference path="../@types/libp2p.ts" />

import { RelayContext, DEFAULT_PING_TIMEOUT } from './context'
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants'
import { u8aEquals } from '@hoprnet/hopr-utils'
import Pair from 'it-pair'
import handshake from 'it-handshake'

import type { StreamType } from 'libp2p'
import assert from 'assert'

describe('relay swtich context', function () {
  it('forward payload messages', async function () {
    const nodeToRelay = Pair()
    const relayToNode = Pair()

    const ctx = new RelayContext({
      source: nodeToRelay.source,
      sink: relayToNode.sink
    })

    const nodeShaker = handshake<StreamType>({
      source: relayToNode.source,
      sink: nodeToRelay.sink
    })
    const destinationShaker = handshake<StreamType>(ctx)

    const firstMessage = new TextEncoder().encode('first message')
    nodeShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage]))

    assert(u8aEquals((await destinationShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage])))

    const secondMessage = new TextEncoder().encode('second message')
    destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage]))

    assert(u8aEquals((await nodeShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage])))
  })

  it('ping comes back in time', async function () {
    const nodeToRelay = Pair()
    const relayToNode = Pair()

    const ctx = new RelayContext({
      source: nodeToRelay.source,
      sink: relayToNode.sink
    })

    const nodeShaker = handshake<StreamType>({
      source: relayToNode.source,
      sink: nodeToRelay.sink
    })

    const pingPromise = ctx.ping()

    assert(u8aEquals((await nodeShaker.read()).slice(), Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING)))
    nodeShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))

    const pingResponse = await pingPromise

    assert(pingResponse >= 0 && pingResponse <= DEFAULT_PING_TIMEOUT)
  })

  it('ping timeout', async function () {
    this.timeout(DEFAULT_PING_TIMEOUT + 2e3)

    const nodeToRelay = Pair()
    const relayToNode = Pair()

    const ctx = new RelayContext({
      source: nodeToRelay.source,
      sink: relayToNode.sink
    })

    const nodeShaker = handshake<StreamType>({
      source: relayToNode.source,
      sink: nodeToRelay.sink
    })

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
    const nodeToRelay = Pair()
    const relayToNode = Pair()

    const ctx = new RelayContext({
      source: nodeToRelay.source,
      sink: relayToNode.sink
    })

    const nodeShaker = handshake<StreamType>({
      source: relayToNode.source,
      sink: nodeToRelay.sink
    })

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
    const nodeToRelay = Pair()
    const relayToNode = Pair()

    const ctx = new RelayContext({
      source: nodeToRelay.source,
      sink: relayToNode.sink
    })

    const nodeShaker = handshake<StreamType>({
      source: relayToNode.source,
      sink: nodeToRelay.sink
    })
    const destinationShaker = handshake<StreamType>(ctx)

    // Sending messages should work before stream switching
    const firstMessage = new TextEncoder().encode('first message')
    nodeShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage]))

    assert(u8aEquals((await destinationShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessage])))

    const secondMessage = new TextEncoder().encode('second message')
    destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage]))

    assert(u8aEquals((await nodeShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessage])))

    const nodeToRelayAfterUpdate = Pair()
    const relayToNodeAfterUpdate = Pair()

    // Try to read something
    nodeShaker.read()

    ctx.update({
      source: nodeToRelayAfterUpdate.source,
      sink: relayToNodeAfterUpdate.sink
    })

    const nodeShakerAfterUpdate = handshake<StreamType>({
      source: relayToNodeAfterUpdate.source,
      sink: nodeToRelayAfterUpdate.sink
    })

    const firstMessageAfterUpdate = new TextEncoder().encode('first message after update')
    nodeShakerAfterUpdate.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessageAfterUpdate]))

    assert(u8aEquals((await destinationShaker.read()).slice(), Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)))

    assert(u8aEquals((await destinationShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...firstMessageAfterUpdate])))

    await new Promise((resolve) => setTimeout(resolve))
    const secondMessageAfterUpdate = new TextEncoder().encode('second message after update')
    destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessageAfterUpdate]))

    assert(u8aEquals((await nodeShakerAfterUpdate.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...secondMessageAfterUpdate])))
  })
})
