import { handshake } from 'it-handshake'
import { duplexPair } from 'it-pair/duplex'

import type { StreamType } from '../types.js'

import assert from 'assert'
import { RelayState } from './state.js'
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants.js'
import { u8aEquals, privKeyToPeerId } from '@hoprnet/hopr-utils'
import { pipe } from 'it-pipe'

const initiator = privKeyToPeerId('0x9feb47f140eb4ebc8b233214451dd097240699f50728a2cdc290643c2f71eb98')
const relay = privKeyToPeerId('0x56f3a30e2736cf964dee9a2fa9575a59361b6be368bb7a52955dabd88327b983')
const destination = privKeyToPeerId('0x7fb0147c1872c39818c88a3b08e93f314ce826138f2330d22ca0e24c33ff5a0c')

function getPingResponder() {
  const [atob, btoa] = duplexPair<StreamType>()

  pipe(
    atob.source,
    // @ts-ignore stream type clash
    function reply(source: AsyncIterable<Uint8Array>) {
      return (async function* (): AsyncIterable<Uint8Array> {
        for await (const msg of source) {
          const [PREFIX, SUFFIX] = [msg.slice(0, 1), msg.slice(1)]

          switch (PREFIX[0]) {
            case RelayPrefix.STATUS_MESSAGE:
              if (SUFFIX[0] == StatusMessages.PING) {
                yield Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG)
              } else {
                yield msg
              }
              break
            case RelayPrefix.CONNECTION_STATUS:
              if (SUFFIX[0] == ConnectionStatusMessages.RESTART) {
                continue
              } else {
                yield msg
              }
              break
            default:
              yield msg
          }
        }
      })()
    },
    atob.sink
  )

  return btoa
}

describe('relay state management', function () {
  it('identifier generation', function () {
    assert(RelayState.getId(initiator, relay) === `${initiator.toString()}-${relay.toString()}`)
    assert(RelayState.getId(relay, destination) === `${relay.toString()}-${destination.toString()}`)

    assert(RelayState.getId(initiator, relay) === RelayState.getId(relay, initiator))

    assert(RelayState.getId(initiator, relay) !== RelayState.getId(relay, destination))

    assert.throws(() => RelayState.getId(initiator, initiator))
  })

  it.only('check if active, create new and exchange messages', async function () {
    const state = new RelayState({
      relayFreeTimeout: 1
    })

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')

    const [initiatorToRelay, relayToInitiator] = duplexPair<StreamType>()
    const [destinationToRelay, relayToDestination] = duplexPair<StreamType>()

    const initiatorShaker = handshake(initiatorToRelay)
    const destinationShaker = handshake(destinationToRelay)

    state.createNew(initiator, destination, relayToInitiator, relayToDestination)

    for (let i = 0; i < 3; i++) {
      const destinationIsActivePromise = state.isActive(initiator, destination)

      assert(
        u8aEquals(
          ((await destinationShaker.read()) as Uint8Array).slice(),
          Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING)
        )
      )

      destinationShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))

      assert((await destinationIsActivePromise) === true, `link to destination must be active`)

      const initiatorIsActivePromise = state.isActive(destination, initiator)

      assert(
        u8aEquals(
          ((await initiatorShaker.read()) as Uint8Array).slice(),
          Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING)
        )
      )

      initiatorShaker.write(Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG))

      assert((await initiatorIsActivePromise) === true, `link to initiator must be active`)
    }

    for (let i = 0; i < 5; i++) {
      // check that we can communicate
      const initiatorHello = new TextEncoder().encode('Hello!')
      initiatorShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...initiatorHello]))

      assert(
        u8aEquals(
          ((await destinationShaker.read()) as Uint8Array).slice(),
          Uint8Array.from([RelayPrefix.PAYLOAD, ...initiatorHello])
        )
      )

      const destinationHello = new TextEncoder().encode('Hello from the other side!')
      destinationShaker.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...destinationHello]))

      assert(
        u8aEquals(
          ((await initiatorShaker.read()) as Uint8Array).slice(),
          Uint8Array.from([RelayPrefix.PAYLOAD, ...destinationHello])
        )
      )
    }

    for (let i = 0; i < 5; i++) {
      const [initiatorToRelayAfterUpdate, relayToInitiatorAfterUpdate] = duplexPair<StreamType>()

      state.updateExisting(initiator, destination, initiatorToRelayAfterUpdate)

      const initiatorShakerAfterUpdate = handshake(relayToInitiatorAfterUpdate)

      await new Promise((resolve) => setTimeout(resolve, 100))

      const initiatorHelloAfterUpdate = new TextEncoder().encode(`Hello, I'm back!`)
      initiatorShakerAfterUpdate.write(Uint8Array.from([RelayPrefix.PAYLOAD, ...initiatorHelloAfterUpdate]))

      assert(
        u8aEquals(
          ((await destinationShaker.read()) as Uint8Array).slice(),
          Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.RESTART)
        )
      )
    }
  })

  it('check cleanup', async function () {
    const state = new RelayState({
      relayFreeTimeout: 1
    })

    assert(!state.exists(initiator, destination))
    const [initiatorToRelay, relayToInitiator] = duplexPair<StreamType>()

    const [destinationToRelay, relayToDestination] = duplexPair<StreamType>()

    const initiatorShaker = handshake(initiatorToRelay)

    const destinationShaker = handshake(relayToDestination)

    state.createNew(initiator, destination, relayToInitiator, destinationToRelay)

    initiatorShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

    assert(
      u8aEquals(
        ((await destinationShaker.read()) as Uint8Array).slice(),
        Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
      )
    )

    // Let I/O operations happen
    await new Promise((resolve) => setTimeout(resolve, 100))

    assert(!state.exists(initiator, destination))
  })
})

