import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import BN from 'bn.js'

class Balance implements Types.Balance {
  constructor(private bn: BN){}

  static get SYMBOL(): string {
    return `HOPR`
  }

  static get DECIMALS(): number {
    return 18
  }

  public toBN(): BN {
    return this.bn
  }

  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', 32))
  }

  public toFormattedString(): string {
    return moveDecimalPoint(this.bn.toString(), Balance.DECIMALS * -1) + ' ' + Balance.SYMBOL
  }

  static get SIZE(): number {
    // Uint256
    return 32
  }
}

export default Balance
