import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { Moment } from '..'
import { hash, stateCounterToStatus, sign } from '../../utils'
import { serializeToU8a } from '@hoprnet/hopr-utils'
import { u8aToNumber, toU8a } from '@hoprnet/hopr-utils'
import Balance from '../balance'

enum ChannelStatus {
  UNINITIALISED,
  FUNDED,
  OPEN,
  PENDING
}

class Channel implements Types.Channel {
  constructor(
    readonly balance: Balance,
    readonly balance_a: Balance,
    readonly state: number,
    readonly moment?: Moment) {}

  static deserialize() {
    const state = u8aToNumber(state)
  }

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.balance.toU8a(), Balance.SIZE],
      [this.balance_a.toU8a(), Balance.SIZE],
      [toU8a(this.state, 1), 1]
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
    return new Channel(balance, new ChannelState(ChannelStatus.FUNDED))
  }

  static createActive(balance: ChannelBalance): Channel {
    return new Channel(balance, new ChannelState(ChannelStatus.OPEN))
  }

  static createPending(moment: Moment, balance: ChannelBalance): Channel {
    return new Channel(balance, new ChannelState(ChannelStatus.PENDING), moment)
  }
}

export { Channel, ChannelBalance, ChannelState, ChannelStatus }
