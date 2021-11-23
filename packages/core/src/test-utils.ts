import type LibP2P from 'libp2p'
import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import type { PeerStore } from 'libp2p-kad-dht'

export function getAddress(node: LibP2P): Multiaddr {
  let addr = node.multiaddrs[0]
  if (!addr.getPeerId()) {
    addr = addr.encapsulate('/p2p/' + node.peerId.toB58String())
  }
  return addr
}

export function fakePeerId(i: number | string): PeerId {
  return {
    id: i as unknown as Uint8Array,
    equals: (x: PeerId) => (x.id as unknown as number) == i,
    toB58String: () => i
  } as PeerId
}

export function fakeAddress(id: PeerId): Multiaddr {
  return {
    getPeerId: () => id.toB58String()
  } as Multiaddr
}

export function showBackoff(networkPeers: PeerStore): number {
  return parseFloat(networkPeers.debugLog().match(/(?<=\(backoff\s)(.*)(?=\,)/g))
}
