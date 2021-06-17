// TODO - replace serialization with a library
import PeerId from 'peer-id'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import fs from 'fs'
import { resolve } from 'path'
import Debug from 'debug'
const log = Debug(`hoprd:identity`)

import Wallet from 'ethereumjs-wallet'

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
export async function deserializeKeyPair(serialized: Uint8Array, password: string, useWeakCrypto = false) {
  const decoded = JSON.parse(new TextDecoder().decode(serialized))

  if (decoded.crypto.kdfparams.n == 1 && useWeakCrypto != true) {
    throw Error(`Attempting to use a development key while not being in development mode`)
  }

  const w = await Wallet.fromV3(decoded, password)

  return privKeyToPeerId(w.getPrivateKey())
}

export type IdentityOptions = {
  initialize: boolean
  idPath: string
  password: string
  useWeakCrypto?: boolean
}

function loadIdentity(path: string): Uint8Array {
  return fs.readFileSync(resolve(path))
}

async function storeIdentity(path: string, id: Uint8Array) {
  fs.writeFileSync(resolve(path), id)
}

async function createIdentity(idPath: string, password: string, useWeakCrypto = false) {
  const peerId = await PeerId.create({ keyType: 'secp256k1' })
  const serializedKeyPair = await serializeKeyPair(peerId, password, useWeakCrypto)
  await storeIdentity(idPath, serializedKeyPair)
  return peerId
}

export async function getIdentity(options: IdentityOptions): Promise<PeerId> {
  if (typeof options.password !== 'string' || options.password.length == 0) {
    throw new Error(`Password must not be empty`)
  }

  let storedIdentity: Uint8Array | undefined
  try {
    storedIdentity = loadIdentity(options.idPath)
  } catch {
    log('Could not load identity', options.idPath)
  }

  if (options.useWeakCrypto) {
    log(`Using weaker key protection to accelerate node startup`)
  }
  if (storedIdentity != undefined) {
    return await deserializeKeyPair(storedIdentity, options.password, options.useWeakCrypto)
  }

  if (options.initialize) {
    log('Creating new identity', options.idPath)
    return await createIdentity(options.idPath, options.password, options.useWeakCrypto)
  }

  throw new Error('Cannot load identity')
}
