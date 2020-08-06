import BN from 'bn.js'
import Balance from './balance'

declare namespace ChannelBalance {
  const SIZE: number

  function create(
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

export default ChannelBalance
