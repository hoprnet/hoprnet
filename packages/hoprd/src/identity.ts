// TODO - replace serialization with a library
import PeerId from 'peer-id'
import { privKeyToPeerId, SECP256K1_CONSTANTS, stringToU8a } from '@hoprnet/hopr-utils'
import fs from 'fs'
import { resolve } from 'path'
import { debug } from '@hoprnet/hopr-utils'
import Wallet from 'ethereumjs-wallet'

const log = debug(`hoprd:identity`)

export enum IdentityErrors {
  FAIL_TO_LOAD_IDENTITY = 'Could not load identity',
  EMPTY_PASSWORD = 'Password must not be empty',
  WRONG_USAGE_OF_WEAK_CRYPTO = 'Attempting to use a development key while not being in development mode',
  WRONG_PASSPHRASE = 'Key derivation failed - possibly wrong passphrase',
  INVALID_PRIVATE_KEY_GIVEN = 'Invalid private key was given',
  INVALID_SECPK256K1_PRIVATE_KEY_GIVEN = 'The key given was not at least 32 bytes long'
}

export enum IdentityLogs {
  USING_WEAK_CRYPTO = 'Using weaker key protection to accelerate node startup'
}

/**
 * Serializes a given peerId by serializing the included private key and public key.
 *
 * @param peerId the peerId that should be serialized
 */
export async function serializeKeyPair(peerId: PeerId, password: string, useWeakCrypto = false): Promise<Uint8Array> {
  const w = new Wallet(peerId.privKey.marshal() as Buffer)

  let serialized: string
  if (useWeakCrypto) {
    // Use weak settings during development for quicker
    // node startup
    serialized = await w.toV3String(password, {
      n: 1
    })
  } else {
    serialized = await w.toV3String(password)
  }

  return new TextEncoder().encode(serialized)
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
export async function deserializeKeyPair(
  serialized: Uint8Array,
  password: string,
  useWeakCrypto = false
): Promise<PeerId> {
  const decoded = JSON.parse(new TextDecoder().decode(serialized))

  if (decoded.crypto.kdfparams.n == 1 && useWeakCrypto != true) {
    throw Error(IdentityErrors.WRONG_USAGE_OF_WEAK_CRYPTO)
  }

  const w = await Wallet.fromV3(decoded, password)

  return privKeyToPeerId(w.getPrivateKey())
}

export type IdentityOptions = {
  initialize: boolean
  idPath: string
  password: string
  useWeakCrypto?: boolean
  privateKey?: string
}

function loadIdentity(path: string): Uint8Array {
  return fs.readFileSync(resolve(path))
}

async function storeIdentity(path: string, id: Uint8Array) {
  fs.writeFileSync(resolve(path), id)
}

async function createIdentity(idPath: string, password: string, useWeakCrypto = false, privateKey?: Uint8Array) {
  const peerId = privateKey ? privKeyToPeerId(privateKey) : await PeerId.create({ keyType: 'secp256k1' })
  const serializedKeyPair = await serializeKeyPair(peerId, password, useWeakCrypto)
  await storeIdentity(idPath, serializedKeyPair)
  return peerId
}

export async function getIdentity(options: IdentityOptions): Promise<PeerId> {
  let privateKey: Uint8Array | undefined
  if (options.privateKey) {
    if (isNaN(parseInt(options.privateKey, 16))) {
      throw new Error(IdentityErrors.INVALID_PRIVATE_KEY_GIVEN)
    }
    privateKey = stringToU8a(options.privateKey)
    if (privateKey.length != SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH) {
      throw new Error(IdentityErrors.INVALID_SECPK256K1_PRIVATE_KEY_GIVEN)
    }
    return await createIdentity(options.idPath, options.password, options.useWeakCrypto, privateKey)
  }
  if (typeof options.password !== 'string' || options.password.length == 0) {
    throw new Error(IdentityErrors.EMPTY_PASSWORD)
  }

  let storedIdentity: Uint8Array | undefined
  try {
    storedIdentity = loadIdentity(options.idPath)
  } catch {
    log(IdentityErrors.FAIL_TO_LOAD_IDENTITY, options.idPath)
  }

  if (options.useWeakCrypto) {
    log(IdentityLogs.USING_WEAK_CRYPTO)
  }
  if (storedIdentity != undefined) {
    return await deserializeKeyPair(storedIdentity, options.password, options.useWeakCrypto)
  }

  if (options.initialize) {
    log('Creating new identity', options.idPath)
    return await createIdentity(options.idPath, options.password, options.useWeakCrypto)
  }

  throw new Error(IdentityErrors.FAIL_TO_LOAD_IDENTITY)
}
