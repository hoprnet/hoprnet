import { createCipheriv, scryptSync, createHmac } from 'crypto'

import {
  KEYPAIR_CIPHER_ALGORITHM,
  KEYPAIR_SCRYPT_PARAMS,
  KEYPAIR_IV_LENGTH,
  KEYPAIR_CIPHER_KEY_LENGTH,
  KEYPAIR_MESSAGE_DIGEST_ALGORITHM
} from '.'

import { privKeyToPeerId, u8aEquals } from '@hoprnet/hopr-utils'

/**
 * Deserializes a serialized key pair and returns a peerId.
 *
 * @notice This method will ask for a password to decrypt the encrypted
 * private key.
 * @notice The decryption of the private key makes use of a memory-hard
 * hash function and consumes therefore a lot of memory.
 *
 * @param encryptedSerializedKeyPair the encoded and encrypted key pair
 */
export async function deserializeKeyPair(encryptedSerializedKeyPair: Uint8Array, password: Uint8Array) {
  const [salt, mac, iv, ciphertext] = [
    encryptedSerializedKeyPair.subarray(0, 32),
    encryptedSerializedKeyPair.subarray(32, 64),
    encryptedSerializedKeyPair.subarray(64, 80),
    encryptedSerializedKeyPair.subarray(80, 112)
  ]

  const key = scryptSync(password, salt, KEYPAIR_CIPHER_KEY_LENGTH, KEYPAIR_SCRYPT_PARAMS)

  if (
    !u8aEquals(
      createHmac(KEYPAIR_MESSAGE_DIGEST_ALGORITHM, key)
        .update(Uint8Array.from([...iv, ...ciphertext]))
        .digest(),
      mac
    )
  ) {
    throw Error(`Invalid MAC. Ciphertext might have been corrupted`)
  }

  if (iv.length != KEYPAIR_IV_LENGTH) {
    throw Error('Invalid IV length.')
  }

  let plaintext = createCipheriv(KEYPAIR_CIPHER_ALGORITHM, key, iv).update(ciphertext)

  return await privKeyToPeerId(plaintext)
}
