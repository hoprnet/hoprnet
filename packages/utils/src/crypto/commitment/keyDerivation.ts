import { expand } from 'futoin-hkdf'
import { createHmac } from 'crypto'
import { HASH_ALGORITHM, HASH_LENGTH, SECRET_LENGTH } from './constants'

const HASH_KEY_COMMITMENT_SEED = 'HASH_KEY_COMMITMENT_SEED'

/**
 * Derives the initial commitment seed on a newly opened channel.
 * @param privateKey Node private key.
 * @param channelInfo Additional information identifying the channel.
 */
export function deriveCommitmentSeed(privateKey: Uint8Array, channelInfo: Uint8Array): Uint8Array {
  const key = expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(privateKey), SECRET_LENGTH, HASH_KEY_COMMITMENT_SEED)

    return new Uint8Array(createHmac(HASH_ALGORITHM, key).update(channelInfo).digest().buffer)
}
