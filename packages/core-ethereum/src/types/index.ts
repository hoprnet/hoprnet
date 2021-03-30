import AccountEntry from './accountEntry'
import AcknowledgedTicket from './acknowledgedTicket'
import { Channel, ChannelBalance, ChannelState } from './channel'
import ChannelEntry from './channelEntry'
import createKeccakHash from 'keccak'
import Signature from './signature'
import SignedChannel from './signedChannel'
import SignedTicket from './signedTicket'
import Snapshot from './snapshot'
import Ticket from './ticket'
import { UINT256 } from './solidity'
import { ADDRESS_LENGTH, HASH_LENGTH } from '../constants'
import { u8aToHex, u8aEquals, stringToU8a, moveDecimalPoint } from '@hoprnet/hopr-utils'
import type { Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import Web3 from 'web3'
import BN from 'bn.js'
import { publicKeyConvert, publicKeyCreate } from 'secp256k1'

class Address implements Interfaces.Address {
  constructor(private id: Uint8Array) {}

  static get SIZE(): number {
    return ADDRESS_LENGTH
  }

  static fromString(str: string): Address {
    if (!Web3.utils.isAddress(str)) throw Error(`String ${str} is not an address`)
    return new Address(stringToU8a(str))
  }

  serialize() {
    return this.id
  }

  toHex(): string {
    return Web3.utils.toChecksumAddress(u8aToHex(this.id, false))
  }

  eq(b: Address) {
    return u8aEquals(this.id, b.serialize())
  }
}

class Balance implements Interfaces.Balance {
  constructor(private bn: BN) {}

  static get SYMBOL(): string {
    return `HOPR`
  }

  static get DECIMALS(): number {
    return 18
  }

  public toBN(): BN {
    return this.bn
  }

  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', Balance.SIZE))
  }

  public toFormattedString(): string {
    return moveDecimalPoint(this.bn.toString(), Balance.DECIMALS * -1) + ' ' + Balance.SYMBOL
  }

  static get SIZE(): number {
    // Uint256
    return 32
  }
}

class Hash implements Interfaces.Hash {
  constructor(private arr: Uint8Array) {}

  static get SIZE() {
    return HASH_LENGTH
  }

  static create(msg: Uint8Array) {
    return new Hash(createKeccakHash('keccak256').update(Buffer.from(msg)).digest())
  }

  serialize(): Uint8Array {
    return this.arr
  }

  eq(b: Hash) {
    return u8aEquals(this.arr, b.serialize())
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  clone(): Hash {
    return new Hash(this.arr.slice())
  }

  hash(): Hash {
    // Sometimes we double hash.
    return Hash.create(this.serialize())
  }

  get length() {
    return this.arr.length
  }
}

class NativeBalance implements Interfaces.Balance {
  constructor(private bn: BN) {}

  static get SYMBOL(): string {
    return `xDAI`
  }

  static get DECIMALS(): number {
    return 18
  }

  public toBN(): BN {
    return this.bn
  }

  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', NativeBalance.SIZE))
  }

  public toFormattedString(): string {
    return moveDecimalPoint(this.bn.toString(), NativeBalance.DECIMALS * -1) + ' ' + NativeBalance.SYMBOL
  }

  static get SIZE(): number {
    // Uint256
    return 32
  }

}

class PublicKey implements Interfaces.PublicKey {
  constructor(private arr: Uint8Array) {
    // TODO check length
  }

  static fromPrivKey(privKey: Uint8Array): PublicKey {
    let arr = publicKeyCreate(privKey, true)
    return new PublicKey(arr)
  }

  toAddress(): Address {
    return new Address(
      Hash.create(publicKeyConvert(this.arr, false).slice(1)).serialize().slice(12)
    )
  }

  static fromString(str: string): PublicKey {
    return new PublicKey(stringToU8a(str))
  }

  static get SIZE(): number {
    return 33 
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return u8aToHex(this.arr) 
  }
}

class Types {
  public AccountEntry = AccountEntry
  public Address = Address
  public AcknowledgedTicket = AcknowledgedTicket
  public Balance = Balance
  public Channel = Channel
  public ChannelBalance = ChannelBalance
  public ChannelState = ChannelState
  public ChannelEntry = ChannelEntry
  public Hash = Hash
  public NativeBalance = NativeBalance
  public PublicKey = PublicKey
  public Signature = Signature
  public SignedChannel = SignedChannel
  public SignedTicket = SignedTicket
  public Snapshot = Snapshot
  public Ticket = Ticket
  public UINT256 = UINT256
}

export {
  AccountEntry,
  Address,
  AcknowledgedTicket,
  Balance,
  Channel,
  ChannelBalance,
  ChannelEntry,
  ChannelState,
  Hash,
  NativeBalance,
  PublicKey,
  Signature,
  SignedChannel,
  SignedTicket,
  Snapshot,
  Ticket,
  UINT256
}

export default Types
