import BN from 'bn.js'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { Uint8ArrayE } from '../extended'
import { Balance } from '..'

class ChannelBalance extends Uint8ArrayE implements Types.ChannelBalance {
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
    if (arr) {
      super(arr.bytes, arr.offset, ChannelBalance.SIZE)
    } else {
      super(ChannelBalance.SIZE)
    }

    if (struct) {
      if (struct.balance_a) {
        this.set(struct.balance_a.serialize(), this.balanceAOffset - this.byteOffset)
      }

      if (struct.balance) {
        this.set(struct.balance.serialize(), this.balanceOffset - this.byteOffset)
      }
    }
  }

  slice(begin = 0, end = ChannelBalance.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = ChannelBalance.SIZE) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get balanceOffset(): number {
    return this.byteOffset
  }
  get balance(): Balance {
    return new Balance(new BN(new Uint8Array(this.buffer, this.balanceOffset, Balance.SIZE)))
  }

  get balanceAOffset(): number {
    return this.byteOffset + Balance.SIZE
  }

  get balance_a(): Balance {
    return new Balance(new BN(new Uint8Array(this.buffer, this.balanceAOffset, Balance.SIZE)))
  }

  static get SIZE(): number {
    return Balance.SIZE + Balance.SIZE
  }

  static create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      balance: Balance
      balance_a: Balance
    }
  ): ChannelBalance {
    return new ChannelBalance(arr, struct)
  }
}

export default ChannelBalance
