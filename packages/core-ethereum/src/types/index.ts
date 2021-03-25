import AcknowledgedTicket from './acknowledgedTicket'
import Balance from './balance'
import { Channel, ChannelBalance, ChannelState } from './channel'
import ChannelEntry from './channelEntry'
import Hash from './hash'
import Moment from './moment'
import NativeBalance from './nativeBalance'
import Public from './public'
import Signature from './signature'
import SignedChannel from './signedChannel'
import SignedTicket from './signedTicket'
import Snapshot from './snapshot'
import Ticket from './ticket'
import TicketEpoch from './ticketEpoch'

import { ADDRESS_LENGTH } from '../constants'
import { u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'
import type { Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import Web3 from 'web3'

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

class Types {
  public Address = Address
  public AcknowledgedTicket = AcknowledgedTicket
  public Balance = Balance
  public Channel = Channel
  public ChannelBalance = ChannelBalance
  public ChannelState = ChannelState
  public ChannelEntry = ChannelEntry
  public Hash = Hash
  public Moment = Moment
  public NativeBalance = NativeBalance
  public Public = Public
  public Signature = Signature
  public SignedChannel = SignedChannel
  public SignedTicket = SignedTicket
  public Snapshot = Snapshot
  public Ticket = Ticket
  public TicketEpoch = TicketEpoch
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
  Moment,
  NativeBalance,
  Public,
  Signature,
  SignedChannel,
  SignedTicket,
  Snapshot,
  Ticket,
  TicketEpoch
}

export default Types
