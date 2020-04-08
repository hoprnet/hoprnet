/// <reference types="node" />
import PeerInfo from 'peer-info';
/**
 * Deserializes a serialized PeerInfo
 * @param arr Uint8Array that contains a serialized PeerInfo
 */
declare function deserializePeerInfo(arr: Buffer): Promise<PeerInfo>;
export { deserializePeerInfo };
