import type PeerId from 'peer-id'
import { privKeyToPeerId } from '..'
import { debug } from '../debug'
import { Wallet } from '@ethersproject/wallet'

const logError = debug('hopr:keypair')

/**
 * Serializes a peerId using geth's KeyStore format
 * see https://medium.com/@julien.maffre/what-is-an-ethereum-keystore-file-86c8c5917b97
 * @dev This method uses a computation and memory intensive hash function,
 *      for testing set `useWeaKCrytpo = true`
 * @param peerId id to store
 * @param password password used for encryption
 * @param useWeakCrypto [optional] weak parameter for fast serialization
 * @returns Uint8Array representation
 */
export async function serializeKeyPair(
  peerId: PeerId,
  password: string,
  useWeakCrypto = false,
  __iv?: string,
  __salt?: string,
  __uuidSalt?: string
): Promise<Uint8Array> {
  const w = new Wallet(peerId.privKey.marshal() as Buffer)

  let serialized: string
  if (useWeakCrypto) {
    // Use weak settings during development for quicker
    // node startup
    serialized = await w.encrypt(password, {
      scrypt: {
        N: 1
      },
      salt: __salt,
      iv: __iv,
      uuid: __uuidSalt
    })
  } else {
    serialized = await w.encrypt(password)
  }

  return new TextEncoder().encode(serialized)
}

type DeserializationError = 'Wrong usage of weak crypto' | 'Invalid password'

type DeserializationResponse =
  | {
      success: false
      error?: DeserializationError
    }
  | {
      success: true
      identity: PeerId
    }

/**
 * Deserializes an encoded key pair
 * @dev This method uses a computation and memory intensive hash function,
 *      for testing set `useWeaKCrytpo = true`
 * @param serialized encoded key pair
 * @param password password to use for decryption
 * @param useWeakCrypto [optional] use faster but weaker crypto to reconstruct key pair
 * @returns reconstructed key pair
 */
export async function deserializeKeyPair(
  serialized: Uint8Array,
  password: string,
  useWeakCrypto = false
): Promise<DeserializationResponse> {
  const encodedString = new TextDecoder().decode(serialized)

  const decoded = JSON.parse(encodedString)

  if ((decoded.Crypto.kdfparams.n == 1) != useWeakCrypto) {
    logError(
      `Either tried to deserialize a key file using weak crypto or to deserialize a weak crypto key file without using weak crypto.`
    )

    return {
      success: false,
      error: 'Wrong usage of weak crypto'
    }
  }

  let w: Wallet
  try {
    w = await Wallet.fromEncryptedJson(encodedString, password)
  } catch (err) {
    logError(`Key deserialization faild,`, err)

    return {
      success: false,
      error: 'Invalid password'
    }
  }

  return {
    success: true,
    identity: privKeyToPeerId(w.privateKey)
  }
}
