// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

// import '@openzeppelin/contracts/token/ERC20/IERC20.sol';

// FIXME: update interfaces when HoprChannels contracts get updated
interface IHoprChannels {
  function IS_HOPR_CHANNELS() external view returns (bool);
  // FIXME:
  // function token() external view returns (address);
  function fundChannelMulti(address account1, address account2, uint256 amount1, uint256 amount2) external;
  function redeemTicket(
    address source,
    address destination,
    bytes32 nextCommitment,
    uint256 ticketEpoch,
    uint256 ticketIndex,
    bytes32 proofOfRelaySecret,
    uint256 amount,
    uint256 winProb,
    bytes memory signature
  ) external;
  // FIXME: update with the correct functions
  function redeemTickets(
    address[] memory source,
    address[] memory destination,
    bytes32[] memory nextCommitment,
    uint256[] memory ticketEpoch,
    uint256[] memory ticketIndex,
    bytes32[] memory proofOfRelaySecret,
    uint256[] memory amount,
    uint256[] memory winProb,
    bytes memory signature
  ) external;
  function initiateChannelClosure(address source, address destination) external;
  function finalizeChannelClosure(address source, address destination) external;
  function bumpChannel(address source, address destination, bytes32 newCommitment) external;
}
