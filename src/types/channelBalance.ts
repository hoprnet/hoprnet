import BN from "bn.js"
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aConcat } from '@hoprnet/hopr-utils'
import { Uint8ArrayE } from '../types/extended'
import Balance from './balance'

class ChannelBalance extends Uint8ArrayE implements Types.ChannelBalance {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      balance: BN | Balance
      balance_a: BN | Balance
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, ChannelBalance.SIZE)
    } else if (arr == null && struct != null) {
      super(u8aConcat(new Balance(struct.balance.toString()).toU8a(), new Balance(struct.balance_a.toString()).toU8a()))
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

  static create(    arr?: {
    bytes: ArrayBuffer
    offset: number
  },
  struct?: {
    balance: Balance
    balance_a: Balance
  }) {
    return new ChannelBalance(arr, struct)
  }
}

export default ChannelBalance
