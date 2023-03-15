import type { Libp2p } from 'libp2p'
import type { Multiaddr } from '@multiformats/multiaddr'

export function getAddress(node: Libp2p): Multiaddr {
  let addr = node.getMultiaddrs()[0]
  if (!addr.getPeerId()) {
    addr = addr.encapsulate('/p2p/' + node.peerId.toString())
  }
  return addr
}
