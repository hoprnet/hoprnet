import { decode } from 'rlp'
import PeerBook from 'peer-book'

import { SerializedPeerBook } from './serialize'
import { deserializePeerInfo } from '..'
/**
 * Decodes the serialized peerBook and inserts the peerInfos in the given
 * peerBook instance.
 *
 * @param serializePeerBook the encodes serialized peerBook
 * @param peerBook a peerBook instance to store the peerInfo instances
 */
export async function deserializePeerBook(serializedPeerBook: Uint8Array, peerBook?: PeerBook): Promise<PeerBook> {
  if (peerBook == null) {
    peerBook = new PeerBook()
  }

  const serializedPeerInfos = (decode(serializedPeerBook) as unknown) as SerializedPeerBook

  await Promise.all(
    serializedPeerInfos.map(async (serializedPeerInfo: Buffer) => {
      const peerInfo = await deserializePeerInfo(serializedPeerInfo)
      peerBook.put(peerInfo)
    })
  )

  return peerBook
}
