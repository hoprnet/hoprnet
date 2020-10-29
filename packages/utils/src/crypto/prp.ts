import {createCipheriv, createHmac} from 'crypto'
import {u8aXOR} from '../u8a'

const INTERMEDIATE_KEY_LENGTH = 32
const INTERMEDIATE_IV_LENGTH = 16

const HASH_LENGTH = 32
const KEY_LENGTH = 4 * INTERMEDIATE_KEY_LENGTH // 128 Bytes
const IV_LENGTH = 4 * INTERMEDIATE_IV_LENGTH // Bytes
const MIN_LENGTH = HASH_LENGTH // Bytes
const HASH_ALGORITHM = 'blake2s256'

const CIPHER_ALGORITHM = 'chacha20'

export class PRP {
  private readonly k1: Uint8Array
  private readonly k2: Uint8Array
  private readonly k3: Uint8Array
  private readonly k4: Uint8Array

  private readonly iv1: Uint8Array
  private readonly iv2: Uint8Array
  private readonly iv3: Uint8Array
  private readonly iv4: Uint8Array

  private initialised: boolean = false

  private constructor(key: Uint8Array, iv: Uint8Array) {
    if (key.length != KEY_LENGTH) {
      throw Error(
        `Invalid key. Expected ${Uint8Array.name} of size ${KEY_LENGTH} bytes but got a ${typeof key} of ${
          key.length
        } bytes.`
      )
    }

    if (iv.length != IV_LENGTH) {
      throw Error(
        `Invalid initialisation vector. Expected ${
          Uint8Array.name
        } of size ${IV_LENGTH} bytes but got a ${typeof key} of ${key.length} bytes..`
      )
    }

    this.k1 = key.subarray(0, INTERMEDIATE_KEY_LENGTH)
    this.k2 = key.subarray(INTERMEDIATE_KEY_LENGTH, 2 * INTERMEDIATE_KEY_LENGTH)
    this.k3 = key.subarray(2 * INTERMEDIATE_KEY_LENGTH, 3 * INTERMEDIATE_KEY_LENGTH)
    this.k4 = key.subarray(3 * INTERMEDIATE_KEY_LENGTH, 4 * INTERMEDIATE_KEY_LENGTH)

    this.iv1 = iv.subarray(0, INTERMEDIATE_IV_LENGTH)
    this.iv2 = iv.subarray(INTERMEDIATE_IV_LENGTH, 2 * INTERMEDIATE_IV_LENGTH)
    this.iv3 = iv.subarray(2 * INTERMEDIATE_IV_LENGTH, 3 * INTERMEDIATE_IV_LENGTH)
    this.iv4 = iv.subarray(3 * INTERMEDIATE_IV_LENGTH, 4 * INTERMEDIATE_IV_LENGTH)

    this.initialised = true
  }

  static get KEY_LENGTH(): number {
    return KEY_LENGTH
  }

  static get IV_LENGTH(): number {
    return IV_LENGTH
  }

  static get MIN_LENGTH(): number {
    return MIN_LENGTH
  }

  static createPRP(key: Uint8Array, iv: Uint8Array): PRP {
    return new PRP(key, iv)
  }

  permutate(plaintext: Uint8Array): Uint8Array {
    if (!this.initialised) {
      throw Error(`Uninitialised. Provide key and iv first.`)
    }

    if (plaintext.length < MIN_LENGTH) {
      throw Error(`Expected plaintext with a length of a least '${MIN_LENGTH}' bytes. Got '${plaintext.length}'.`)
    }

    const data = plaintext

    encrypt(data, this.k1, this.iv1)
    hash(data, this.k2, this.iv2)
    encrypt(data, this.k3, this.iv3)
    hash(data, this.k4, this.iv4)

    return plaintext
  }

  inverse(ciphertext: Uint8Array): Uint8Array {
    if (!this.initialised) {
      throw Error(`Uninitialised. Provide key and iv first.`)
    }

    if (ciphertext.length < MIN_LENGTH) {
      throw Error(`Expected ciphertext with a length of a least '${MIN_LENGTH}' bytes. Got '${ciphertext.length}'.`)
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
