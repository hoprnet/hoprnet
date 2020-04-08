/// <reference types="node" />
export declare type SerializedPeerBook = Buffer[];
/**
 * Serializes a given peerBook by serializing the included peerInfo instances.
 *
 * @param {PeerBook} peerBook the peerBook instance
 * @returns the encoded peerBook
 */
export declare function serializePeerBook(peerBook: any): Uint8Array;
