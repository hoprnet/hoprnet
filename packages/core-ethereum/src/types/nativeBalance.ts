import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { UINT256 } from './solidity'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'

class NativeBalance extends UINT256 implements Types.NativeBalance {
  static get SYMBOL(): string {
    return `gETH`
  }

  static get DECIMALS(): number {
    return 18
  }

  public toFormattedString(): string {
    return moveDecimalPoint(this.toString(), NativeBalance.DECIMALS * -1) + ' ' + NativeBalance.SYMBOL
  }
}

export default NativeBalance
