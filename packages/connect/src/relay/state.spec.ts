import handshake from 'it-handshake'
import Pair from 'it-pair'

import type { StreamType } from '../types'

import assert from 'assert'
import { RelayState } from './state'
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants'
import { u8aEquals, privKeyToPeerId } from '@hoprnet/hopr-utils'

const initiator = privKeyToPeerId('0x9feb47f140eb4ebc8b233214451dd097240699f50728a2cdc290643c2f71eb98')
const relay = privKeyToPeerId('0x56f3a30e2736cf964dee9a2fa9575a59361b6be368bb7a52955dabd88327b983')
const destination = privKeyToPeerId('0x7fb0147c1872c39818c88a3b08e93f314ce826138f2330d22ca0e24c33ff5a0c')

describe('relay state management', function () {
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
        source: initiatorToRelay.source as any,
        sink: relayToInitiator.sink
      },
      {
        source: destinationToRelay.source as any,
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
      source: initiatorToRelayAfterUpdate.source as any,
      sink: relayToInitiatorAfterUpdate.sink
    })

    const initiatorShakerAfterUpdate = handshake({
      source: relayToInitiatorAfterUpdate.source as any,
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
        source: initiatorToRelay.source as any,
        sink: relayToInitiator.sink
      },
      {
        source: destinationToRelay.source as any,
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

describe('relay state management - errors', function () {
  it('new stream throws synchronously', async function () {
    const state = new RelayState()

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')
    await assert.rejects(
      async () =>
        await state.createNew(
          initiator,
          destination,
          {
            source: (async function* () {})(),
            sink: async (_source: any) => {
              throw Error(`boom`)
            }
          },
          {
            source: (async function* () {})(),
            sink: async (_source: any) => {
              throw Error(`boom`)
            }
          }
        ),
      Error(`boom`)
    )

    assert(!state.exists(initiator, destination))
  })

  it('new stream throws asynchronously', async function () {
    const state = new RelayState()

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')
    await assert.rejects(
      async () =>
        await state.createNew(
          initiator,
          destination,
          {
            source: (async function* () {})(),
            sink: async (_source: any) => Promise.reject(Error(`boom`))
          },
          {
            source: (async function* () {})(),
            sink: async (_source: any) => Promise.reject(Error(`boom`))
          }
        ),
      Error(`boom`)
    )

    assert(!state.exists(initiator, destination))
  })
})
