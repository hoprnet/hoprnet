// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

/**
 * @title Interface for HoprNetworkRegistryProxy
 * @dev Network Registry contract (NR) delegates its eligibility check to Network
 * Registry Proxy (NR Proxy) contract. This interface must be implemented by the
 * NR Proxy contract.
 */
interface IHoprNetworkRegistryRequirement {
  /**
   * @dev Get the maximum number of nodes that a staking account can register, and 
   * the number of nodes that can be registered by the staking account
   * @notice This check is only performed when registering new nodes, i.e. if the number gets
   * reduced later, it does not affect registered nodes.
   * Not all the nodes can be registered by the given staking account.
   * @param account Address that can register other nodes
   */
  function maxAllowedRegistrations(address stakingAccount, address[] memory nodeAddresses) external view returns (uint256 allowancePerAccount, uint256 eligibleNodeCount);
}
