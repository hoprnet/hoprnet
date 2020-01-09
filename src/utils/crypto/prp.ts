import { createCipheriv, createHmac } from 'crypto'
import { u8aXOR } from '../u8a/xor'

const INTERMEDIATE_KEY_LENGTH = 32
const INTERMEDIATE_IV_LENGTH = 12

const HASH_LENGTH = 32
const KEY_LENGTH = 4 * INTERMEDIATE_KEY_LENGTH // 128 Bytes
const IV_LENGTH = 4 * INTERMEDIATE_IV_LENGTH // Bytes
const MIN_LENGTH = HASH_LENGTH // Bytes
const HASH_ALGORITHM = 'blake2b512'

const CIPHER_ALGORITHM = 'chacha20'

export class PRP {
  private readonly k1: Buffer
  private readonly k2: Buffer
  private readonly k3: Buffer
  private readonly k4: Buffer

  private readonly iv1: Buffer
  private readonly iv2: Buffer
  private readonly iv3: Buffer
  private readonly iv4: Buffer

  private initialised: boolean = false

  private constructor(key: Buffer, iv: Buffer) {
    if (key.length != KEY_LENGTH) throw Error('Invalid key. Expected Buffer of size ' + KEY_LENGTH + ' bytes.')

    if (iv.length != IV_LENGTH) throw Error('Invalid initialisation vector. Expected Buffer of size ' + IV_LENGTH + ' bytes.')

    this.k1 = key.slice(0, INTERMEDIATE_KEY_LENGTH)
    this.k2 = key.slice(INTERMEDIATE_KEY_LENGTH, 2 * INTERMEDIATE_KEY_LENGTH)
    this.k3 = key.slice(2 * INTERMEDIATE_KEY_LENGTH, 3 * INTERMEDIATE_KEY_LENGTH)
    this.k4 = key.slice(3 * INTERMEDIATE_KEY_LENGTH, 4 * INTERMEDIATE_KEY_LENGTH)

    this.iv1 = iv.slice(0, INTERMEDIATE_IV_LENGTH)
    this.iv2 = iv.slice(INTERMEDIATE_IV_LENGTH, 2 * INTERMEDIATE_IV_LENGTH)
    this.iv3 = iv.slice(2 * INTERMEDIATE_IV_LENGTH, 3 * INTERMEDIATE_IV_LENGTH)
    this.iv4 = iv.slice(3 * INTERMEDIATE_IV_LENGTH, 4 * INTERMEDIATE_IV_LENGTH)

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

  static createPRP(key: Buffer, iv: Buffer): PRP {
    return new PRP(key, iv)
  }

  permutate(plaintext: Buffer): Buffer {
    if (!this.initialised) throw Error(`Uninitialised. Provide key and iv first.`)

    if (plaintext.length < MIN_LENGTH) throw Error(`Expected plaintext with a length of a least '${MIN_LENGTH}' bytes. Got '${plaintext.length}'.`)

    const data = plaintext

    encrypt(data, this.k1, this.iv1)
    hash(data, this.k2, this.iv2)
    encrypt(data, this.k3, this.iv3)
    hash(data, this.k4, this.iv4)

    return data
  }

  inverse(ciphertext: Buffer): Buffer {
    if (!this.initialised) throw Error(`Uninitialised. Provide key and iv first.`)

    if (ciphertext.length < MIN_LENGTH) throw Error(`Expected ciphertext with a length of a least '${MIN_LENGTH}' bytes. Got '${ciphertext.length}'.`)

    const data = ciphertext

    hash(data, this.k4, this.iv4)
    encrypt(data, this.k3, this.iv3)
    hash(data, this.k2, this.iv2)
    encrypt(data, this.k1, this.iv1)

    return data
  }
}

function hash(data: Buffer, k: Buffer, iv: Buffer): void {
  const hash = createHmac(HASH_ALGORITHM, Buffer.concat([k, iv], INTERMEDIATE_KEY_LENGTH + INTERMEDIATE_IV_LENGTH))
  hash.update(data.slice(HASH_LENGTH))

  data.fill(u8aXOR(false, data.slice(0, HASH_LENGTH), hash.digest()), 0, HASH_LENGTH)
}

function encrypt(data: Buffer, k: Buffer, iv: Buffer): void {
  const cipher = createCipheriv(CIPHER_ALGORITHM, u8aXOR(false, k, data.slice(0, HASH_LENGTH)), iv)

  const ciphertext = cipher.update(data.slice(HASH_LENGTH))
  ciphertext.copy(data, HASH_LENGTH)
}
