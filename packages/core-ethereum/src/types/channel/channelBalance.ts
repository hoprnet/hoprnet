import BN from 'bn.js'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { Uint8ArrayE } from '../extended'
import Balance from '../balance'

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
    if (arr != null) {
      super(arr.bytes, arr.offset, ChannelBalance.SIZE)
    } else {
      super(ChannelBalance.SIZE)
    }

    if (struct != null) {
      if (struct.balance_a != null) {
        this.set(new Balance(struct.balance_a.toString()).toU8a(), this.balanceAOffset - this.byteOffset)
      }

      if (struct.balance != null) {
        this.set(new Balance(struct.balance.toString()).toU8a(), this.balanceOffset - this.byteOffset)
      }
    }
  }

  get balanceOffset(): number {
    return this.byteOffset
  }
  get balance(): Balance {
    return new Balance(new Uint8Array(this.buffer, this.balanceOffset, Balance.SIZE))
  }

  get balanceAOffset(): number {
    return this.byteOffset + Balance.SIZE
  }

  get balance_a(): Balance {
    return new Balance(new Uint8Array(this.buffer, this.balanceAOffset, Balance.SIZE))
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
