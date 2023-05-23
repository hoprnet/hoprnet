// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

import 'safe-contracts/base/GuardManager.sol';

contract CapabilityBasedGuard is BaseGuard {
  // TODO: Implement this function
  function checkTransaction(
    address to,
    uint256 value,
    bytes memory data,
    Enum.Operation operation,
    uint256 safeTxGas,
    uint256 baseGas,
    uint256 gasPrice,
    address gasToken,
    address payable refundReceiver,
    bytes memory signatures,
    address msgSender
  ) external {}

  // TODO: Implement this function
  function checkAfterExecution(bytes32 txHash, bool success) external {}
}
