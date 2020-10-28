import Multiaddr from 'multiaddr'

// This should filter IP4's from private networks, as defined by RFC1918
// It also includes 0.0.0.0 as this is unroutable.
export const PRIVATE_NETS = /(^127\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^192\.168\.)|(^0\.0\.0\.0)/

export const peerHasOnlyPrivateAddresses = (peer: Multiaddr[]): boolean => {
  return peer.length > 0 && peer.filter((ma) => !isOnPrivateNet(ma)).length == 0
}

export const peerHasOnlyPublicAddresses = (peer: Multiaddr[]): boolean => {
  return peer.length > 0 && peer.filter(isOnPrivateNet).length == 0
}

export const isOnPrivateNet = (ma: Multiaddr): boolean => {
  if (['ip4', 'ip6', 'dns4', 'dns6'].includes(ma.protoNames()[0])) {
    return Boolean(ma.nodeAddress().address.match(PRIVATE_NETS))
  }
  // most likely not on private net
  return false
}
