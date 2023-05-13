import { handshake } from 'it-handshake'
import { duplexPair } from 'it-pair/duplex'

import type { StreamType } from '../types.js'

import assert from 'assert'
import { RelayConnection } from './connection.js'
import { WebRTCConnection } from '../webrtc/connection.js'
import { WebRTCUpgrader } from '../webrtc/upgrader.js'
import { IStream, RelayState, connect_relay_set_panic_hook } from '../../lib/connect_relay.js'
connect_relay_set_panic_hook()
import { ConnectionStatusMessages, RelayPrefix, StatusMessages } from '../constants.js'
import { u8aEquals, privKeyToPeerId } from '@hoprnet/hopr-utils'
import { setTimeout } from 'timers/promises'
// import { pipe } from 'it-pipe'

const initiator = privKeyToPeerId('0x9feb47f140eb4ebc8b233214451dd097240699f50728a2cdc290643c2f71eb98')
// const relay = privKeyToPeerId('0x56f3a30e2736cf964dee9a2fa9575a59361b6be368bb7a52955dabd88327b983')
const destination = privKeyToPeerId('0x7fb0147c1872c39818c88a3b08e93f314ce826138f2330d22ca0e24c33ff5a0c')

// function getPingResponder() {
//   const [atob, btoa] = duplexPair<StreamType>()

//   pipe(
//     atob.source,
//     // @ts-ignore stream type clash
//     function reply(source: AsyncIterable<Uint8Array>) {
//       return (async function* (): AsyncIterable<Uint8Array> {
//         for await (const msg of source) {
//           const [PREFIX, SUFFIX] = [msg.slice(0, 1), msg.slice(1)]

//           switch (PREFIX[0]) {
//             case RelayPrefix.STATUS_MESSAGE:
//               if (SUFFIX[0] == StatusMessages.PING) {
//                 yield Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PONG)
//               } else {
//                 yield msg
//               }
//               break
//             case RelayPrefix.CONNECTION_STATUS:
//               if (SUFFIX[0] == ConnectionStatusMessages.RESTART) {
//                 continue
//               } else {
//                 yield msg
//               }
//               break
//             default:
//               yield msg
//           }
//         }
//       })()
//     },
//     atob.sink
//   )

//   return btoa
// }

describe('relay state management', function () {
  //   it('identifier generation', function () {
  //     assert(RelayState.getId(initiator, relay) === `${initiator.toString()}-${relay.toString()}`)
  //     assert(RelayState.getId(relay, destination) === `${relay.toString()}-${destination.toString()}`)

  //     assert(RelayState.getId(initiator, relay) === RelayState.getId(relay, initiator))

  //     assert(RelayState.getId(initiator, relay) !== RelayState.getId(relay, destination))

  //     assert.throws(() => RelayState.getId(initiator, initiator))
  //   })

  it('check if active, create new and exchange messages', async function () {
    const state = new RelayState({
      relayFreeTimeout: 1
    })

    assert(!state.exists(initiator, destination))

    assert(!(await state.isActive(initiator, destination)), 'empty state must not be active')

    const [initiatorToRelay, relayToInitiator] = duplexPair<StreamType>()
    const [destinationToRelay, relayToDestination] = duplexPair<StreamType>()

    const initiatorShaker = handshake(initiatorToRelay)
    const destinationShaker = handshake(destinationToRelay)

    state.createNew(initiator, destination, relayToInitiator as IStream, relayToDestination as IStream)

    initiatorShaker.write(Uint8Array.of(RelayPrefix.PAYLOAD))
    destinationShaker.write(Uint8Array.of(RelayPrefix.PAYLOAD))

    await Promise.all([initiatorShaker.read(), destinationShaker.read()])
    console.log(`after message exchange`)

    for (let i = 0; i < 3; i++) {
      const destinationIsActivePromise = state.isActive(initiator, destination)

      assert(
        u8aEquals(
          ((await destinationShaker.read()) as Uint8Array).slice(),
          Uint8Array.of(RelayPrefix.STATUS_MESSAGE, StatusMessages.PING)
        )
      )

      console.log(`ping received`)

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

      state.updateExisting(initiator, destination, initiatorToRelayAfterUpdate as IStream)

      const initiatorShakerAfterUpdate = handshake(relayToInitiatorAfterUpdate)

      await setTimeout(100)

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

    state.createNew(initiator, destination, relayToInitiator as IStream, destinationToRelay as IStream)

    let stopPromiseDestination = destinationShaker.read()
    let stopPromiseInitiator = initiatorShaker.read()

    initiatorShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))
    destinationShaker.write(Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP))

    assert(
      u8aEquals(
        (await stopPromiseDestination) as Uint8Array,
        Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
      )
    )

    assert(
      u8aEquals(
        (await stopPromiseInitiator) as Uint8Array,
        Uint8Array.of(RelayPrefix.CONNECTION_STATUS, ConnectionStatusMessages.STOP)
      )
    )

    // Let I/O operations happen
    await setTimeout(100)

    for await (const _msg of destinationShaker.stream.source) {
      // must close the stream
    }

    for await (const _msg of initiatorShaker.stream.source) {
      // must close the stream
    }

    assert(!state.exists(initiator, destination))
  })
})

