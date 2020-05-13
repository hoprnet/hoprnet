import assert from 'assert'

// @ts-ignore
import libp2p = require('libp2p')

// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')

import PeerId from 'peer-id'

import { MultiaddrConnection } from './types'

import TCP from '.'
import Multiaddr from 'multiaddr'
import PeerInfo = require('peer-info')

describe('should create a socket and connect to it', function () {
  const upgrader = {
    upgradeOutbound: async (maConn: MultiaddrConnection) => maConn,
    upgradeInbound: async (maConn: MultiaddrConnection) => maConn,
  }

  async function generateNode(id: number, bootstrap?: PeerInfo): Promise<libp2p> {
    const peerInfo = new PeerInfo(await PeerId.create({ keyType: 'secp256k1' }))

    peerInfo.multiaddrs.add(
      Multiaddr(`/ip4/127.0.0.1/tcp/${9090 + id}`).encapsulate(`/ipfs/${peerInfo.id.toB58String()}`)
    )

    const node = new libp2p({
      peerInfo,
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
        dht: KadDHT,
      },
      config: {
        transport: {
          TCP: {
            bootstrap
          }
        },
        dht: {
          enabled: false,
        },
        relay: {
          enabled: false,
        },
        peerDiscovery: {
          autoDial: false,
        },
      },
    })

    await node.start()

    return node
  }

  it('should set up a socket', async function () {
    const relay = await generateNode(2)

    const [sender, counterparty] = await Promise.all([
      generateNode(0, relay.peerInfo),
      generateNode(1, relay.peerInfo),
    ])

    connectionHelper([sender, relay])
    connectionHelper([relay, counterparty])

    const conn = await sender.dial(counterparty.peerInfo, {
      relay: relay.peerInfo,
    })
  })
})

/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
function connectionHelper(nodes: libp2p[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.put(nodes[j].peerInfo)
      nodes[j].peerStore.put(nodes[i].peerInfo)
    }
  }
}
