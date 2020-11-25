import PeerId from 'peer-id'
import { keys as libp2p_crypto } from 'libp2p-crypto'
import { stringToU8a } from './u8a'

const COMPRESSED_PUBLIC_KEY_LENGTH = 33

/**
 * Converts a plain compressed ECDSA public key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 *
 * @notice Libp2p stores the keys in format that is derived from `protobuf`.
 * Using `libsecp256k1` directly does not work.
 *
 * @param pubKey the plain public key
 */
export function pubKeyToPeerId(pubKey: Uint8Array | string): Promise<PeerId> {
  if (typeof pubKey == 'string') {
    pubKey = stringToU8a(pubKey, COMPRESSED_PUBLIC_KEY_LENGTH)
  }

  if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH) {
    throw Error(
      `Invalid public key. Expected a buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH} bytes. Got one of ${pubKey.length} bytes.`
    )
  }

  const secp256k1PubKey = new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PublicKey(Buffer.from(pubKey))

  return PeerId.createFromPubKey(secp256k1PubKey.bytes)
}
