import type PeerId from 'peer-id'
import debug from 'debug'
import { privKeyToPeerId } from '../index.js'
import { Wallet } from '@ethersproject/wallet'

const logError = debug('hopr:keypair')

/**
 * Serializes a peerId using geth's KeyStore format
 * see https://medium.com/@julien.maffre/what-is-an-ethereum-keystore-file-86c8c5917b97
 * @dev This method uses a computation and memory intensive hash function,
 *      for testing set `useWeakCrypto = true`
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

  const decoded = JSON.parse(serialized)

  // Following the Ethereum specification,
  // for privacy reasons keyStore files should
  // not include the Ethereum address
  delete decoded.address

  // Following the Ethereum specification,
  // the crypto property in keystore is lowercase
  Object.assign(decoded, {
    crypto: decoded.Crypto
  })

  // Removing property to follow Ethereum standard
  delete decoded.Crypto

  return new TextEncoder().encode(JSON.stringify(decoded))
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
 *      for testing set `useWeakCrypto = true`
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

  // Ethereum key are protected by iterating a hash function that is
  // hard in memory and computation very often which results in delays
  // of multiple seconds at startup and in unit tests.
  // Production keyStore must use strictly more than `1` iteration, hence
  // deserialization must fail when using low iterations in production
  if ((decoded.crypto.kdfparams.n == 1) != useWeakCrypto) {
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
    //logError(`Key deserialization failed,`, err)

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
