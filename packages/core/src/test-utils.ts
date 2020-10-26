import LibP2P from 'libp2p'
import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
export function connectionHelper(nodes: LibP2P[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.addressBook.add(nodes[j].peerId, nodes[j].multiaddrs)
      nodes[j].peerStore.addressBook.add(nodes[i].peerId, nodes[i].multiaddrs)
    }
  }
}

export type LibP2PMocks = {
  node: LibP2P
  address: Multiaddr
}

export async function generateLibP2PMock(
  addr = '/ip4/0.0.0.0/tcp/0'
): Promise<LibP2PMocks>{
  const node = await LibP2P.create({
    peerId: await PeerId.create({ keyType: 'secp256k1' }),
    addresses: { listen: [Multiaddr(addr)] },
    modules: {
      transport: [TCP],
      streamMuxer: [MPLEX],
      connEncryption: [SECIO]
    }
  })

  await node.start()

  return {
    node,
    address: node.multiaddrs[0].encapsulate('/p2p/' + node.peerId.toB58String()),
  }
}
