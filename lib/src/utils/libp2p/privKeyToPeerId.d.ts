import PeerId from 'peer-id';
/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 *
 * @param privKey the plain private key
 */
export declare function privKeyToPeerId(privKey: Uint8Array | string): Promise<PeerId>;
