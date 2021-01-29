import PeerId from 'peer-id'
import { keys as libp2p_crypto } from 'libp2p-crypto'

import { stringToU8a } from '../u8a'

import secp256k1 from 'secp256k1'

const PRIVKEY_LENGTH = 32

/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 *
 * @param privKey the plain private key
 */
export function privKeyToPeerId(privKey: Uint8Array | string): Promise<PeerId> {
  if (typeof privKey == 'string') {
    privKey = stringToU8a(privKey, PRIVKEY_LENGTH)
  }

  if (privKey.length != PRIVKEY_LENGTH) {
    throw Error(
      `Invalid private key. Expected a buffer of size ${PRIVKEY_LENGTH} bytes. Got one of ${privKey.length} bytes.`
    )
  }

  const secp256k1PrivKey = new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PrivateKey(
    Buffer.from(privKey),
    secp256k1.publicKeyCreate(privKey)
  )

  return PeerId.createFromPrivKey(secp256k1PrivKey.bytes)
}
