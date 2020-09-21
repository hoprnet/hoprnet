import { Uint8ArrayE } from '../../types/extended'
import { HASH_LENGTH } from '../../constants'

// @TODO: SIZE check on construction
class BYTES32 extends Uint8ArrayE {
  static get SIZE(): number {
    return HASH_LENGTH
  }
}

export default BYTES32
