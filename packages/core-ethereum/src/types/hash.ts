import type {Types} from '@hoprnet/hopr-core-connector-interface'
import {Uint8ArrayE} from './extended'
import {HASH_LENGTH} from '../constants'

class Hash extends Uint8ArrayE implements Types.Hash {
  slice(begin = 0, end = Hash.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = Hash.SIZE) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  static get SIZE() {
    return HASH_LENGTH
  }
}

export default Hash
