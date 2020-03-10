import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { ChannelBalance, Moment } from '.'
import { Uint8ArrayE } from '../types/extended'
import { typedClass } from '../tsc/utils'
import { u8aConcat } from '../core/u8a'

enum ChannelStatus {
  UNINITIALISED,
  FUNDING,
  OPEN,
  PENDING
}

@typedClass<TypeConstructors['Channel']>()
class Channel extends Uint8ArrayE {
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
      offset: balance.byteOffset
    })
  }

  get status(): ChannelStatus {
    const index = Number(this.subarray(ChannelBalance.SIZE, ChannelBalance.SIZE + 1)[0])

    if (index === 0) return ChannelStatus.UNINITIALISED
    else if (index === 1) return ChannelStatus.FUNDING
    else if (index === 2) return ChannelStatus.OPEN
    else if (index === 3) return ChannelStatus.PENDING
    else {
      throw Error("status like this doesn't exist")
    }
  }

  static get SIZE() {
    return ChannelBalance.SIZE + 1
  }

  static createFunded(balance: ChannelBalance) {
    return new Channel(undefined, {
      balance,
      status: ChannelStatus.FUNDING
    })
  }

  static createActive(balance: ChannelBalance): Channel {
    return new Channel(undefined, {
      balance,
      status: ChannelStatus.OPEN
    })
  }

  static createPending(moment: Moment, balance: ChannelBalance): Channel {
    return new Channel(undefined, {
      balance,
      status: ChannelStatus.PENDING,
      moment
    })
  }
}

export default Channel
