import { createLibp2p, type Libp2p } from 'libp2p'
import { TCP } from '@libp2p/tcp'
import { Mplex } from '@libp2p/mplex'
import { Noise } from '@chainsafe/libp2p-noise'
import { KadDHT } from '@libp2p/kad-dht'
import { Multiaddr } from '@multiformats/multiaddr'

import type { PeerId } from '@libp2p/interface-peer-id'

import { createRelayerKey } from './relayCode.js'
import { privKeyToPeerId } from './privKeyToPeerId.js'

const peerA = privKeyToPeerId('0x06243fcfd7d7ba9364c9903b95cb8cfb3a3e6e95a80c96656598bda6942ae1c2')
const peerB = privKeyToPeerId('0x0e5574d6fcb05bc06542daeaa231639d26753f366b02fdc072944e728cbd4647')
const peerC = privKeyToPeerId('0x462684d27c3573981dd8b62ec4fbb92446dbb1797ef1278208f99216995015d5')

/**
 * Creates and starts a minimal libp2p instance
 * @param id peerId of the node to create
 * @returns a started libp2p instance with a DHT
 */
async function getNode(id: PeerId): Promise<Libp2p> {
  const node = await createLibp2p({
    addresses: {
      listen: [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${id.toString()}`).toString()]
    },
    peerId: id,
    transports: [new TCP()],
    streamMuxers: [new Mplex()],
    connectionEncryption: [new Noise()],
    // @ts-ignore
    dht: new KadDHT({ clientMode: false, protocolPrefix: '/hopr' }),
    metrics: {
      enabled: false
    },
    nat: {
      enabled: false
    },
    relay: {
      enabled: false
    },
    connectionManager: {
      autoDial: true
    }
  })

  await node.start()

  return node
}

describe('relay code generation', function () {
  it('provide and fetch CID key', async function () {
    this.timeout(9e3)

    const nodeA = await getNode(peerA)
    const nodeB = await getNode(peerB)
    const nodeC = await getNode(peerC)

    nodeA.peerStore.addressBook.set(nodeB.peerId, nodeB.getMultiaddrs())
    nodeC.peerStore.addressBook.set(nodeB.peerId, nodeB.getMultiaddrs())

    const CIDA = createRelayerKey(nodeB.peerId)

    while (true) {
      try {
        await nodeB.contentRouting.provide(CIDA)
        break
      } catch {}

      // busy waiting since there are no observable events
      await new Promise((resolve) => setTimeout(resolve, 1e3))
    }

    while (true) {
      const fetchedFromNodeA = []

      try {
        for await (const resultA of nodeB.contentRouting.findProviders(CIDA)) {
          fetchedFromNodeA.push(resultA)
        }
        if (fetchedFromNodeA.length > 0) {
          break
        }
      } catch {}

      // busy waiting since there are no observable events
      await new Promise((resolve) => setTimeout(resolve, 1e3))
    }

    while (true) {
      const fetchedFromNodeC = []

      try {
        for await (const resultC of nodeC.contentRouting.findProviders(CIDA)) {
          fetchedFromNodeC.push(resultC)
        }
        if (fetchedFromNodeC.length > 0) {
          break
        }
      } catch {}

      // busy waiting since there are no observable events
      await new Promise((resolve) => setTimeout(resolve, 1e3))
    }

    // Produces a timeout if not successful
    await Promise.all([nodeA.stop(), nodeB.stop(), nodeC.stop()])
  })

  // Check that nodes can renew keys in the DHT
  it('renew CID key', async function () {
    this.timeout(5e3)

    const nodeA = await getNode(peerA)
    const nodeB = await getNode(peerB)
    const nodeC = await getNode(peerC)

    nodeA.peerStore.addressBook.set(nodeB.peerId, nodeB.getMultiaddrs())
    nodeC.peerStore.addressBook.set(nodeB.peerId, nodeB.getMultiaddrs())

    const CIDA = createRelayerKey(nodeA.peerId)

    const ATTEMPTS = 3

    for (let i = 0; i < ATTEMPTS; i++) {
      while (true) {
        try {
          await nodeB.contentRouting.provide(CIDA)
          break
        } catch {}

        // busy waiting since there are no observable events
        await new Promise((resolve) => setTimeout(resolve, 1e3))
      }
      while (true) {
        const fetchedFromNodeA = []

        try {
          for await (const resultA of nodeB.contentRouting.findProviders(CIDA)) {
            fetchedFromNodeA.push(resultA)
          }
          if (fetchedFromNodeA.length > 0) {
            break
          }
        } catch {}

        // busy waiting since there are no observable events
        await new Promise((resolve) => setTimeout(resolve, 1e3))
      }

      while (true) {
        const fetchedFromNodeC = []

        try {
          for await (const resultC of nodeC.contentRouting.findProviders(CIDA)) {
            fetchedFromNodeC.push(resultC)
          }
          if (fetchedFromNodeC.length > 0) {
            break
          }
        } catch {}

        // busy waiting since there are no observable events
        await new Promise((resolve) => setTimeout(resolve, 1e3))
      }
    }

    // Produces a timeout if not successful
    await Promise.all([nodeA.stop(), nodeB.stop(), nodeC.stop()])
  })
})
