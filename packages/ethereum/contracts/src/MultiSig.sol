// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

import { HoprNodeSafeRegistry } from "./node-stake/NodeSafeRegistry.sol";

/**
 *    &&&&
 *    &&&&
 *    &&&&
 *    &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *    &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *     &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *     &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *     &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *     %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *    %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                          %%%%
 *                                          %%%%
 *                                          %%%%
 *
 * Provides modifiers to enforce usage of a MultiSig contract
 */
abstract contract HoprMultiSig {
    error AlreadyInitialized();
    error MultiSigUninitialized();
    error ContractNotResponsible();
    error InvalidSafeAddress();

    HoprNodeSafeRegistry registry;
    bool initialized = false;

    /**
     * Sets address of NodeSafeRegistry contract.
     *
     * @dev Must be called exactly once
     */
    function setNodeSafeRegistry(HoprNodeSafeRegistry _registry) internal {
        if (initialized) {
            revert AlreadyInitialized();
        }
        if (address(_registry) == address(0)) {
            revert InvalidSafeAddress();
        }

        initialized = true;
        registry = _registry;
    }

    /**
     * Enforces usage of Safe contract specified in NodeSafeRegistry
     */
    modifier onlySafe(address self) {
        if (!initialized) {
            revert MultiSigUninitialized();
        }

        if (registry.nodeToSafe(self) != msg.sender) {
            revert ContractNotResponsible();
        }
        _;
    }

    /**
     * Only permits operation if no Safe contract has been specified
     * in NodeSafeRegistry
     */
    modifier noSafeSet() {
        if (!initialized) {
            revert MultiSigUninitialized();
        }

        if (registry.nodeToSafe(msg.sender) != address(0)) {
            revert ContractNotResponsible();
        }
        _;
    }
}
