// SPDX-License-Identifier: GPL-3.0-only

pragma solidity ^0.8.0;

import "../../src/utils/EnumerableTargetSet.sol";

/**
 * @dev Mock contract to test internal library of EnumerableTargetSet
 * Each function from the libarray has a wrapper in the mock contract
 */
contract EnumerableTargetSetMock {
    using EnumerableTargetSet for TargetSet;

    TargetSet internal targetSet;

    function add(Target target) public returns (bool) {
        return EnumerableTargetSet.add(targetSet, target);
    }

    function remove(address targetAddress) public returns (bool) {
        return EnumerableTargetSet.remove(targetSet, targetAddress);
    }

    function contains(address targetAddress) public view returns (bool) {
        return EnumerableTargetSet.contains(targetSet, targetAddress);
    }

    function length() public view returns (uint256) {
        return EnumerableTargetSet.length(targetSet);
    }

    function at(uint256 index) public view returns (Target) {
        return EnumerableTargetSet.at(targetSet, index);
    }

    function values() public view returns (Target[] memory) {
        return EnumerableTargetSet.values(targetSet);
    }

    function tryGet(address targetAddress) public view returns (bool, Target) {
        return EnumerableTargetSet.tryGet(targetSet, targetAddress);
    }

    function get(address targetAddress) public view returns (Target) {
        return EnumerableTargetSet.get(targetSet, targetAddress);
    }
}
