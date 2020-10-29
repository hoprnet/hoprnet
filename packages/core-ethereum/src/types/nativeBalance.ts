import type {Types} from '@hoprnet/hopr-core-connector-interface'
import {UINT256} from './solidity'

class NativeBalance extends UINT256 implements Types.NativeBalance {
  static get SYMBOL(): string {
    return `ETH`
  }

  static get DECIMALS(): number {
    return 18
  }
}

export default NativeBalance
