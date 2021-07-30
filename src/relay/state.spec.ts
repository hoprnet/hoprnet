/// <reference path="../@types/it-handshake.ts" />
/// <reference path="../@types/it-pair.ts" />

import handshake from 'it-handshake'
import Pair from 'it-pair'

import type { StreamType } from '../types'

import assert from 'assert'
import PeerId from 'peer-id'
import { RelayState } from './state'
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants'
import { u8aEquals } from '@hoprnet/hopr-utils'

describe('relay state management', function () {
  let initiator: PeerId, relay: PeerId, destination: PeerId

  before(async function () {
    ;[initiator, relay, destination] = await Promise.all(
      Array.from({ length: 3 }, (_) => PeerId.create({ keyType: 'secp256k1' }))
    )
  })

  it('identifier generation', function () {
    assert(RelayState.getId(initiator, relay) === RelayState.getId(relay, initiator))

    assert(RelayState.getId(initiator, relay) !== RelayState.getId(relay, destination))

    assert.throws(() => RelayState.getId(initiator, initiator))
  })

  it('check if active, create new and exchange messages', async function () {
    const state = new RelayState()

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')

    const initiatorToRelay = Pair<StreamType>()
    const relayToInitiator = Pair<StreamType>()

    const destinationToRelay = Pair<StreamType>()
    const relayToDestination = Pair<StreamType>()

    const initiatorShaker = handshake({
      source: relayToInitiator.source,
      sink: initiatorToRelay.sink
    })

    const destinationShaker = handshake({
      source: relayToDestination.source,
      sink: destinationToRelay.sink
    })

    state.createNew(
      initiator,
      destination,
      {
        source: initiatorToRelay.source,
        sink: relayToInitiator.sink
      },
      {
        source: destinationToRelay.source,
        sink: relayToDestination.sink
      }
    )

    const destinationIsActivePromise = state.isActive(initiator, destination)

    assert(
      u8aEquals(
        (await destinationShaker.read()).slice(),
        Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING)
      )
    )

    destinationShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))

    assert((await destinationIsActivePromise) === true, `link to destination must be active`)

    const initiatorIsActivePromise = state.isActive(destination, initiator)

    assert(
      u8aEquals((await initiatorShaker.read()).slice(), Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING))
    )

    initiatorShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))

    assert((await initiatorIsActivePromise) === true, `link to initiator must be active`)

    // check that we can communicate
    const initiatorHello = new TextEncoder().encode('Hello!')
    initiatorShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...initiatorHello]))

    assert(
      u8aEquals((await destinationShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...initiatorHello]))
    )

    const destinationHello = new TextEncoder().encode('Hello from the other side!')
    destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...destinationHello]))

    assert(
      u8aEquals((await initiatorShaker.read()).slice(), Uint8Array.from([RelayPrefix.PAYLOAD, ...destinationHello]))
    )

    const initiatorToRelayAfterUpdate = Pair<StreamType>()
    const relayToInitiatorAfterUpdate = Pair<StreamType>()

    state.updateExisting(initiator, destination, {
      source: initiatorToRelayAfterUpdate.source,
      sink: relayToInitiatorAfterUpdate.sink
    })

    const initiatorShakerAfterUpdate = handshake({
      source: relayToInitiatorAfterUpdate.source,
      sink: initiatorToRelayAfterUpdate.sink
    })

    const initiatorHelloAfterUpdate = new TextEncoder().encode(`Hello, I'm back!`)
    initiatorShakerAfterUpdate.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...initiatorHelloAfterUpdate]))

    assert(
      u8aEquals(
        (await destinationShaker.read()).slice(),
        Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)
      )
    )

    assert(
      u8aEquals(
        (await destinationShaker.read()).slice(),
        Uint8Array.from([RelayPrefix.PAYLOAD, ...initiatorHelloAfterUpdate])
      )
    )
  })

  it('check cleanup', async function () {
    const state = new RelayState()

    assert(!state.exists(initiator, destination))
    const initiatorToRelay = Pair<StreamType>()
    const relayToInitiator = Pair<StreamType>()

    const destinationToRelay = Pair<StreamType>()
    const relayToDestination = Pair<StreamType>()

    const initiatorShaker = handshake({
      source: relayToInitiator.source,
      sink: initiatorToRelay.sink
    })

    const destinationShaker = handshake({
      source: relayToDestination.source,
      sink: destinationToRelay.sink
    })

    state.createNew(
      initiator,
      destination,
      {
        source: initiatorToRelay.source,
        sink: relayToInitiator.sink
      },
      {
        source: destinationToRelay.source,
        sink: relayToDestination.sink
      }
    )

    initiatorShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

    assert(
      u8aEquals(
        (await destinationShaker.read()).slice(),
        Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
      )
    )

    // Let I/O operations happen
    await new Promise((resolve) => setTimeout(resolve))

    assert(!state.exists(initiator, destination))
  })
})
