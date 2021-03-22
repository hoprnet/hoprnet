import { Types } from '@hoprnet/hopr-core-connector-interface'
import { Moment } from '..'
import { hash,  sign } from '../../utils'
import { u8aToNumber, toU8a, u8aSlice, serializeToU8a } from '@hoprnet/hopr-utils'
import Balance from '../balance'

class ChannelState implements Types.ChannelState {
  constructor(
    readonly balance: Balance,
    readonly balance_a: Balance,
    readonly status: Types.ChannelStatus,
    readonly moment?: Moment) {}

  static deserialize(arr: Uint8Array) {
    const [a, b, c] = u8aSlice(arr, [Balance.SIZE, Balance.SIZE, 1])
    const balance = new Balance(a)
    const balance_a = new Balance(b)
    const state = u8aToNumber(c)
    return new ChannelState(balance, balance_a, state)
  }

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.balance.toU8a(), Balance.SIZE],
      [this.balance_a.toU8a(), Balance.SIZE],
      [toU8a(this.status, 1), 1]
    ])
  }

  async hash() {
    return hash(this.serialize())
  }

  async sign(privKey: Uint8Array): Promise<Types.Signature> {
    return await sign(await this.hash(), privKey)
  }

  get isFunded(): boolean {
    return this.status == Types.ChannelStatus.FUNDED
  }

  get isActive(): boolean {
    return this.status == Types.ChannelStatus.OPEN
  }

  get isPending(): boolean {
    return this.status == Types.ChannelStatus.PENDING
  }

  static get SIZE(): number {
    return Balance.SIZE + Balance.SIZE + 1 
  }

  static createFunded(balance: Balance, balance_a: Balance): ChannelState {
    return new ChannelState(balance, balance_a, Types.ChannelStatus.FUNDED)
  }

  static createActive(balance: Balance, balance_a: Balance): ChannelState {
    return new ChannelState(balance, balance_a, ChannelStatus.OPEN)
  }

  static createPending(moment: Moment, balance: Balance, balance_a: Balance): ChannelState {
    return new ChannelState(balance, balance_a, ChannelStatus.PENDING, moment)
  }
}

export { ChannelState, ChannelStatus }
