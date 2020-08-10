import { BYTES32 } from './solidity'
import { COMPRESSED_PUBLIC_KEY_LENGTH } from '../constants'

class Public extends BYTES32 {
  static get SIZE() {
    return COMPRESSED_PUBLIC_KEY_LENGTH
  }
}

export default Public
