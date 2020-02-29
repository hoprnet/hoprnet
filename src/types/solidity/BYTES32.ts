import { Uint8Array } from 'src/types/extended'
import { HASH_LENGTH } from 'src/constants'

// TODO: SIZE check on construction
class BYTES32 extends Uint8Array {
  static get SIZE() {
    return HASH_LENGTH
  }
}

export default BYTES32
