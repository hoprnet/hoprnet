import PeerId from 'peer-id'
import rlp from 'rlp'
import { randomBytes, createCipheriv, scryptSync } from 'crypto'
import chalk from 'chalk'
import askForPassword from '../askForPassword'

const SALT_LENGTH = 32
const CIPHER_ALGORITHM = 'chacha20'

/**
 * Serializes a given peerId by serializing the included private key and public key.
 *
 * @param peerId the peerId that should be serialized
 */
export async function serializeKeyPair(peerId: PeerId) {
    const salt: Buffer = randomBytes(SALT_LENGTH)
    const scryptParams = { N: 8192, r: 8, p: 16 }

    const question = 'Please type in the password that will be used to encrypt the generated key.'

    const pw: string = await askForPassword(question)

    console.log(`Done. Using peerId '${chalk.blue(peerId.toB58String())}'`)

    const key = scryptSync(pw, salt, 32, scryptParams)
    const iv = randomBytes(12)

    const serializedPeerId = Buffer.concat([Buffer.alloc(16, 0), peerId.marshal()])

    const ciphertext = createCipheriv(CIPHER_ALGORITHM, key, iv).update(serializedPeerId)

    return rlp.encode([salt, iv, ciphertext])
}
