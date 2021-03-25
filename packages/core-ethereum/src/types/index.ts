import AcknowledgedTicket from './acknowledgedTicket'
import { Channel, ChannelBalance, ChannelState } from './channel'
import ChannelEntry from './channelEntry'
import Hash from './hash'
import NativeBalance from './nativeBalance'
import Public from './public'
import Signature from './signature'
import SignedChannel from './signedChannel'
import SignedTicket from './signedTicket'
import Snapshot from './snapshot'
import Ticket from './ticket'
import { UINT256 } from './solidity'

import { ADDRESS_LENGTH } from '../constants'
import { u8aToHex, u8aEquals, moveDecimalPoint } from '@hoprnet/hopr-utils'
import type { Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import Web3 from 'web3'
import BN from 'bn.js'

class Address implements Interfaces.Address {
  constructor(private id: Uint8Array) {}

  static get SIZE(): number {
    return ADDRESS_LENGTH
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

  static fromUint96(arr: Uint8Array): Balance {
    return new Balance(new BN(arr))
  }

  public toBN(): BN {
    return this.bn
  }

  public toUint96() {
    // Temp hack
    return this.bn.toBuffer('be', 12)
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

class Types {
  public Address = Address
  public AcknowledgedTicket = AcknowledgedTicket
  public Balance = Balance
  public Channel = Channel
  public ChannelBalance = ChannelBalance
  public ChannelState = ChannelState
  public ChannelEntry = ChannelEntry
  public Hash = Hash
  public NativeBalance = NativeBalance
  public Public = Public
  public Signature = Signature
  public SignedChannel = SignedChannel
  public SignedTicket = SignedTicket
  public Snapshot = Snapshot
  public Ticket = Ticket
  public UINT256 = UINT256
}

export {
  Address,
  AcknowledgedTicket,
  Balance,
  Channel,
  ChannelBalance,
  ChannelEntry,
  ChannelState,
  Hash,
  NativeBalance,
  Public,
  Signature,
  SignedChannel,
  SignedTicket,
  Snapshot,
  Ticket,
  UINT256
}

export default Types
