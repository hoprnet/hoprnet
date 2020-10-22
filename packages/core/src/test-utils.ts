import LibP2P from 'libp2p'

/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
export function connectionHelper(nodes: LibP2P[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.addressBook.add(nodes[j].peerId, nodes[j].multiaddrs[0])
      nodes[j].peerStore.addressBook.add(nodes[i].peerId, nodes[i].multiaddrs[0])
    }
  }
}
