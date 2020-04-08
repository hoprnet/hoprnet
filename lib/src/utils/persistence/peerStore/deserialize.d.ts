/**
 * Decodes the serialized peerBook and inserts the peerInfos in the given
 * peerBook instance.
 *
 * @param serializePeerBook the encodes serialized peerBook
 * @param peerBook a peerBook instance to store the peerInfo instances
 */
export declare function deserializePeerBook(serializedPeerBook: Uint8Array, peerBook?: any): Promise<any>;
