import AcknowledgedTicket from './acknowledgedTicket'
import { Channel, ChannelBalance, ChannelState } from './channel'
import ChannelEntry from './channelEntry'
import NativeBalance from './nativeBalance'
import Public from './public'
import Signature from './signature'
import SignedChannel from './signedChannel'
import SignedTicket from './signedTicket'
import Ticket from './ticket'

declare interface AddressStatic {
  readonly SIZE: number
  new (accountId: Uint8Array): Address
}
declare interface Address {
  serialize(): Uint8Array
  eq(b: Address): boolean
  toHex(): string
}
declare var Address: AddressStatic

declare interface BalanceStatic {
  readonly SIZE: number
  readonly SYMBOL: string // Abbreviation of the currency, e.g. `HOPR`
  readonly DECIMALS: number
  new (balance: BN): Balance
}
declare interface Balance {
  toBN(): BN
  serialize(): Uint8Array
  toFormattedString(): string // Readable version of the balance
}
declare var Balance: BalanceStatic

declare interface HashStatic {
  readonly SIZE: number
  new (hash: Uint8Array): Hash
}
declare interface Hash {
  serialize(): Uint8Array
  toHex(): string
  eq(b: Hash): boolean
  hash(): Hash
}
declare var Hash: HashStatic

export {
  Address,
  AcknowledgedTicket,
  Balance,
  Channel,
  ChannelBalance,
  ChannelState,
  ChannelEntry,
  Hash,
  NativeBalance,
  Public,
  Signature,
  SignedChannel,
  SignedTicket,
  Ticket,
  UINT256
}
