// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

// import '@openzeppelin/contracts/token/ERC20/IERC20.sol';

interface IHoprChannels {
  function IS_HOPR_CHANNELS() external view returns (bool);
  function token() public view returns (address);
  function fundChannelMulti(address account1, address account2, uint256 amount1, uint256 amount2) external;
  function redeemTicket(
    address source,
    bytes32 nextCommitment,
    uint256 ticketEpoch,
    uint256 ticketIndex,
    bytes32 proofOfRelaySecret,
    uint256 amount,
    uint256 winProb,
    bytes memory signature
  ) external;
  function initiateChannelClosure(address destination) external;
  function finalizeChannelClosure(address destination) external;
  function bumpChannel(address source, bytes32 newCommitment) external;
  // TODO: add close incoming channel
  // FIXME: update interface when HoprChannels contracts get updated
}
