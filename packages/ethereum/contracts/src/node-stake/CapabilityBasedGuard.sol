// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

import 'safe-contracts/base/GuardManager.sol';
import '../interfaces/IHoprChannels.sol';
import '../interfaces/IAvatar.sol';

error AddressIsZero();
error InitializationErrorSafeSet();
error GuardTheWrongSafe(address wrongSafe);

/**
 * @dev HoprCapabilityBasedGuard - A Safe Guard that checks if the signer has the capability of calling
 * certain functions on HoprChannels contract.
 */
contract HoprCapabilityBasedGuard is BaseGuard {
  IAvatar public safe;

  modifier onlySafe() {
    if (msg.sender != address(safe)) {
      revert GuardTheWrongSafe({wrongSafe: msg.sender});
    }
    _;
  }

  // set the safe address to be single
  constructor() {
    safe = IAvatar(address(0));
  }

  function setup(address _safe) external {
    // cannot accept a zero address as Safe contract
    if (_safe == address(0)) {
      revert AddressIsZero();
    }
    // cannot setup again if it's been set up
    if (address(safe) != address(0)) {
      revert InitializationErrorSafeSet();
    }
    safe = IAvatar(_safe);
  }

  // solhint-disable-next-line payable-fallback
  fallback() external {
    // We don't revert on fallback to avoid issues in case of a Safe upgrade
    // E.g. The expected check method might change and then the Safe would be locked.
  }

  // TODO: Implement this function
  /**
   * @dev
   * This function can only be called by the guarded Safe.
   * It only guards if the transaction targets a HoprChannels contract, or it targets the current guard.
   * owner allow ADMIN_ROLE to make delegate calls
   */
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
  ) external onlySafe {
    // depending on the operation type. Check directly if the
    // guarding targets: this guard and HoprChannels contracts. Relax guard when the tx is for other contracts
    // if (IHoprChannels(to).IS_HOPR_CHANNELS() == false || to != address(this)) return;

    if (to == address(this)) {
      // check if signature has sufficient permission. At least one signer should have ADMIN_ROLE
    } else {
      // when the target is HoprChannel
    }

    // Only owners can exec
    address[] memory owners = IAvatar(msg.sender).getOwners();
  }

  // TODO: Implement this function
  function checkAfterExecution(bytes32 txHash, bool success) external {}

  // function readData(bytes calldata data) private view returns () {}

  // function unwrapMultiSend() internal returns {

  // }
}
