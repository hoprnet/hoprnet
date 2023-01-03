// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import '../../src/HoprChannels.sol';

contract ChannelsUtilsTest is Test {
  bytes32 public SECRET_0 = keccak256(abi.encodePacked('secret'));
  bytes32 public SECRET_1 = keccak256(abi.encodePacked(SECRET_0));
  bytes32 public SECRET_2 = keccak256(abi.encodePacked(SECRET_1));

  /**
   * Emitted on every channel state change.
   */
  event ChannelUpdated(address indexed source, address indexed destination, HoprChannels.Channel newState);

  /**
   * Emitted once an account announces.
   */
  event Announcement(address indexed account, bytes publicKey, bytes multiaddr);

  /**
   * Emitted once a channel if funded.
   */
  event ChannelFunded(address indexed funder, address indexed source, address indexed destination, uint256 amount);

  /**
   * Emitted once bumpChannel is called.
   */
  event ChannelBumped(
    address indexed source,
    address indexed destination,
    bytes32 newCommitment,
    uint256 ticketEpoch,
    uint256 channelBalance
  );

  /**
   * Emitted once a channel is opened.
   */
  event ChannelOpened(address indexed source, address indexed destination);

  /**
   * Emitted once a channel closure is initialized.
   */
  event ChannelClosureInitiated(address indexed source, address indexed destination, uint32 closureInitiationTime);

  /**
   * Emitted once a channel closure is finalized.
   */
  event ChannelClosureFinalized(
    address indexed source,
    address indexed destination,
    uint32 closureFinalizationTime,
    uint256 channelBalance
  );
  /**
   * Emitted once a ticket is redeemed.
   */
  event TicketRedeemed(
    address indexed source,
    address indexed destination,
    bytes32 nextCommitment,
    uint256 ticketEpoch,
    uint256 ticketIndex,
    bytes32 proofOfRelaySecret,
    uint256 amount,
    uint256 winProb,
    bytes signature
  );

  /**
   * @param source the address of source
   * @param destination the address of destination
   * @return the channel id
   */
  function getChannelId(address source, address destination) public pure returns (bytes32) {
    return keccak256(abi.encodePacked(source, destination));
  }

  function assertEqChannels(HoprChannels.Channel memory channel1, HoprChannels.Channel memory channel2) public {
    bytes32 channel1Hash = getChannelHash(channel1);
    bytes32 channel2Hash = getChannelHash(channel2);
    assertEq(channel1Hash, channel2Hash);
  }

  function getChannelFromTuple(HoprChannels hoprChannels, bytes32 channelId)
    public
    view
    returns (HoprChannels.Channel memory)
  {
    (uint256 a, bytes32 b, uint256 c, uint256 d, HoprChannels.ChannelStatus e, uint256 f, uint32 g) = hoprChannels
      .channels(channelId);
    return
      HoprChannels.Channel({
        balance: a,
        commitment: b,
        ticketEpoch: c,
        ticketIndex: d,
        status: e,
        channelEpoch: f,
        closureTime: g
      });
  }

  function getChannelHash(HoprChannels.Channel memory channel) internal pure returns (bytes32) {
    return
      keccak256(
        abi.encodePacked(
          channel.balance,
          channel.commitment,
          channel.ticketEpoch,
          channel.ticketIndex,
          channel.status,
          channel.channelEpoch,
          channel.closureTime
        )
      );
  }
}
