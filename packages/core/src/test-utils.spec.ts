import type { Libp2p } from 'libp2p'
import type { Multiaddr } from '@multiformats/multiaddr'
import type { PeerId } from '@libp2p/interface-peer-id'
import type NetworkPeers from './network/network-peers.js'
import { MAX_BACKOFF } from './network/network-peers.js'

export function getAddress(node: Libp2p): Multiaddr {
  let addr = node.multiaddrs[0]
  if (!addr.getPeerId()) {
    addr = addr.encapsulate('/p2p/' + node.peerId.toString())
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
    getPeerId: () => id.toString()
  } as Multiaddr
}

export function showBackoff(networkPeers: NetworkPeers): number {
  const matches = networkPeers.debugLog().match(/(?<=\,\sbackoff:\s)(.*)(?=\s\()/g)

  if (matches.length == 0) {
    return MAX_BACKOFF
  }

  const backoffs = matches.map((m) => parseFloat(m))

  return backoffs.sort().pop()
}
