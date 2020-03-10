import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { Uint8ArrayE } from '../types/extended'
import { typedClass } from '../tsc/utils'
import { u8aConcat } from '../core/u8a'
import Balance from './Balance'

export type ChannelBalanceConstructorArguments = [
  any,
  {
    balance: Balance
    balance_a: Balance
  }
]

@typedClass<TypeConstructors['ChannelBalance']>()
class ChannelBalance extends Uint8ArrayE {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      balance: Balance
      balance_a: Balance
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, ChannelBalance.SIZE)
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.balance.toU8a(), struct.balance_a.toU8a()))
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get balance(): Balance {
    return new Balance(this.subarray(0, Balance.SIZE))
  }

  get balance_a(): Balance {
    return new Balance(this.subarray(Balance.SIZE, Balance.SIZE + Balance.SIZE))
  }

  static get SIZE() {
    return Balance.SIZE + Balance.SIZE
  }
}

export default ChannelBalance
