// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import '../../src/Channels.sol';

abstract contract ChannelsUtilsTest is Test {
  bytes32 public SECRET_0 = keccak256(abi.encodePacked('secret'));
  bytes32 public SECRET_1 = keccak256(abi.encodePacked(SECRET_0));
  bytes32 public SECRET_2 = keccak256(abi.encodePacked(SECRET_1));

  /**
   * @dev copy all the events
   */ 
  event ChannelOpened(address indexed source, address indexed destination, HoprChannels.Balance amount);
  event ChannelBalanceIncreased(bytes32 indexed channelId, HoprChannels.Balance newBalance);
  event ChannelBalanceDecreased(bytes32 indexed channelId, HoprChannels.Balance newBalance);
  event CommitmentSet(bytes32 channelId, HoprChannels.ChannelEpoch epoch);
  event OutgoingChannelClosureInitiated(bytes32 channelId, HoprChannels.Timestamp closureInitiationTime);
  event ChannelClosed(bytes32 channelId);
  event TicketRedeemed(bytes32 channelId, HoprChannels.TicketIndex newTicketIndex);

  function assertEqChannels(HoprChannels.Channel memory channel1, HoprChannels.Channel memory channel2) public {
    bytes32 channel1Hash = getChannelHash(channel1);
    bytes32 channel2Hash = getChannelHash(channel2);
    assertEq(channel1Hash, channel2Hash);
  }

  function assertEqChannelsWithId(
    HoprChannels hoprChannels, 
    bytes32 channelId, 
    HoprChannels.Channel memory channel2
  ) public {
    bytes32 channel1Hash = getChannelHash(getChannelFromTuple(hoprChannels, channelId));
    bytes32 channel2Hash = getChannelHash(channel2);
    assertEq(channel1Hash, channel2Hash);
  }

  /**
   * @param source the address of source
   * @param destination the address of destination
   * @return the channel id
   */
  function getChannelId(address source, address destination) public pure returns (bytes32) {
    return keccak256(abi.encodePacked(source, destination));
  }

  function wrapChannel(
    bytes32 _commitment, 
    uint256 _balance, 
    uint256 _ticketIndex, 
    HoprChannels.ChannelStatus _status,
    uint256 _epoch, 
    uint256 _closureTime
  )
    public
    view
    returns (HoprChannels.Channel memory)
  {
    return
      HoprChannels.Channel({
        commitment: _commitment,
        balance: HoprChannels.Balance.wrap(uint96(_balance)),
        ticketIndex: HoprChannels.TicketIndex.wrap(uint64(_ticketIndex)),
        status: _status,
        epoch: HoprChannels.ChannelEpoch.wrap(uint24(_epoch)),
        closureTime: HoprChannels.Timestamp.wrap(uint32(_closureTime))
      });
  }

  function getChannelFromTuple(
    HoprChannels hoprChannels, 
    bytes32 channelId
  ) public view returns (HoprChannels.Channel memory) {
    (bytes32 a,HoprChannels.Balance b,HoprChannels.TicketIndex c, HoprChannels.ChannelStatus d,HoprChannels.ChannelEpoch e,HoprChannels.Timestamp f) = hoprChannels.channels(channelId);
    return
      HoprChannels.Channel({
        commitment: a,
        balance: b,
        ticketIndex: c,
        status: d,
        epoch: e,
        closureTime: f
      });
  }

  function getChannelHash(HoprChannels.Channel memory channel) internal pure returns (bytes32) {
    return
      keccak256(
        abi.encodePacked(
          channel.commitment,
          channel.balance,
          channel.ticketIndex,
          channel.status,
          channel.epoch,
          channel.closureTime
        )
      );
  }  

  // function getTicketLuckInternal(
  //     bytes32 ticketHash,
  //     bytes32 secretPreImage,
  //     bytes32 proofOfRelaySecret
  // ) external pure returns (WinProb) {
  //     return _getTicketLuck(ticketHash, secretPreImage, proofOfRelaySecret);
  // }

  // function getTicketHashInternal(
  //     address recipient,
  //     uint256 recipientCounter,
  //     bytes32 proofOfRelaySecret,
  //     uint256 channelIteration,
  //     uint256 amount,
  //     uint256 ticketIndex,
  //     uint256 winProb
  // ) external pure returns (bytes32) {
  //     return ECDSA.toEthSignedMessageHash(
  //         keccak256(_getEncodedTicket(recipient, recipientCounter, proofOfRelaySecret, channelIteration, amount, ticketIndex, winProb))
  //     );
  // }
}

