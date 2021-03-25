import AcknowledgedTicket from './acknowledgedTicket'
import { Channel, ChannelBalance, ChannelState } from './channel'
import Hash from './hash'
import Moment from './moment'
import NativeBalance from './nativeBalance'
import Public from './public'
import Signature from './signature'
import SignedChannel from './signedChannel'
import SignedTicket from './signedTicket'
import Ticket from './ticket'
import TicketEpoch from './ticketEpoch'

declare interface AddressStatic {
  readonly SIZE: number
  new (accountId: Uint8Array): Address
  fromString(str: string): Address
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

declare interface AccountEntryStatic {
  readonly SIZE: number
  new (address: Address, publicKey?: Public, secret?: Hash, counter?: BN): Account
}
declare interface AccountEntry {
  address: Address
  publicKey?: Public
  secret?: Hash
  counter?: BN
  isInitialized(): boolean
}
declare var AccountEntry: AccountEntryStatic

declare interface ChannelEntryStatic {
  readonly SIZE: number
}
declare interface ChannelEntry {
  parties: [Address, Address]
  deposit: BN
  partyABalance: BN
  closureTime: BN
  stateCounter: BN
  closureByPartyA: boolean
  openedAt: BN
  closedAt: BN
  getStatus(): 'CLOSED' | 'OPEN' | 'PENDING_TO_CLOSE'
  getIteration(): number
}
declare var ChannelEntry: ChannelEntryStatic

export {
  AccountEntry,
  Address,
  AcknowledgedTicket,
  Balance,
  Channel,
  ChannelBalance,
  ChannelState,
  ChannelEntry,
  Hash,
  Moment,
  NativeBalance,
  Public,
  Signature,
  SignedChannel,
  SignedTicket,
  Ticket,
  TicketEpoch
}
