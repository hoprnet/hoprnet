import AccountEntry from './accountEntry'
import AcknowledgedTicket from './acknowledgedTicket'
import ChannelEntry from './channelEntry'
import Snapshot from './snapshot'
import Ticket from './ticket'
import { UINT256 } from './solidity'
import { Address, Balance, Hash, NativeBalance, PublicKey, Signature } from './primitives'

class Types {
  public AccountEntry = AccountEntry
  public Address = Address
  public AcknowledgedTicket = AcknowledgedTicket
  public Balance = Balance
  public ChannelEntry = ChannelEntry
  public Hash = Hash
  public NativeBalance = NativeBalance
  public PublicKey = PublicKey
  public Signature = Signature
  public Snapshot = Snapshot
  public Ticket = Ticket
  public UINT256 = UINT256
}

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
  Snapshot,
  Ticket,
  UINT256
}

export default Types
