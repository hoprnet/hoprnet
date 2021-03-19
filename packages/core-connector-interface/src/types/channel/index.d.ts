import { Moment, Hash, Signature } from '..'
import Balance from '../balance'

declare interface ChannelStatic {
  createFunded(balance: Balance, balance_a: Balance): Channel
  createActive(balance: Balance, balance_a: Balance): Channel
  createPending(pending: Moment, balance: Balance, balance_a: Balance): Channel
  deserialize(arr: Uint8Array): Channel
  SIZE: number
}

declare interface ChannelState {
  sign(privKey: Uint8Array, pubKey: Uint8Array | undefined): Promise<Signature>
  balance: Balance
  balance_a: Balance
  pending?: Moment
  isFunded: boolean
  isActive: boolean
  isPending: boolean
  status: number
  hash(): Promise<Hash>
  serialize(): Uint8Array
}

declare var Channel: ChannelStatic

export { Channel, ChannelBalance, ChannelState }
