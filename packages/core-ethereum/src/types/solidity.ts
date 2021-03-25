import { BNE } from '../types/extended'

class UINT256 extends BNE {
  toU8a(): Uint8Array {
    return super.toU8a(UINT256.SIZE)
  }

  static get SIZE(): number {
    return 32
  }
}

export { UINT256 }

