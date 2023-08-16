// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.0;

/**
 * @title Interface for HoprNetworkRegistryProxy
 * @dev Network Registry contract (NR) delegates its eligibility check to Network
 * Registry Proxy (NR Proxy) contract. This interface must be implemented by the
 * NR Proxy contract.
 */
interface IHoprNetworkRegistryRequirement {
    /**
     * @dev Get the maximum number of nodes that a staking account can register
     * @notice This check is only returns value based on current critieria
     * @param stakingAccount Staking account
     */
    function maxAllowedRegistrations(address stakingAccount) external view returns (uint256 allowancePerAccount);

    /**
     * @dev Get if the staking account is eligible to act on node address
     * @param stakingAccount Staking account
     * @param nodeAddress node address
     */
    function canOperateFor(address stakingAccount, address nodeAddress) external view returns (bool eligiblity);
}
