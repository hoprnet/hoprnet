import { randomBytes } from 'crypto'

import Libp2p from 'libp2p'
import TCP from 'libp2p-tcp'
import MPLEX from 'libp2p-mplex'
import { NOISE } from '@chainsafe/libp2p-noise'
import KadDHT from 'libp2p-kad-dht'
import { Multiaddr } from 'multiaddr'

import type PeerId from 'peer-id'

import { createRelayerKey } from './relayCode'
import { privKeyToPeerId } from './privKeyToPeerId'
import assert from 'assert'

const peerA = privKeyToPeerId('0x06243fcfd7d7ba9364c9903b95cb8cfb3a3e6e95a80c96656598bda6942ae1c2')
const peerB = privKeyToPeerId('0x0e5574d6fcb05bc06542daeaa231639d26753f366b02fdc072944e728cbd4647')
const peerC = privKeyToPeerId('0x462684d27c3573981dd8b62ec4fbb92446dbb1797ef1278208f99216995015d5')

/**
 * Synchronous method to sample peerIds
 * @returns a random peerId
 */
function getPeerId(): PeerId {
  return privKeyToPeerId(randomBytes(32))
}

/**
 * Creates and starts a minimal libp2p instance
 * @param id peerId of the node to create
 * @returns a started libp2p instance with a DHT
 */
async function getNode(id = getPeerId()): Promise<Libp2p> {
  const node = await Libp2p.create({
    addresses: {
      listen: [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${id.toB58String()}`).toString()]
    },
    peerId: id,
    modules: {
      transport: [TCP],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE],
      dht: KadDHT
    },
    metrics: {
      enabled: false
    },
    config: {
      dht: {
        enabled: true,
        // @ts-ignore
        bootstrapPeers: [peerA, peerB].map((id) => ({ id, multiaddrs: [] }))
      },
      nat: {
        enabled: false
      },
      relay: {
        enabled: false
      },
      peerDiscovery: {
        autoDial: false
      }
    }
  })

  await node.start()

  return node
}

describe('relay code generation', function () {
  it('provide and fetch CID key', async function () {
    const nodeA = await getNode(peerA)
    const nodeB = await getNode(peerB)
    const nodeC = await getNode(peerC)

    await nodeA.dial(nodeB.multiaddrs[0])
    const CIDA = await createRelayerKey(nodeA.peerId)

    await nodeA.contentRouting.provide(CIDA)

    const fetchedFromNodeB = []
    for await (const resultB of nodeB.contentRouting.findProviders(CIDA)) {
      fetchedFromNodeB.push(resultB)
    }

    assert(fetchedFromNodeB.length > 0, `Node B must be able to perform the DHT query`)

    // Add PeerC to the network
    await nodeB.dial(nodeC.multiaddrs[0])

    const fetchedFromNodeC = []
    for await (const resultC of nodeC.contentRouting.findProviders(CIDA)) {
      fetchedFromNodeC.push(resultC)
    }

    assert(fetchedFromNodeC.length > 0, `Node C must be able to perform the DHT query`)

    await Promise.all([nodeA.stop(), nodeB.stop(), nodeC.stop()])
  })
})
