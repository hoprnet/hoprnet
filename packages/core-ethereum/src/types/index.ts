import AccountId from './accountId'
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
import UnacknowledgedTicket from './unacknowledged'

class Types {
  public AccountId = AccountId
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
  public UnacknowledgedTicket = UnacknowledgedTicket
}

export {
  AccountId,
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
  TicketEpoch,
  UnacknowledgedTicket
}

export default Types
