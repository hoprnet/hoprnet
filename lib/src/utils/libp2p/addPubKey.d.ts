import PeerId from 'peer-id';
/**
 * Takes a peerId and returns a peerId with the public key set to the corresponding
 * public key.
 *
 * @param peerId the PeerId instance that has probably no pubKey set
 */
export declare function addPubKey(peerId: PeerId): Promise<PeerId>;
