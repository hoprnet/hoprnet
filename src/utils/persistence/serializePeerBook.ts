import rlp from 'rlp'
import * as PeerBook from 'peer-book'
import * as Multiaddr from 'multiaddr'
import * as PeerInfo from 'peer-info'

/**
 * Serializes a given peerBook by serializing the included peerInfo instances.
 *
 * @param {PeerBook} peerBook the peerBook instance
 * @returns the encoded peerBook
 */
export default function serializePeerBook(peerBook: PeerBook): Uint8Array {
    function serializePeerInfo(peerInfo: PeerInfo) {
        const result = [peerInfo.id.toBytes(), peerInfo.multiaddrs.toArray().map((multiaddr: Multiaddr) => multiaddr.buffer)]

        if (peerInfo.id.pubKey) {
            result.push(peerInfo.id.pubKey.bytes)
        }

        return result
    }

    const peerInfos = []
    peerBook.getAllArray().forEach(peerInfo => peerInfos.push(serializePeerInfo(peerInfo)))

    return new Uint8Array(rlp.encode(peerInfos))
}