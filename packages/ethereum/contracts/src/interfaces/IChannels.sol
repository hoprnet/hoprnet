// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import 'openzeppelin-contracts-4.8.3/token/ERC20/IERC20.sol';

interface IHoprChannels {
    /**
     * @dev Type definitions
     */
    type Balance is uint96;
    type TicketIndex is uint64;
    type ChannelEpoch is uint24;
    type WinProb is uint56; // Using IEEE 754 double precision -> 53 significant bits

  /**
   * Holds a compact ECDSA signature, following ERC-2098
   */
  struct CompactSignature {
    bytes32 r;
    bytes32 vs;
  }

  /**
   * Represents a ticket that can be redeemed using `redeemTicket` function.
   *
   * Aligned to 2 EVM words
   */
  struct Ticket {
    bytes32 channelId;
    Balance amount;
    TicketIndex ticketIndex;
    ChannelEpoch epoch;
    WinProb winProb;
    uint16 resered; // for future use
  }


  /**
   * @dev
   */
  function token() external view returns (IERC20);

  /**
   * @dev
   */
  function fundChannelMulti(
    address account1,
    Balance amount1,
    address account2,
    Balance amount2
  ) external;

  /**
   * @dev
   */
  function redeemTicketSafe(
    address self,
    bytes32 nextCommitment,
    bytes32 porSecret,
    CompactSignature calldata signature,
    Ticket calldata ticket
  ) external;

  /**
   * @dev
   */
  function redeemTicket(
    bytes32 nextCommitment,
    bytes32 porSecret,
    CompactSignature calldata signature,
    Ticket calldata ticket
  ) external;

//   // FIXME: update with the correct functions
//   function redeemTickets(
//     address[] memory source,
//     address[] memory destination,
//     bytes32[] memory nextCommitment,
//     uint256[] memory ticketEpoch,
//     uint256[] memory ticketIndex,
//     bytes32[] memory proofOfRelaySecret,
//     uint256[] memory amount,
//     uint256[] memory winProb,
//     bytes memory signature
//   ) external;

  /**
   * @dev
   */
  function initiateOutgoingChannelClosureSafe(address self, address destination) external;
  /**
   * @dev
   */
  function initiateOutgoingChannelClosure(address destination) external;
  /**
   * @dev
   */
  function finalizeOutgoingChannelClosureSafe(address self, address destination) external;
  /**
   * @dev
   */
  function finalizeOutgoingChannelClosure(address destination) external;
  /**
   * @dev
   */
  function closeIncomingChannelSafe(address self, address source) external;
  /**
   * @dev
   */
  function closeIncomingChannel(address source) external;
  /**
   * @dev
   */
  function setCommitmentSafe(address self, bytes32 newCommitment, address source) external;
  /**
   * @dev
   */
  function setCommitment(bytes32 newCommitment, address source) external;
}
