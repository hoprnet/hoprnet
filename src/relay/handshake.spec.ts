/// <reference path="../@types/it-pair.ts" />

import { RelayHandshake, RelayHandshakeMessage, StreamResult } from './handshake'
import { u8aEquals } from '@hoprnet/hopr-utils'
import Pair from 'it-pair'
import PeerId from 'peer-id'
import Defer from 'p-defer'
import assert from 'assert'
import { Stream } from 'libp2p'

describe('test relay handshake', function () {
  let initiator: PeerId, relay: PeerId, destination: PeerId

  before(async function () {
    ;[initiator, relay, destination] = await Promise.all(
      Array.from({ length: 3 }, (_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    console.log(
      `initiator`,
      initiator.toB58String(),
      `relay`,
      relay.toB58String(),
      `destination`,
      destination.toB58String()
    )
  })

  it('check initiating sequence', async function () {
    const initiatorToRelay = Pair()
    const relayToInitiator = Pair()

    const initiatorReceived = Defer()

    const initiatorHandshake = new RelayHandshake({
      source: relayToInitiator.source,
      sink: initiatorToRelay.sink
    })

    const relayHandshake = new RelayHandshake({
      source: initiatorToRelay.source,
      sink: relayToInitiator.sink
    })

    initiatorHandshake.initiate(relay, destination)

    await relayHandshake.negotiate(
      initiator,
      async (pId: PeerId) => {
        console.log(`attempting to establish connection to`, pId.toB58String())
        if (!pId.equals(destination)) {
          throw Error(`Invalid destination`)
        }

        return {
          source: (async function* () {
            yield Uint8Array.from([RelayHandshakeMessage.OK])
          })(),
          sink: async function (source: AsyncIterable<StreamResult>) {
            for await (const msg of source) {
              if (u8aEquals(msg.slice(), initiator.pubKey.marshal())) {
                initiatorReceived.resolve()
              }
            }
          }
        }
      },
      () => false,
      async () => true,
      () => {},
      () => {}
    )

    await initiatorReceived.promise
  })

  it('check forwarding sequence', async function () {
    const relayToDestination = Pair()
    const destinationToRelay = Pair()

    const okReceived = Defer()

    const relayHandshake = new RelayHandshake({
      source: (async function* () {
        yield destination.pubKey.marshal()
      })(),
      sink: async (source: AsyncIterable<StreamResult>) => {
        for await (const msg of source) {
          if (msg.slice()[0] == RelayHandshakeMessage.OK) {
            okReceived.resolve()
          }
        }
      }
    })

    const destinationHandshake = new RelayHandshake({
      source: relayToDestination.source,
      sink: destinationToRelay.sink
    }).handle(relay)

    const handshakePromise = relayHandshake.negotiate(
      initiator,
      async () => {
        return {
          source: destinationToRelay.source,
          sink: relayToDestination.sink
        }
      },
      () => false,
      async () => true,
      () => {},
      () => {}
    )

    await Promise.all([handshakePromise, destinationHandshake])

    await okReceived.promise
  })

  it('should send messages after handshake', async function () {
    const initiatorToRelay = Pair()
    const relayToInitiator = Pair()

    const relayToDestination = Pair()
    const destinationToRelay = Pair()

    const initiatorHandshake = new RelayHandshake({
      source: relayToInitiator.source,
      sink: initiatorToRelay.sink
    })

    const relayHandshake = new RelayHandshake({
      source: initiatorToRelay.source,
      sink: relayToInitiator.sink
    })

    const destinationHandshake = new RelayHandshake({
      source: relayToDestination.source,
      sink: destinationToRelay.sink
    })

    relayHandshake.negotiate(
      initiator,
      async () => {
        return {
          source: destinationToRelay.source,
          sink: relayToDestination.sink
        }
      },
      () => false,
      async () => true,
      () => {},
      (_source, _destination, toSource: Stream, toDestination: Stream) => {
        toSource.sink(toDestination.source)
        toDestination.sink(toSource.source)
      }
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