describe('relay state management - errors', function () {
  it('new stream throws synchronously - source ended', async function () {
    const state = new RelayState({
      relayFreeTimeout: 1
    })

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')

    state.createNew(
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
    )

    assert((await state.isActive(initiator, destination)) === false)

    state.updateExisting(destination, initiator, getPingResponder())
    state.updateExisting(initiator, destination, getPingResponder())

    // Propagation delay
    await new Promise((resolve) => setTimeout(resolve, 100))

    assert((await state.isActive(initiator, destination)) === true)
    assert((await state.isActive(destination, initiator)) === true)
  })

  it('new stream throws synchronously - source undefined', async function () {
    const state = new RelayState({
      relayFreeTimeout: 1
    })

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')

    state.createNew(
      initiator,
      destination,
      {
        source: undefined as any,
        sink: async (_source: any) => {
          throw Error(`boom`)
        }
      },
      {
        source: undefined as any,
        sink: async (_source: any) => {
          throw Error(`boom`)
        }
      }
    )

    assert((await state.isActive(initiator, destination)) === false)

    state.updateExisting(destination, initiator, getPingResponder())
    state.updateExisting(initiator, destination, getPingResponder())

    // Propagation delay
    await new Promise((resolve) => setTimeout(resolve, 100))

    assert((await state.isActive(initiator, destination)) === true)
    assert((await state.isActive(destination, initiator)) === true)
  })

  it('new stream throws asynchronously', async function () {
    const state = new RelayState({
      relayFreeTimeout: 1
    })

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')

    state.createNew(
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
    )

    assert((await state.isActive(initiator, destination)) === false)

    state.updateExisting(destination, initiator, getPingResponder())
    state.updateExisting(initiator, destination, getPingResponder())

    // Propagation delay
    await new Promise((resolve) => setTimeout(resolve, 100))

    assert((await state.isActive(initiator, destination)) === true)
    assert((await state.isActive(destination, initiator)) === true)
  })
})
