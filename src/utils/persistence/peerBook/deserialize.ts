import rlp from 'rlp'
import * as PeerBook from 'peer-book'
import PeerId from 'peer-id'
import PeerInfo from 'peer-info'
import Multiaddr from 'multiaddr'
import { keys as libp2pCrypto } from 'libp2p-crypto'
import { SerializedPeerBook, SerializedPeerInfo} from './serialize'

/**
 * Decodes the serialized peerBook and inserts the peerInfos in the given
 * peerBook instance.
 *
 * @param {Buffer} serializePeerBook the encodes serialized peerBook
 * @param {PeerBook} peerBook a peerBook instance to store the peerInfo instances
 */
export default async function deserializePeerBook(serializedPeerBook: Uint8Array, peerBook?: PeerBook): PeerBook {
    const serializedPeerInfos = (rlp.decode(serializedPeerBook) as unknown) as SerializedPeerBook

    await Promise.all(
        serializedPeerInfos.map(async (serializedPeerInfo: SerializedPeerInfo) => {
            const peerId = PeerId.createFromBytes(serializedPeerInfo[0])

            if (serializedPeerInfo.length === 3) {
                peerId.pubKey = libp2pCrypto.unmarshalPublicKey(serializedPeerInfo[2])
            }

            const peerInfo = await PeerInfo.create(peerId)
            serializedPeerInfo[1].forEach((multiaddr: Buffer) => peerInfo.multiaddrs.add(Multiaddr(multiaddr)))
            peerBook.put(peerInfo)
        })
    )

    return peerBook
}
