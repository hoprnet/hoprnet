import AccountEntry from './accountEntry'
import AcknowledgedTicket from './acknowledgedTicket'
import { Channel, ChannelBalance, ChannelState } from './channel'
import ChannelEntry from './channelEntry'
import Signature from './signature'
import SignedChannel from './signedChannel'
import SignedTicket from './signedTicket'
import Snapshot from './snapshot'
import Ticket from './ticket'
import { UINT256 } from './solidity'
import { Address, Balance, Hash, NativeBalance, PublicKey } from './primitives'


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
