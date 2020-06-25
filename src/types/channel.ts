import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aConcat } from '@hoprnet/hopr-utils'
import { ChannelBalance, Moment } from '.'
import { Uint8ArrayE } from '../types/extended'
import { hash, stateCountToStatus, sign } from '../utils'
import { Signature } from '@hoprnet/hopr-core-connector-interface/src/types'

export enum ChannelStatus {
  UNINITIALISED,
  FUNDING,
  OPEN,
  PENDING,
}

class Channel extends Uint8ArrayE implements Types.Channel {
  moment?: Moment

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      balance: ChannelBalance
      status: ChannelStatus
      moment?: Moment
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, Channel.SIZE)
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.balance.toU8a(), new Uint8Array([struct.status])))
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get balance(): ChannelBalance {
    const balance = this.subarray(0, ChannelBalance.SIZE)
    return new ChannelBalance({
      bytes: balance.buffer,
      offset: balance.byteOffset,
    })
  }

  get stateCounter(): number {
    return Number(this.subarray(ChannelBalance.SIZE, ChannelBalance.SIZE + 1)[0])
  }

  get status(): ChannelStatus {
    return stateCountToStatus(this.stateCounter)
  }

  get hash() {
    return hash(this)
  }

  async sign(
    privKey: Uint8Array,
    pubKey: Uint8Array,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<Signature> {
    return await sign(await this.hash, privKey, undefined, arr)
  }

  static get SIZE() {
    return ChannelBalance.SIZE + 1
  }

  static createFunded(balance: ChannelBalance) {
    return new Channel(undefined, {
      balance,
      status: ChannelStatus.FUNDING,
    })
  }

  static createActive(balance: ChannelBalance): Channel {
    return new Channel(undefined, {
      balance,
      status: ChannelStatus.OPEN,
    })
  }

  static createPending(moment: Moment, balance: ChannelBalance): Channel {
    return new Channel(undefined, {
      balance,
      status: ChannelStatus.PENDING,
      moment,
    })
  }
}

export default Channel
