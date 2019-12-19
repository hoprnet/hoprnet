import rlp from 'rlp'
import * as PeerBook from 'peer-book'
import * as PeerId from 'peer-id'
import * as PeerInfo from 'peer-info'
import * as Multiaddr from 'multiaddr'
import * as libp2pCrypto from 'libp2p-crypto'

/**
 * Decodes the serialized peerBook and inserts the peerInfos in the given
 * peerBook instance.
 *
 * @param {Buffer} serializePeerBook the encodes serialized peerBook
 * @param {PeerBook} peerBook a peerBook instance to store the peerInfo instances
 */
export default async function deserializePeerBook(serializedPeerBook: Uint8Array, peerBook?: PeerBook): PeerBook {
    const serializedPeerInfos = rlp.decode(serializedPeerBook)

    await Promise.all(
        serializedPeerInfos.map(async (serializedPeerInfo: [any, any, any]) => {
            const peerId = PeerId.createFromBytes(serializedPeerInfo[0])

            if (serializedPeerInfo.length === 3) {
                peerId.pubKey = libp2pCrypto.unmarshalPublicKey(serializedPeerInfo[2])
            }

            const peerInfo = await PeerInfo.create(peerId)
            serializedPeerInfo[1].forEach(multiaddr => peerInfo.multiaddrs.add(Multiaddr(multiaddr)))
            peerBook.put(peerInfo)
        })
    )

    return peerBook
}
