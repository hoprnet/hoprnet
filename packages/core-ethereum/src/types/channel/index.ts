import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { Moment } from '..'
import { Uint8ArrayE } from '../extended'
import { hash, stateCounterToStatus, sign } from '../../utils'
import ChannelState from './channelState'
import ChannelBalance from './channelBalance'

enum ChannelStatus {
  CLOSED,
  OPEN,
  PENDING_TO_CLOSE
}

class Channel extends Uint8ArrayE implements Types.Channel {
  private _rawState?: ChannelState

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      state: ChannelState
      balance?: ChannelBalance
      moment?: Moment
    }
  ) {
    if (!arr) {
      super(Channel.SIZE)
    } else {
      super(arr.bytes, arr.offset, Channel.SIZE)
    }

    if (struct) {
      this.set(struct.state, ChannelBalance.SIZE)

      if (struct.balance) {
        this.set(struct.balance.toU8a(), 0)
      }
    }
  }

  // @TODO fix SIZE
  slice(begin = 0, end = Channel.SIZE) {
    return this.subarray(begin, end)
  }

  // @TODO fix SIZE
  subarray(begin = 0, end = Channel.SIZE) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get balance(): ChannelBalance {
    const balance = this.subarray(0, ChannelBalance.SIZE)
    return new ChannelBalance({
      bytes: balance.buffer,
      offset: balance.byteOffset
    })
  }

  get rawState(): ChannelState {
    if (!this._rawState) {
      this._rawState = new ChannelState({
        bytes: this.buffer,
        offset: this.byteOffset + ChannelBalance.SIZE
      })
    }

    return this._rawState
  }

  get moment(): Moment | void {
    if (this._status != ChannelStatus.PENDING_TO_CLOSE) {
      return
    }

    return new Moment(this.subarray(ChannelBalance.SIZE + 1, ChannelBalance.SIZE + 1 + Moment.SIZE))
  }

  get _status(): ChannelStatus {
    return stateCounterToStatus(this.rawState.toNumber())
  }

  get hash() {
    return hash(this)
  }

  async sign(privKey: Uint8Array): Promise<Types.Signature> {
    return await sign(await this.hash, privKey)
  }

  get isActive(): boolean {
    return this._status == ChannelStatus.OPEN
  }

  get isPending(): boolean {
    return this._status == ChannelStatus.PENDING_TO_CLOSE
  }

  // @TODO fix size
  static get SIZE(): number {
    return ChannelBalance.SIZE + ChannelState.SIZE
  }

  static createActive(balance: ChannelBalance): Channel {
    return new Channel(undefined, {
      balance,
      state: new ChannelState(undefined, { state: ChannelStatus.OPEN })
    })
  }

  static createPending(moment: Moment, balance: ChannelBalance): Channel {
    return new Channel(undefined, {
      balance,
      state: new ChannelState(undefined, { state: ChannelStatus.PENDING_TO_CLOSE }),
      moment
    })
  }
}

export { Channel, ChannelBalance, ChannelState, ChannelStatus }
