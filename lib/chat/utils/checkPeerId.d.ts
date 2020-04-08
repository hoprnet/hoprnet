import PeerId from 'peer-id';
/**
 * Takes the string representation of a peerId and checks whether it is a valid
 * peerId, i. e. it is a valid base58 encoding.
 * It then generates a PeerId instance and returns it.
 *
 * @param query query that contains the peerId
 */
export declare function checkPeerIdInput(query: string): Promise<PeerId>;
