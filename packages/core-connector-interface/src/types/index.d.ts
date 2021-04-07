import AcknowledgedTicket from './acknowledgedTicket'
import Ticket from './ticket'
import BN from 'bn.js'

declare interface AddressStatic {
  readonly SIZE: number
  new (arr: Uint8Array): Address
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
  new (address: Address, publicKey?: PublicKey, secret?: Hash, counter?: BN): AccountEntry
}
declare interface AccountEntry {
  address: Address
  publicKey?: PublicKey
  secret?: Hash
  counter?: BN
  isInitialized(): boolean
}
declare var AccountEntry: AccountEntryStatic

interface ChannelEntryVals {
  partyA: Address
  partyB: Address
  deposit: BN
  partyABalance: BN
  closureTime: BN
  stateCounter: BN
  closureByPartyA: boolean
  openedAt: BN
  closedAt: BN
}
declare interface ChannelEntryStatic {
  readonly SIZE: number
  new (...ChannelEntryVals): ChannelEntry
  fromObject(obj: ChannelEntryVals): ChannelEntry
}
declare interface ChannelEntry extends ChannelEntryVals {
  serialize(): Uint8Array
  getStatus(): 'CLOSED' | 'OPEN' | 'PENDING_TO_CLOSE'
  getIteration(): BN
  getId(): Promise<Hash>
  getBalances(): { partyA: Balance; partyB: Balance }
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

declare interface PublicKeyStatic {
  SIZE: number
  new (public: Uint8Array): PublicKey
  fromString(str: string): PublicKey
}

declare interface PublicKey {
  toAddress(): Address
  serialize(): Uint8Array
  toHex(): string
}

declare var PublicKey: PublicStatic

declare interface SignatureStatic {
  readonly SIZE: number
  deserialize(arr: Uint8Array): Signature
}
declare interface Signature {
  signature: Uint8Array
  recovery: number
  serialize(): Uint8Array
}
declare var Signature: SignatureStatic

export {
  AccountEntry,
  Address,
  AcknowledgedTicket,
  Balance,
  ChannelEntry,
  Hash,
  NativeBalance,
  PublicKey,
  Signature,
  Ticket,
  UINT256
}
