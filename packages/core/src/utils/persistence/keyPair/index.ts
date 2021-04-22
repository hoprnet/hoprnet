import PeerId from 'peer-id'
import { randomBytes, createCipheriv, scryptSync, createHmac } from 'crypto'
import { privKeyToPeerId, u8aEquals } from '@hoprnet/hopr-utils'

export const KEYPAIR_CIPHER_ALGORITHM = 'chacha20'
export const KEYPAIR_IV_LENGTH = 16
export const KEYPAIR_CIPHER_KEY_LENGTH = 32
export const KEYPAIR_SALT_LENGTH = 32
export const KEYPAIR_SCRYPT_PARAMS = { N: 8192, r: 8, p: 16 }
export const KEYPAIR_PADDING = Buffer.alloc(16, 0x00)
export const KEYPAIR_MESSAGE_DIGEST_ALGORITHM = 'sha256'

/**
 * Serializes a given peerId by serializing the included private key and public key.
 *
 * @param peerId the peerId that should be serialized
 */
export function serializeKeyPair(peerId: PeerId, password: Uint8Array) {
  const salt: Buffer = randomBytes(KEYPAIR_SALT_LENGTH)

  const key = scryptSync(password, salt, KEYPAIR_CIPHER_KEY_LENGTH, KEYPAIR_SCRYPT_PARAMS)

  const iv = randomBytes(KEYPAIR_IV_LENGTH)

  const ciphertext = createCipheriv(KEYPAIR_CIPHER_ALGORITHM, key, iv).update(peerId.privKey.marshal())

  return Uint8Array.from([
    ...salt,
    ...createHmac(KEYPAIR_MESSAGE_DIGEST_ALGORITHM, key)
      .update(Uint8Array.from([...iv, ...ciphertext]))
      .digest(),
    ...iv,
    ...ciphertext
  ])
}

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
