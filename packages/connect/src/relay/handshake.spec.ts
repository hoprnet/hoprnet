import { RelayHandshake, RelayHandshakeMessage } from './handshake'
import { u8aEquals, defer, privKeyToPeerId } from '@hoprnet/hopr-utils'
import DuplexPair from 'it-pair/duplex'
import type PeerId from 'peer-id'
import assert from 'assert'
import type { Stream, StreamType } from '../types'

const initiator = privKeyToPeerId('0x695a1ad048d12a1a82f827a38815ab33aa4464194fa0bdb99f78d9c66ec21505')
const relay = privKeyToPeerId('0xf0b8e814c3594d0c552d72fb3dfda7f0d9063458a7792369e7c044eda10f3b52')
const destination = privKeyToPeerId('0xf2462c7eec43cde144e025c8feeac547d8f87fb9ad87e625c833391085e94d5d')

function getRelayState(existing: boolean = false): Parameters<RelayHandshake['negotiate']>[2] {
  return {
    exists: () => existing,
    isActive: async () => false,
    updateExisting: () => false,
    createNew: async (_source: PeerId, _destination: PeerId, toSource: Stream, toDestination: Stream) => {
      if (existing) {
        toSource.sink(toDestination.source)
        toDestination.sink(toSource.source)
      }
    }
  }
}

describe('test relay handshake', function () {
  it('check initiating sequence', async function () {
    const [relayToInitiator, initiatorToRelay] = DuplexPair<StreamType>()

    const initiatorReceived = defer()

    const initiatorHandshake = new RelayHandshake(relayToInitiator)
    const relayHandshake = new RelayHandshake(initiatorToRelay)

    initiatorHandshake.initiate(relay, destination)

    await relayHandshake.negotiate(
      initiator,
      async (pId: PeerId) => {
        if (!pId.equals(destination)) {
          throw Error(`Invalid destination`)
        }

        return {
          stream: {
            source: (async function* () {
              yield Uint8Array.from([RelayHandshakeMessage.OK])
            })(),
            sink: async function (source: Stream['source']) {
              for await (const msg of source) {
                if (u8aEquals(msg.slice(), initiator.pubKey.marshal())) {
                  initiatorReceived.resolve()
                }
              }
            }
          },
          conn: {
            close: async () => {}
          } as any
        }
      },
      getRelayState()
    )

    await initiatorReceived.promise
  })

  it('check forwarding sequence', async function () {
    const [destinationToRelay, relayToDestination] = DuplexPair<StreamType>()

    const okReceived = defer()

    const relayHandshake = new RelayHandshake({
      source: (async function* () {
        yield destination.pubKey.marshal()
      })(),
      sink: async (source: Stream['source']) => {
        for await (const msg of source) {
          if (msg.slice()[0] == RelayHandshakeMessage.OK) {
            okReceived.resolve()
          }
        }
      }
    })

    const destinationHandshake = new RelayHandshake(relayToDestination).handle(relay)

    const handshakePromise = relayHandshake.negotiate(
      initiator,
      async () => {
        return {
          stream: destinationToRelay,
          conn: {
            close: async () => {}
          } as any
        }
      },
      getRelayState()
    )

    await Promise.all([handshakePromise, destinationHandshake])

    await okReceived.promise
  })

  it('should send messages after handshake', async function () {
    const [relayToInitiator, initiatorToRelay] = DuplexPair<StreamType>()
    const [destinationToRelay, relayToDestination] = DuplexPair<StreamType>()

    const initiatorHandshake = new RelayHandshake(relayToInitiator)
    const relayHandshake = new RelayHandshake(initiatorToRelay)

    const destinationHandshake = new RelayHandshake(relayToDestination)

    relayHandshake.negotiate(
      initiator,
      async () => {
        return {
          stream: destinationToRelay,
          conn: {
            close: async () => {}
          } as any
        }
      },
      getRelayState(true)
    )

    const [initiatorResult, destinationResult] = await Promise.all([
      initiatorHandshake.initiate(relay, destination),
      destinationHandshake.handle(relay)
    ])

    assert(initiatorResult.success && destinationResult.success)

    const messageInitiatorDestination = new TextEncoder().encode('initiatorMessage')
    const messageDestinationInitiator = new TextEncoder().encode('initiatorMessage')

    initiatorResult.stream.sink(
      (async function* () {
        yield messageInitiatorDestination
      })()
    )

    destinationResult.stream.sink(
      (async function* () {
        yield messageDestinationInitiator
      })()
    )

    let msgReceivedInitiator = false
    for await (const msg of initiatorResult.stream.source) {
      assert(u8aEquals(msg.slice(), messageDestinationInitiator))
      msgReceivedInitiator = true
    }

    let msgReceivedDestination = false
    for await (const msg of destinationResult.stream.source) {
      assert(u8aEquals(msg.slice(), messageInitiatorDestination))
      msgReceivedDestination = true
    }

    assert(msgReceivedDestination && msgReceivedInitiator)
  })
})
