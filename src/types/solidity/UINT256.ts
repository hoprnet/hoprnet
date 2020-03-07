import { BNE } from 'src/types/extended'

// TODO: SIZE check on construction
class UINT265 extends BNE {
  static get SIZE() {
    return 32
  }
}

export default UINT265
