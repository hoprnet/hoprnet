import {
  RelayHandshakeMessage,
  negotiateRelayHandshake,
  initiateRelayHandshake,
  handleRelayHandshake
} from './handshake.js'
import { u8aEquals, defer, privKeyToPeerId } from '@hoprnet/hopr-utils'
import { duplexPair } from 'it-pair/duplex'
import type { PeerId } from '@libp2p/interface-peer-id'
import assert from 'assert'
import type { Stream, StreamType } from '../types.js'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'
import { createFakeComponents, createFakeNetwork } from '../utils/libp2p.mock.spec.js'
import { getPeerStoreEntry } from '../base/utils.spec.js'
import { DELIVERY_PROTOCOLS } from '../constants.js'

const initiator = privKeyToPeerId('0x695a1ad048d12a1a82f827a38815ab33aa4464194fa0bdb99f78d9c66ec21505')
const relay = privKeyToPeerId('0xf0b8e814c3594d0c552d72fb3dfda7f0d9063458a7792369e7c044eda10f3b52')
const destination = privKeyToPeerId('0xf2462c7eec43cde144e025c8feeac547d8f87fb9ad87e625c833391085e94d5d')

function getRelayState(existing: boolean = false): Parameters<typeof negotiateRelayHandshake>[3] {
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
    const network = createFakeNetwork()
    const [relayToInitiator, initiatorToRelay] = duplexPair<StreamType>()

    const initiatorReceived = defer<void>()
    const relayEntry = getPeerStoreEntry('/ip4/127.0.0.1/tcp/1', destination)
    const destinationEntry = getPeerStoreEntry('/ip4/127.0.0.1/tcp/2', destination)

    const relayComponents = await createFakeComponents(relay, network, {
      listeningAddrs: relayEntry.multiaddrs
    })

    const destinationComponents = await createFakeComponents(destination, network, {
      listeningAddrs: destinationEntry.multiaddrs,
      protocols: [
        [
          DELIVERY_PROTOCOLS(),
          async ({ stream }) => {
            stream.sink(
              (async function* () {
                console.log(`sent`)
                yield Uint8Array.from([RelayHandshakeMessage.OK])
              })()
            )
            for await (const msg of stream.source) {
              if (u8aEquals(msg.slice(), unmarshalPublicKey(initiator.publicKey as Uint8Array).marshal())) {
                initiatorReceived.resolve()
              }
            }
          }
        ]
      ]
    })

    await relayComponents
      .getPeerStore()
      .addressBook.add(destinationComponents.getPeerId(), destinationComponents.getTransportManager().getAddrs())

    initiateRelayHandshake(relayToInitiator, relay, destination)

    await negotiateRelayHandshake(initiatorToRelay, initiator, relayComponents, getRelayState(), {})

    await initiatorReceived.promise
    network.stop()
  })

  it('check forwarding sequence', async function () {
    const network = createFakeNetwork()
    const [destinationToRelay, relayToDestination] = duplexPair<StreamType>()

    const relayEntry = getPeerStoreEntry('/ip4/127.0.0.1/tcp/1', destination)
    const destinationEntry = getPeerStoreEntry('/ip4/127.0.0.1/tcp/2', destination)

    const relayComponents = await createFakeComponents(relay, network, {
      listeningAddrs: relayEntry.multiaddrs
    })

    const destinationComponents = await createFakeComponents(destination, network, {
      listeningAddrs: destinationEntry.multiaddrs,
      protocols: [
        [
          DELIVERY_PROTOCOLS(),
          async ({ stream }) => {
            stream.sink(destinationToRelay.source)
            destinationToRelay.sink(stream.source)
          }
        ]
      ]
    })

    const okReceived = defer<void>()

    const destinationHandshake = handleRelayHandshake(relayToDestination, relay)

    await relayComponents
      .getPeerStore()
      .addressBook.add(destinationComponents.getPeerId(), destinationComponents.getTransportManager().getAddrs())

    const handshakePromise = negotiateRelayHandshake(
      {
        source: (async function* (): AsyncIterable<Uint8Array> {
          yield unmarshalPublicKey(destination.publicKey as Uint8Array).marshal()
        })(),
        sink: async (source: Stream['source']) => {
          for await (const msg of source) {
            if (msg.slice()[0] == RelayHandshakeMessage.OK) {
              okReceived.resolve()
            }
          }
        }
      },
      initiator,
      relayComponents,
      getRelayState(),
      {}
    )

    await Promise.all([handshakePromise, destinationHandshake])

    await okReceived.promise
  })

  it('should send messages after handshake', async function () {
    const network = createFakeNetwork()
    const [relayToInitiator, initiatorToRelay] = duplexPair<StreamType>()
    const [destinationToRelay, relayToDestination] = duplexPair<StreamType>()

    const relayEntry = getPeerStoreEntry('/ip4/127.0.0.1/tcp/1', destination)
    const destinationEntry = getPeerStoreEntry('/ip4/127.0.0.1/tcp/2', destination)

    const relayComponents = await createFakeComponents(relay, network, {
      listeningAddrs: relayEntry.multiaddrs
    })

    const destinationComponents = await createFakeComponents(destination, network, {
      listeningAddrs: destinationEntry.multiaddrs,
      protocols: [
        [
          DELIVERY_PROTOCOLS(),
          async ({ stream }) => {
            stream.sink(destinationToRelay.source)
            destinationToRelay.sink(stream.source)
          }
        ]
      ]
    })

    await relayComponents
      .getPeerStore()
      .addressBook.add(destinationComponents.getPeerId(), destinationComponents.getTransportManager().getAddrs())

    negotiateRelayHandshake(relayToInitiator, initiator, relayComponents, getRelayState(true), {})

    const [initiatorResult, destinationResult] = await Promise.all([
      initiateRelayHandshake(initiatorToRelay, relay, destination),
      handleRelayHandshake(relayToDestination, relay)
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
    network.stop()
  })
})
