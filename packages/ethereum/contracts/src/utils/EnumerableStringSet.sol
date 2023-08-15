// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

/**
 * @dev Library for managing
 * https://en.wikipedia.org/wiki/Set_(abstract_data_type)[sets] of primitive
 * types.
 *
 * Adapted from OpenZeppelin's EnumerableSet library, for string type.
 */
library EnumerableStringSet {
    struct StringSet {
        // Storage of set values
        string[] _values;
        // Position of the value in the `values` array, plus 1 because index 0
        // means a value is not in the set.
        mapping(string => uint256) _indexes;
    }

    event SetCreated(uint256 indexed typeIndex);

    /**
     * @dev Add a value to a set. O(1).
     *
     * Returns true if the value was added to the set, that is if it was not
     * already present.
     */
    function add(StringSet storage set, string memory value) internal returns (bool) {
        if (!contains(set, value)) {
            set._values.push(value);
            // The value is stored at length-1, but we add 1 to all indexes
            // and use 0 as a sentinel value
            set._indexes[value] = set._values.length;
            emit SetCreated(set._indexes[value]);
            return true;
        } else {
            return false;
        }
    }

    /**
     * @dev Returns true if the value is in the set. O(1).
     */
    function contains(StringSet storage set, string memory value) internal view returns (bool) {
        return set._indexes[value] != 0;
    }

    /**
     * @dev Returns the value stored at position `index` in the set. O(1).
     */
    function at(StringSet storage set, uint256 index) internal view returns (string memory) {
        return set._values[index - 1];
    }

    /**
     * @dev Returns index of a given value.
     */
    function indexOf(StringSet storage set, string memory value) internal view returns (uint256) {
        return set._indexes[value];
    }
}
