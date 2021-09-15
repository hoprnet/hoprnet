// LIONESS implementation for packet payload based on the ChaCha stream cipher and the BLAKE2 hash algorithm

import { createCipheriv, createHmac } from 'crypto'
import { u8aXOR } from '../u8a'

const HASH_ALGORITHM = 'blake2s256'   
const CIPHER_ALGORITHM = 'chacha20'  

const INTERMEDIATE_KEY_LENGTH = 32
const INTERMEDIATE_IV_LENGTH = 16

const HASH_LENGTH = 32

export const PRP_KEY_LENGTH = 4 * INTERMEDIATE_KEY_LENGTH
export const PRP_IV_LENGTH = 4 * INTERMEDIATE_IV_LENGTH
export const PRP_MIN_LENGTH = HASH_LENGTH

// PRP-- Pseudo Random Permutation (stream cipher)
export type PRPParameters = {
  key: Uint8Array // key for the payload
  iv: Uint8Array // iv The initialization vector
  
// k1,k2,k3,k4 are Round keys
export class PRP {
  private readonly k1: Uint8Array
  private readonly k2: Uint8Array
  private readonly k3: Uint8Array
  private readonly k4: Uint8Array

  private readonly iv1: Uint8Array
  private readonly iv2: Uint8Array
  private readonly iv3: Uint8Array
  private readonly iv4: Uint8Array

  private constructor(iv: Uint8Array, key: Uint8Array) {
    this.k1 = key.subarray(0, INTERMEDIATE_KEY_LENGTH)
    this.k2 = key.subarray(INTERMEDIATE_KEY_LENGTH, 2 * INTERMEDIATE_KEY_LENGTH)
    this.k3 = key.subarray(2 * INTERMEDIATE_KEY_LENGTH, 3 * INTERMEDIATE_KEY_LENGTH)
    this.k4 = key.subarray(3 * INTERMEDIATE_KEY_LENGTH, 4 * INTERMEDIATE_KEY_LENGTH)

    this.iv1 = iv.subarray(0, INTERMEDIATE_IV_LENGTH)
    this.iv2 = iv.subarray(INTERMEDIATE_IV_LENGTH, 2 * INTERMEDIATE_IV_LENGTH)
    this.iv3 = iv.subarray(2 * INTERMEDIATE_IV_LENGTH, 3 * INTERMEDIATE_IV_LENGTH)
    this.iv4 = iv.subarray(3 * INTERMEDIATE_IV_LENGTH, 4 * INTERMEDIATE_IV_LENGTH)
  }

  static createPRP(params: PRPParameters): PRP {
    if (params.key.length != PRP_KEY_LENGTH) {
      throw Error(
        `Invalid key. Expected ${Uint8Array.name} of size ${PRP_KEY_LENGTH} bytes but got a ${typeof params.key} of ${
          params.key.length
        } bytes.`
      )
    }

    if (params.iv.length != PRP_IV_LENGTH) {
      throw Error(
        `Invalid initialisation vector. Expected ${
          Uint8Array.name
        } of size ${PRP_IV_LENGTH} bytes but got a ${typeof params.key} of ${params.key.length} bytes.`
      )
    }

    return new PRP(params.iv, params.key)
  }

  permutate(plaintext: Uint8Array): Uint8Array {
    if (plaintext.length < PRP_MIN_LENGTH) {
      throw Error(`Expected plaintext with a length of a least '${PRP_MIN_LENGTH}' bytes. Got '${plaintext.length}'.`)
    }

    const data = plaintext
    
  // k1 and k3 will be used to key the stream cipher, while k2 and k4 are used to key the hash function.
    encrypt(data, this.k1, this.iv1)
    hash(data, this.k2, this.iv2)
    encrypt(data, this.k3, this.iv3)
    hash(data, this.k4, this.iv4)

    return plaintext
  }

  inverse(ciphertext: Uint8Array): Uint8Array {
    if (ciphertext.length < PRP_MIN_LENGTH) {
      throw Error(`Expected ciphertext with a length of a least '${PRP_MIN_LENGTH}' bytes. Got '${ciphertext.length}'.`)
    }

    const data = ciphertext

    hash(data, this.k4, this.iv4)
    encrypt(data, this.k3, this.iv3)
    hash(data, this.k2, this.iv2)
    encrypt(data, this.k1, this.iv1)

    return data
  }
}

function hash(data: Uint8Array, k: Uint8Array, iv: Uint8Array): void {
  const hash = createHmac(HASH_ALGORITHM, Buffer.concat([k, iv], INTERMEDIATE_KEY_LENGTH + INTERMEDIATE_IV_LENGTH))
  hash.update(data.subarray(HASH_LENGTH))

  u8aXOR(true, data.subarray(0, HASH_LENGTH), hash.digest())
}

function encrypt(data: Uint8Array, k: Uint8Array, iv: Uint8Array): void {
  const cipher = createCipheriv(CIPHER_ALGORITHM, u8aXOR(false, k, data.subarray(0, HASH_LENGTH)), iv)

  const ciphertext = cipher.update(data.subarray(HASH_LENGTH))
  data.set(ciphertext, HASH_LENGTH)
}
