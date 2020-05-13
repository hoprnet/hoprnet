"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deserializePeerBook = void 0;
/**
 * Decodes the serialized peerBook and inserts the peerInfos in the given
 * peerBook instance.
 *
 * @param serializePeerBook the encodes serialized peerBook
 * @param peerBook a peerBook instance to store the peerInfo instances
 */
async function deserializePeerBook(serializedPeerBook, peerBook) {
    // if (peerBook == null) {
    //   peerBook = new PeerBook()
    // }
    // const serializedPeerInfos = (decode(serializedPeerBook) as unknown) as SerializedPeerBook
    // await Promise.all(
    //   serializedPeerInfos.map(async (serializedPeerInfo: Buffer) => {
    //     const peerInfo = await deserializePeerInfo(serializedPeerInfo)
    //     peerBook.put(peerInfo)
    //   })
    // )
    // return peerBook
    throw Error('not implemented');
    return;
}
exports.deserializePeerBook = deserializePeerBook;
//# sourceMappingURL=deserialize.js.map