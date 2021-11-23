import type LibP2P from 'libp2p'
import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import type NetworkPeers from './network/network-peers'
import { MAX_BACKOFF } from './network/network-peers'

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

export function showBackoff(networkPeers: NetworkPeers): number {
  const matches = networkPeers.debugLog().match(/(?<=\(backoff\s)(.*)(?=\,)/g)

  if (matches.length == 0) {
    return MAX_BACKOFF
  }

  return parseFloat(matches[0])
}
