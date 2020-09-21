import PeerInfo from 'peer-info'

// @TODO get back to proper types
// const Multiaddr = require('multiaddr')

import { encode } from 'rlp'

import { SerializedPeerInfo } from '.'
/**
 * Serializes peerInfos including their multiaddrs.
 * @param peerInfo PeerInfo to serialize
 */
function serializePeerInfo(peerInfo: PeerInfo): Uint8Array {
  const result: SerializedPeerInfo = [
    peerInfo.id.toBytes(),
    peerInfo.multiaddrs.toArray().map((multiaddr: any) => multiaddr.buffer),
  ]

  if (peerInfo.id.pubKey) {
    result.push(peerInfo.id.pubKey.bytes)
  }

  return encode(result)
}

export { serializePeerInfo }
