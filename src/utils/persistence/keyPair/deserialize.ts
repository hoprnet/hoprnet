import rlp from 'rlp'
import { createCipheriv, scryptSync } from 'crypto'
import chalk from 'chalk'
import PeerId from 'peer-id'

import askForPassword from '../askForPassword'

const CIPHER_ALGORITHM = 'chacha20'

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
export async function deserializeKeyPair(encryptedSerializedKeyPair: Uint8Array) {
    const [salt, iv, ciphertext] = rlp.decode(encryptedSerializedKeyPair) as Buffer[]

    const scryptParams = { N: 8192, r: 8, p: 16 }

    const question = 'Please type in the password that was used to encrypt the key.'

    let plaintext: Buffer

    do {
        const pw = await askForPassword(question)

        const key = scryptSync(pw, salt, 32, scryptParams)
        
        plaintext = createCipheriv(CIPHER_ALGORITHM, key, iv).update(ciphertext)
    } while (!plaintext.slice(0, 16).equals(Buffer.alloc(16, 0)))

    const peerId: PeerId = await PeerId.createFromProtobuf(plaintext)
    console.log(`Successfully restored ID ${chalk.blue(peerId.toB58String())}.`)

    return peerId
}