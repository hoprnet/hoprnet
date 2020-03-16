import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { UINT256 } from './solidity'

class Balance extends UINT256 implements Types.Balance {
  static get SYMBOL(): string {
    return `HOPR`
  }

  static get DECIMALS(): number {
    return 18
  }
}

export default Balance
