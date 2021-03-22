import AccountId from './accountId'
import AcknowledgedTicket from './acknowledgedTicket'
import Balance from './balance'
import { ChannelState } from './channel'
import ChannelEntry from './channelEntry'
import Hash from './hash'
import Moment from './moment'
import NativeBalance from './nativeBalance'
import Public from './public'
import Signature from './signature'
import SignedTicket from './signedTicket'
import Snapshot from './snapshot'
import Ticket from './ticket'
import TicketEpoch from './ticketEpoch'
import { Types as T } from '@hoprnet/hopr-core-connector-interface'

class Types {
  public AccountId = AccountId
  public AcknowledgedTicket = AcknowledgedTicket
  public Balance = Balance
  public ChannelState = ChannelState
  public ChannelEntry = ChannelEntry
  public ChannelStatus = T.ChannelStatus
  public Hash = Hash
  public Moment = Moment
  public NativeBalance = NativeBalance
  public Public = Public
  public Signature = Signature
  public SignedTicket = SignedTicket
  public Snapshot = Snapshot
  public Ticket = Ticket
  public TicketEpoch = TicketEpoch
}

export {
  AccountId,
  AcknowledgedTicket,
  Balance,
  ChannelEntry,
  ChannelState,
  Hash,
  Moment,
  NativeBalance,
  Public,
  Signature,
  SignedTicket,
  Snapshot,
  Ticket,
  TicketEpoch
}

export default Types
