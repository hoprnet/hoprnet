import { BNE } from '../../types/extended'

class UINT256 extends BNE {
  toU8a() {
    return super.toU8a(UINT256.SIZE)
  }

  static get SIZE() {
    return 32
  }
}

export default UINT256
