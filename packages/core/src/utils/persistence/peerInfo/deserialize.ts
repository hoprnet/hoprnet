import { decode } from 'rlp'

import PeerId from 'peer-id'
import PeerInfo from 'peer-info'

// @TODO get back to proper types
const Multiaddr = require('multiaddr')
import { keys as libp2pCrypto } from 'libp2p-crypto'

import { SerializedPeerInfo } from '.'

/**
 * Deserializes a serialized PeerInfo
 * @param arr Uint8Array that contains a serialized PeerInfo
 */
async function deserializePeerInfo(arr: Buffer): Promise<PeerInfo> {
  const serialized = (decode(Buffer.from(arr)) as unknown) as SerializedPeerInfo
  const peerId = PeerId.createFromBytes(serialized[0])

  if (serialized.length == 3) {
    peerId.pubKey = libp2pCrypto.unmarshalPublicKey(serialized[2])
  }

  const peerInfo = await PeerInfo.create(peerId)

  serialized[1].forEach((multiaddr: Buffer) => peerInfo.multiaddrs.add(Multiaddr(multiaddr)))

  return peerInfo
}

export { deserializePeerInfo }
