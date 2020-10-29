import { createCipheriv } from 'crypto'
import { u8aConcat, toU8a } from '../u8a'

const BLOCK_LENGTH = 16
const KEY_LENGTH = BLOCK_LENGTH
const IV_LENGTH = 12
const COUNTER_LENGTH = 4

const PRG_ALGORITHM = 'aes-128-ctr'

export class PRG {
  private readonly key: Uint8Array
  private readonly iv: Uint8Array

  private initialised: boolean = false

  private constructor(key: Uint8Array, iv: Uint8Array) {
    this.key = key
    this.iv = iv

    this.initialised = true
  }

  static get IV_LENGTH(): number {
    return IV_LENGTH
  }

  static get KEY_LENGTH(): number {
    return KEY_LENGTH
  }

  static createPRG(key: Uint8Array, iv: Uint8Array): PRG {
    if (key.length != KEY_LENGTH || iv.length != IV_LENGTH) {
      throw Error(
        `Invalid input parameters. Expected a key of ${KEY_LENGTH} bytes and an initialization vector of ${IV_LENGTH} bytes.`
      )
    }

    return new PRG(key, iv)
  }

  digest(start: number, end: number): Uint8Array {
    if (!this.initialised) {
      throw Error(`Module not initialized. Please do that first.`)
    }

    if (start >= end) {
      throw Error(`Invalid range parameters. 'start' must be strictly smaller than 'end'.`)
    }

    const firstBlock = Math.floor(start / BLOCK_LENGTH)
    const startOffset = start % BLOCK_LENGTH

    const lastBlock = Math.ceil(end / BLOCK_LENGTH)
    const lastBlockSize = end % BLOCK_LENGTH

    const amountOfBlocks = lastBlock - firstBlock

    const iv = u8aConcat(this.iv, toU8a(firstBlock, COUNTER_LENGTH))

    return createCipheriv(PRG_ALGORITHM, this.key, iv)
      .update(new Uint8Array(amountOfBlocks * BLOCK_LENGTH))
      .subarray(startOffset, amountOfBlocks * BLOCK_LENGTH - (lastBlockSize > 0 ? BLOCK_LENGTH - lastBlockSize : 0))
  }
}
