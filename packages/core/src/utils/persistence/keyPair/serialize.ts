import PeerId from 'peer-id'
import {encode} from 'rlp'
import {randomBytes, createCipheriv, scryptSync, createHmac} from 'crypto'

import {
  KEYPAIR_CIPHER_ALGORITHM,
  KEYPAIR_SALT_LENGTH,
  KEYPAIR_SCRYPT_PARAMS,
  KEYPAIR_IV_LENGTH,
  KEYPAIR_CIPHER_KEY_LENGTH,
  KEYPAIR_MESSAGE_DIGEST_ALGORITHM
} from '.'

/**
 * Serializes a given peerId by serializing the included private key and public key.
 *
 * @param peerId the peerId that should be serialized
 */
export async function serializeKeyPair(peerId: PeerId, password: Uint8Array) {
  const salt: Buffer = randomBytes(KEYPAIR_SALT_LENGTH)

  const key = scryptSync(password, salt, KEYPAIR_CIPHER_KEY_LENGTH, KEYPAIR_SCRYPT_PARAMS)

  const iv = randomBytes(KEYPAIR_IV_LENGTH)

  const ciphertext = createCipheriv(KEYPAIR_CIPHER_ALGORITHM, key, iv).update(peerId.marshal())

  const encodedCipherText = encode([iv, ciphertext])

  return encode([
    salt,
    createHmac(KEYPAIR_MESSAGE_DIGEST_ALGORITHM, key).update(encodedCipherText).digest(),
    encodedCipherText
  ])
}
