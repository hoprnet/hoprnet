// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

/**
 * @dev Minimum interface for NodeSafeRegistry contract
 */
contract IHoprNodeSafeRegistry {
    function nodeToSafe(address node) public view returns (address) {}
}
