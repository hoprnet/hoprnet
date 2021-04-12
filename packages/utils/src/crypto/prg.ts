import { createCipheriv } from 'crypto'
import { toU8a } from '../u8a'

const BLOCK_LENGTH = 16
export const PRG_KEY_LENGTH = BLOCK_LENGTH
export const PRG_IV_LENGTH = 12
export const PRG_COUNTER_LENGTH = 4

const PRG_ALGORITHM = 'aes-128-ctr'

export type PRGParameters = {
  key: Uint8Array
  iv: Uint8Array
}

export class PRG {
  private readonly key: Uint8Array
  private readonly iv: Uint8Array

  private constructor(key: Uint8Array, iv: Uint8Array) {
    this.key = key
    this.iv = iv
  }

  static createPRG(params: PRGParameters): PRG {
    if (params.key.length != PRG_KEY_LENGTH || params.iv.length != PRG_IV_LENGTH) {
      throw Error(
        `Invalid input parameters. Expected a key of ${PRG_KEY_LENGTH} bytes and an initialization vector of ${PRG_IV_LENGTH} bytes.`
      )
    }

    return new PRG(params.key, params.iv)
  }

  digest(start: number, end: number): Uint8Array {
    if (start >= end) {
      throw Error(`Invalid range parameters. 'start' must be strictly smaller than 'end'.`)
    }

    const firstBlock = Math.floor(start / BLOCK_LENGTH)
    const startOffset = start % BLOCK_LENGTH

    const lastBlock = Math.ceil(end / BLOCK_LENGTH)
    const lastBlockSize = end % BLOCK_LENGTH

    const amountOfBlocks = lastBlock - firstBlock

    const iv = Uint8Array.from([...this.iv, ...toU8a(firstBlock, PRG_COUNTER_LENGTH)])

    return createCipheriv(PRG_ALGORITHM, this.key, iv)
      .update(new Uint8Array(amountOfBlocks * BLOCK_LENGTH))
      .subarray(startOffset, amountOfBlocks * BLOCK_LENGTH - (lastBlockSize > 0 ? BLOCK_LENGTH - lastBlockSize : 0))
  }
}
