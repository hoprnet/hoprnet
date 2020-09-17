import PeerInfo from 'peer-info'
import Multiaddr from 'multiaddr'

// This should filter IP4's from private networks, as defined by RFC1918
export const PRIVATE_NETS = /(^127\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^192\.168\.)/

export const peerHasOnlyPrivateAddresses = (peer: PeerInfo): boolean => {
  return peer.multiaddrs.size > 0 && peer.multiaddrs.toArray().filter((ma) => !isOnPrivateNet(ma)).length == 0
}

export const peerHasOnlyPublicAddresses = (peer: PeerInfo): boolean => {
  return peer.multiaddrs.size > 0 && peer.multiaddrs.toArray().filter(isOnPrivateNet).length == 0
}

export const isOnPrivateNet = (ma: Multiaddr): boolean => {
  return Boolean(ma.nodeAddress().address.match(PRIVATE_NETS))
}
