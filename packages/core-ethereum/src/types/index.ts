import { Account } from './account'
import Address from './accountId'
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

class Types {
  public Account = Account
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
  Account,
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
