import PeerId from 'peer-id';
/**
 * Converts a plain compressed ECDSA public key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 *
 * @notice Libp2p stores the keys in format that is derived from `protobuf`.
 * Using `libsecp256k1` directly does not work.
 *
 * @param pubKey the plain public key
 */
export declare function pubKeyToPeerId(pubKey: Uint8Array | string): Promise<PeerId>;
