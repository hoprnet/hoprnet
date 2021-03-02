import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { UINT256 } from './solidity'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'

class Balance extends UINT256 implements Types.Balance {
  static get SYMBOL(): string {
    return `xHOPR`
  }

  static get DECIMALS(): number {
    return 18
  }

  public toFormattedString(): string {
    return moveDecimalPoint(this.toString(), Balance.DECIMALS * -1) + ' ' + Balance.SYMBOL
  }
}

export default Balance
