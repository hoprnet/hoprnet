// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;

import { EnumerableKeyBindingSet, KeyBindingSet, KeyBindingWithSignature } from "../../src/utils/EnumerableKeyBindingSet.sol";

/**
 * @dev Mock contract to test internal library of EnumerableKeyBindingSet
 * Each function from the libarray has a wrapper in the mock contract
 */
contract EnumerableKeyBindingSetMock {
    using EnumerableKeyBindingSet for KeyBindingSet;

    KeyBindingSet internal keyBindingSet;

    function add(KeyBindingWithSignature memory keyBinding) public returns (uint256) {
        return EnumerableKeyBindingSet.add(keyBindingSet, keyBinding);
    }

    /// forge-lint:disable-next-line(mixed-case-variable)
    function contains(bytes32 ed25519_pub_key) public view returns (bool) {
        return EnumerableKeyBindingSet.contains(keyBindingSet, ed25519_pub_key);
    }

    function length() public view returns (uint256) {
        return EnumerableKeyBindingSet.length(keyBindingSet);
    }

    function at(uint256 index) public view returns (KeyBindingWithSignature memory) {
        return EnumerableKeyBindingSet.at(keyBindingSet, index);
    }

    function values() public view returns (KeyBindingWithSignature[] memory) {
        return EnumerableKeyBindingSet.values(keyBindingSet);
    }

    /// forge-lint:disable-next-line(mixed-case-variable)
    function tryGet(bytes32 ed25519_pub_key) public view returns (bool, uint256, KeyBindingWithSignature memory) {
        return EnumerableKeyBindingSet.tryGet(keyBindingSet, ed25519_pub_key);
    }

    /// forge-lint:disable-next-line(mixed-case-variable)
    function get(bytes32 ed25519_pub_key) public view returns (KeyBindingWithSignature memory) {
        return EnumerableKeyBindingSet.get(keyBindingSet, ed25519_pub_key);
    }
}
