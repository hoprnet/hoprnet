// SPDX-License-Identifier: MIT
// OpenZeppelin Contracts (last updated v4.8.0) (utils/structs/EnumerableSet.sol)
// This file was procedurally generated from scripts/generate/templates/EnumerableSet.js.

pragma solidity ^0.8.0;

import { TargetUtils, Target } from "./TargetUtils.sol";

struct TargetSet {
    // Storage of set values
    Target[] _values;
    // Position of the value in the `values` array, plus 1 because index 0
    // means a value is not in the set.
    // the key is not `Target` but the first 160 bits converted to address
    mapping(address => uint256) _indexes;
}

/**
 * @dev Library for managing
 * https://en.wikipedia.org/wiki/Set_(abstract_data_type)[sets] of primitive
 * types.
 *
 * Adapted from OpenZeppelin's EnumerableSet and EnumerableMap (`AddressToUintMap`)
 * library, for TargetDefaultPermissions type.
 */
library EnumerableTargetSet {
    using TargetUtils for Target;

    // when the address is not stared as a target address
    error NonExistentKey();

    /**
     * @dev Add a value to a set. O(1).
     *
     * Returns true if the value was added to the set, that is if it was not
     * already present.
     */
    function add(TargetSet storage set, Target value) internal returns (bool) {
        if (!contains(set, value.getTargetAddress())) {
            set._values.push(value);
            // The value is stored at length-1, but we add 1 to all indexes
            // and use 0 as a sentinel value
            set._indexes[value.getTargetAddress()] = set._values.length;
            return true;
        } else {
            return false;
        }
    }

    /**
     * @dev Removes a value from a set. O(1).
     * @notice remove by the first 160 bits of target value (target address) instead of the target
     * Returns true if the value was removed from the set, that is if it was
     * present.
     */
    function remove(TargetSet storage set, address targetAddress) internal returns (bool) {
        // We read and store the value's index to prevent multiple reads from the same storage slot
        uint256 valueIndex = set._indexes[targetAddress];

        if (valueIndex != 0) {
            // Equivalent to contains(set, value)
            // To delete an element from the _values array in O(1), we swap the element to delete with the last one in
            // the array, and then remove the last element (sometimes called as 'swap and pop').
            // This modifies the order of the array, as noted in {at}.

            uint256 toDeleteIndex = valueIndex - 1;
            uint256 lastIndex = set._values.length - 1;

            if (lastIndex != toDeleteIndex) {
                Target lastValue = set._values[lastIndex];

                // Move the last value to the index where the value to delete is
                set._values[toDeleteIndex] = lastValue;
                // Update the index for the moved value
                set._indexes[lastValue.getTargetAddress()] = valueIndex; // Replace lastValue's index to valueIndex
            }

            // Delete the slot where the moved value was stored
            set._values.pop();

            // Delete the index for the deleted slot
            delete set._indexes[targetAddress];

            return true;
        } else {
            return false;
        }
    }

    /**
     * @dev Returns true if the targetAddress (first 160 bits in `value`) is in the set. O(1).
     * @notice remove by the first 160 bits of target value (target address) instead of the target
     */
    function contains(TargetSet storage set, address targetAddress) internal view returns (bool) {
        return set._indexes[targetAddress] != 0;
    }

    /**
     * @dev Returns the number of values on the set. O(1).
     */
    function length(TargetSet storage set) internal view returns (uint256) {
        return set._values.length;
    }

    /**
     * @dev Returns the value stored at position `index` in the set. O(1).
     *
     * Note that there are no guarantees on the ordering of values inside the
     * array, and it may change when more values are added or removed.
     *
     * Requirements:
     *
     * - `index` must be strictly less than {length}.
     */
    function at(TargetSet storage set, uint256 index) internal view returns (Target) {
        return set._values[index];
    }

    /**
     * @dev Return the entire set in an array
     *
     * WARNING: This operation will copy the entire storage to memory, which can be quite expensive. This is designed
     * to mostly be used by view accessors that are queried without any gas fees. Developers should keep in mind that
     * this function has an unbounded cost, and using it as part of a state-changing function may render the function
     * uncallable if the set grows to a point where copying to memory consumes too much gas to fit in a block.
     */
    function values(TargetSet storage set) internal view returns (Target[] memory) {
        return set._values;
    }

    /**
     * @dev Tries to returns the value associated with the key `targetAddress`. O(1).
     * Does not revert if `targetAddress` is not in the map.
     */
    function tryGet(TargetSet storage set, address targetAddress) internal view returns (bool, Target) {
        uint256 index = set._indexes[targetAddress];
        if (index == 0) {
            return (false, Target.wrap(0));
        } else {
            return (true, set._values[index - 1]);
        }
    }

    /**
     * @dev Returns the value associated with `targetAddress` key. O(1).
     *
     * Requirements:
     *
     * - `targetAddress` key must be in the map.
     */
    function get(TargetSet storage set, address targetAddress) internal view returns (Target) {
        uint256 index = set._indexes[targetAddress];
        if (index == 0) {
            revert NonExistentKey();
        }
        return set._values[index - 1];
    }
}
