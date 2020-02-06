import PeerInfo from 'peer-info'
import Multiaddr from 'multiaddr'

import { encode } from 'rlp'

import { SerializedPeerInfo } from '.'
/**
 * Serializes peerInfos including their multiaddrs.
 * @param peerInfo PeerInfo to serialize
 */
function serializePeerInfo(peerInfo: PeerInfo): Uint8Array {
  const result: SerializedPeerInfo = [peerInfo.id.toBytes(), peerInfo.multiaddrs.toArray().map((multiaddr: Multiaddr) => multiaddr.buffer)]

  if (peerInfo.id.pubKey) {
    result.push(peerInfo.id.pubKey.bytes)
  }

  return encode(result)
}

export { serializePeerInfo }
