/// <reference types="node" />
import PeerId from 'peer-id';
/**
 * Serializes a given peerId by serializing the included private key and public key.
 *
 * @param peerId the peerId that should be serialized
 */
export declare function serializeKeyPair(peerId: PeerId, password: Uint8Array): Promise<Buffer>;