it('workflow', async function () {
  const state = new RelayState({
    relayFreeTimeout: 1
  })

  const [a_frontend, a_backend] = duplexPair<StreamType>()
  const [b_frontend, b_backend] = duplexPair<StreamType>()

  let a = RelayConnection(
    a_frontend,
    initiator,
    destination,
    'outbound',
    () => {},
    undefined as any,
    {
      __noWebRTCUpgrade: true
    },
    async () => {}
  )

  let b = RelayConnection(
    b_frontend,
    destination,
    initiator,
    'inbound',
    () => {},
    undefined as any,
    {
      __noWebRTCUpgrade: true
    },
    async () => {}
  )

  state.createNew(initiator, destination, a_backend as IStream, b_backend as IStream)

  assert(await state.isActive(initiator, destination))

  let a_handshake = handshake(a)
  let b_handshake = handshake(b)

  for (let i = 0; i < 23; i++) {
    await setTimeout(20)
    a_handshake.write(Uint8Array.of(2 * i))
    b_handshake.write(Uint8Array.of(2 * i + 1))

    assert(u8aEquals((await a_handshake.read()) as Uint8Array, Uint8Array.of(2 * i + 1)))
    assert(u8aEquals((await b_handshake.read()) as Uint8Array, Uint8Array.of(2 * i)))
  }
})

it('webrtc workflow', async function () {
  const state = new RelayState({
    relayFreeTimeout: 1
  })

  const [a_frontend, a_backend] = duplexPair<StreamType>()
  const [b_frontend, b_backend] = duplexPair<StreamType>()

  let a = WebRTCConnection(
    RelayConnection(
      a_frontend,
      initiator,
      destination,
      'outbound',
      () => {},
      undefined as any,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    ),
    {},
    () => {}
  )

  let b = WebRTCConnection(
    RelayConnection(
      b_frontend,
      destination,
      initiator,
      'inbound',
      () => {},
      undefined as any,
      {
        __noWebRTCUpgrade: true
      },
      async () => {}
    ),
    {},
    () => {}
  )

  state.createNew(initiator, destination, a_backend as IStream, b_backend as IStream)

  assert(await state.isActive(initiator, destination))

  let a_handshake = handshake(a)
  let b_handshake = handshake(b)

  for (let i = 0; i < 23; i++) {
    await setTimeout(20)
    a_handshake.write(Uint8Array.of(2 * i))
    b_handshake.write(Uint8Array.of(2 * i + 1))

    assert(u8aEquals((await a_handshake.read()) as Uint8Array, Uint8Array.of(2 * i + 1)))
    assert(u8aEquals((await b_handshake.read()) as Uint8Array, Uint8Array.of(2 * i)))
  }
})

it('webrtc workflow', async function () {
  this.timeout(10e3)
  const state = new RelayState({
    relayFreeTimeout: 1
  })

  const [a_frontend, a_backend] = duplexPair<StreamType>()
  const [b_frontend, b_backend] = duplexPair<StreamType>()

  let a_upgrader = new WebRTCUpgrader({})
  let b_upgrader = new WebRTCUpgrader({})
  let a = WebRTCConnection(
    RelayConnection(
      a_frontend,
      initiator,
      destination,
      'outbound',
      () => {},
      {
        getWebRTCUpgrader: () => a_upgrader
      } as any,
      {},
      async () => {}
    ),
    {},
    () => {}
  )

  let b = WebRTCConnection(
    RelayConnection(
      b_frontend,
      destination,
      initiator,
      'inbound',
      () => {},
      {
        getWebRTCUpgrader: () => b_upgrader
      } as any,
      {},
      async () => {}
    ),
    {},
    () => {}
  )

  state.createNew(initiator, destination, a_backend as IStream, b_backend as IStream)

  let a_handshake = handshake(a)
  let b_handshake = handshake(b)

  assert(await state.isActive(initiator, destination))

  for (let i = 0; i < 4; i++) {
    await setTimeout(200)
    a_handshake.write(Uint8Array.of(2 * i))
    b_handshake.write(Uint8Array.of(2 * i + 1))

    assert(u8aEquals((await a_handshake.read()) as Uint8Array, Uint8Array.of(2 * i + 1)))
    assert(u8aEquals((await b_handshake.read()) as Uint8Array, Uint8Array.of(2 * i)))
  }
})
