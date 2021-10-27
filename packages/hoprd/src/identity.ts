// TODO - replace serialization with a library
import PeerId from 'peer-id'
import {
  privKeyToPeerId,
  SECP256K1_CONSTANTS,
  stringToU8a,
  serializeKeyPair,
  deserializeKeyPair
} from '@hoprnet/hopr-utils'
import fs from 'fs'
import { resolve } from 'path'
import { debug } from '@hoprnet/hopr-utils'
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
    const deserialized = await deserializeKeyPair(storedIdentity, options.password, options.useWeakCrypto)

    if (deserialized.success == false) {
      if (deserialized.error === 'Invalid password') {
        throw new Error(IdentityErrors.WRONG_PASSPHRASE)
      } else if (deserialized.error === 'Wrong usage of weak crypto') {
        throw new Error(IdentityErrors.WRONG_USAGE_OF_WEAK_CRYPTO)
      } else {
        throw new Error(`Unknown identity error ${deserialized.error}`)
      }
    } else {
      return deserialized.identity
    }
  }

  if (options.initialize) {
    log('Creating new identity', options.idPath)
    return await createIdentity(options.idPath, options.password, options.useWeakCrypto)
  }

  throw new Error(IdentityErrors.FAIL_TO_LOAD_IDENTITY)
}
