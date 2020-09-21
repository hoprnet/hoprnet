import { encode } from 'rlp'
import PeerInfo from 'peer-info'

import { serializePeerInfo } from '..'

export type SerializedPeerBook = Buffer[]

/**
 * Serializes a given peerBook by serializing the included peerInfo instances.
 *
 * @param {PeerBook} peerBook the peerBook instance
 * @returns the encoded peerBook
 */
export function serializePeerBook(peerBook: any): Uint8Array {
  // const peerInfos = []
  // peerBook.getAllArray().forEach((peerInfo: PeerInfo) => peerInfos.push(serializePeerInfo(peerInfo)))

  // return encode(peerInfos)
  throw Error('not implemented')
}
