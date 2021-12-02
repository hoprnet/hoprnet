import PeerId from 'peer-id'
import { keys as libp2p_crypto } from 'libp2p-crypto'

import { stringToU8a } from '../u8a'

import secp256k1 from 'secp256k1'
import { encode } from 'multihashes'

const PRIVKEY_LENGTH = 32

/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 *
 * @param privKey the plain private key
 */
export function privKeyToPeerId(privKey: Uint8Array | string): PeerId {
  if (typeof privKey === 'string') {
    if (privKey.startsWith('0x')) {
      privKey = privKey.slice(2)
    }

    if (privKey.length != PRIVKEY_LENGTH * 2) {
      throw Error(`Incorrect private key size.`)
    }
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

  const id = encode(secp256k1PrivKey.public.bytes, 'identity')

  return new PeerId(id, secp256k1PrivKey, secp256k1PrivKey.public)
}
