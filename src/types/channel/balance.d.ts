import BN from 'bn.js'
import Balance from '../balance'

declare interface ChannelBalanceStatic {
  readonly SIZE: number

  create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      balance: Balance | BN
      balance_a: Balance | BN
    }
  ): ChannelBalance
}
declare interface ChannelBalance {
  balance: Balance
  balance_a: Balance

  toU8a(): Uint8Array
}
declare var ChannelBalance: ChannelBalanceStatic

export default ChannelBalance
