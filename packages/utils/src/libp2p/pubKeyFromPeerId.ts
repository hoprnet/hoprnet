import type PeerId from 'peer-id'

export function peerIdToPubKey(peerId: PeerId) {
  return peerId.pubKey.marshal()
}
