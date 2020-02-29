import BN from 'bn.js'
import { toU8a } from 'src/core/u8a'

class BN_U8a extends BN {
  toU8a() {
    return toU8a(this.toNumber())
  }
}

export default BN_U8a
