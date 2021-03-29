import AcknowledgedTicket from './acknowledgedTicket'
import { Channel, ChannelBalance, ChannelState } from './channel'
import Hash from './hash'
import Public from './public'
import Signature from './signature'
import SignedChannel from './signedChannel'
import SignedTicket from './signedTicket'
import Ticket from './ticket'
import BN from 'bn.js'

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

declare interface NativeBalanceStatic {
  readonly SIZE: number
  readonly SYMBOL: string // Abbreviation of the currency, e.g. `ETH`
  readonly DECIMALS: number
  new (balance: BN): Balance
}
declare interface NativeBalance {
  toBN(): BN
  serialize(): Uint8Array
  toFormattedString(): string // Readable version of the balance
}
declare var NativeBalance: NativeBalanceStatic

declare interface AccountEntryStatic {
  readonly SIZE: number
  new (address: Address, publicKey?: Public, secret?: Hash, counter?: BN): AccountEntry
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
  partyA: Address
  partyB: Address
  deposit: BN
  partyABalance: BN
  closureTime: BN
  stateCounter: BN
  closureByPartyA: boolean
  openedAt: BN
  closedAt: BN
  getStatus(): 'CLOSED' | 'OPEN' | 'PENDING_TO_CLOSE'
  getIteration(): number
  getChannelId(): Promise<Hash>
}
declare var ChannelEntry: ChannelEntryStatic

declare interface UINT256Static {
  readonly SIZE: number
  new (amount: BN): UINT256
  fromString(str: string): UINT256
}
declare interface UINT256 {
  toBN(): BN
  serialize(): Uint8Array
}
declare var UINT256: UINT256Static

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
  NativeBalance,
  Public,
  Signature,
  SignedChannel,
  SignedTicket,
  Ticket,
  UINT256
}
