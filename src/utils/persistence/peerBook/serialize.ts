import rlp from 'rlp'
import * as PeerBook from 'peer-book'
import { MultiaddrClass } from 'multiaddr'
import PeerInfo from 'peer-info'

export type SerializedPeerInfo = [Buffer, Buffer[]] | [Buffer, Buffer[], Buffer]

export type SerializedPeerBook = [SerializedPeerInfo]

/**
 * Serializes a given peerBook by serializing the included peerInfo instances.
 *
 * @param {PeerBook} peerBook the peerBook instance
 * @returns the encoded peerBook
 */
export default function serializePeerBook(peerBook: PeerBook): Uint8Array {
    function serializePeerInfo(peerInfo: PeerInfo): SerializedPeerInfo {
        const result: SerializedPeerInfo = [peerInfo.id.toBytes(), peerInfo.multiaddrs.toArray().map((multiaddr: MultiaddrClass) => multiaddr.buffer)]

        if (peerInfo.id.pubKey) {
            result.push(peerInfo.id.pubKey.bytes)
        }

        return result
    }

    const peerInfos = []
    peerBook.getAllArray().forEach((peerInfo: PeerInfo) => peerInfos.push(serializePeerInfo(peerInfo)))

    return new Uint8Array(rlp.encode(peerInfos))
}