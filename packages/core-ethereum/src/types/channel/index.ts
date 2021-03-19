import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { Moment } from '..'
import { hash, stateCounterToStatus, sign } from '../../utils'
import ChannelState from './channelState'
import ChannelBalance from './channelBalance'
import { serializeToU8a } from '@hoprnet/hopr-utils'

enum ChannelStatus {
  UNINITIALISED,
  FUNDED,
  OPEN,
  PENDING
}

class Channel implements Types.Channel {
  constructor(
    readonly balance: ChannelBalance,
    readonly state: ChannelState,
    readonly moment?: Moment
  ) {}

  static deserialize(){}

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.balance.toU8a(), ChannelBalance.SIZE],
      [this.state, ChannelState.SIZE]
    ])

  }

  async hash() {
    return hash(this.serialize())
  }

  async sign(privKey: Uint8Array): Promise<Types.Signature> {
    return await sign(await this.hash(), privKey)
  }

  get isFunded(): boolean {
    return stateCounterToStatus(this.state.toNumber()) == ChannelStatus.FUNDED
  }

  get isActive(): boolean {
    return stateCounterToStatus(this.state.toNumber()) == ChannelStatus.OPEN
  }

  get isPending(): boolean {
    return stateCounterToStatus(this.state.toNumber()) == ChannelStatus.PENDING
  }

  static get SIZE(): number {
    return ChannelBalance.SIZE + ChannelState.SIZE
  }

  static createFunded(balance: ChannelBalance): Channel {
    return new Channel(
      balance,
      new ChannelState(ChannelStatus.FUNDED)
    )
  }

  static createActive(balance: ChannelBalance): Channel {
    return new Channel(
      balance,
      new ChannelState(ChannelStatus.OPEN)
    )
  }

  static createPending(moment: Moment, balance: ChannelBalance): Channel {
    return new Channel(
      balance,
      new ChannelState(ChannelStatus.PENDING),
      moment
    )
  }
}

export { Channel, ChannelBalance, ChannelState, ChannelStatus }
