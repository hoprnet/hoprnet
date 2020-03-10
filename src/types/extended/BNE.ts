import BN from 'bn.js'
import { toU8a } from '../../core/u8a'

class BNE extends BN {
  toU8a() {
    return toU8a(this.toNumber())
  }
}

export default BNE
