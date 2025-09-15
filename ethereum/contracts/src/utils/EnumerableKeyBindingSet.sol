// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

// 4 bytes incremental key-id, as a compression technique
type KeyId is uint32;
// max key id is 2^32 - 1, as key id is a 4 byte unsigned integer
uint32 constant MAX_KEY_ID = 0xFFFFFFFF; // 2^32 - 1

/// forge-lint:disable-next-item(mixed-case-variable)
struct KeyBindingWithSignature {
    bytes32 ed25519_sig_0;
    bytes32 ed25519_sig_1;
    bytes32 ed25519_pub_key;
    address chain_key;
}

struct KeyBindingSet {
    // Storage of KeyBindingWithSignature values
    // items are stored from index 0 to MAX_KEY_ID
    // There can be at most (MAX_KEY_ID + 1) items in the set
    KeyBindingWithSignature[] _values;
    // Position of the value in the `values` array plus 1 
    // because index 0 means a value is not in the set.
    // The key is the ed25519_pub_key of the KeyBindingWithSignature.
    // Each ed25519_pub_key can only be associated with maximum one chain_key.
    mapping(bytes32 => uint256) _indexes;
}

/**
 * @dev Library for managing key bindings
 * Adapted from OpenZeppelin's EnumerableSet and EnumerableMap (`AddressToUintMap`)
 * library, for KeyBindingWithSignature type.
 */
library EnumerableKeyBindingSet {
    // when the address is not stared as a target address
    error NonExistentKey();
    // when the key id is out of range
    error KeyIdOutOfRange();
    // when the ed25519_pub_key is already in the set
    error ExistingKeyBinding();

    /**
     * @dev Add a value to a set. O(1).
     *
     * Returns the key id if the value was added to the set, that is if it was not
     * already present. Key id starts from 0 to MAX_KEY_ID.
     */
    function add(KeyBindingSet storage set, KeyBindingWithSignature memory value) internal returns (uint256) {
        // Check if the set is full
        if (set._values.length >= uint256(MAX_KEY_ID)) {
            revert KeyIdOutOfRange();
        }

        // Check if the ed25519_pub_key is already in the set
        if (contains(set, value.ed25519_pub_key)) {
            revert ExistingKeyBinding();
        }

        // add value to the set
        set._values.push(value);
        // The value is stored at length-1, but we add 1 to all indexes
        // and use 0 as a sentinel value
        set._indexes[value.ed25519_pub_key] = set._values.length;

        return set._values.length - 1;
    }

    /**
     * @dev Returns true if the ed25519_pub_key is in the set. O(1).
     *      This function is used to check if a key binding exists for a given public key.
     *      Chain_key (address) can be associated with multiple ed25519_pub_key.
     *      However, each ed25519_pub_key can only be associated with one chain_key.
     */
    /// forge-lint:disable-next-line(mixed-case-variable)
    function contains(KeyBindingSet storage set, bytes32 ed25519_pub_key) internal view returns (bool) {
        return set._indexes[ed25519_pub_key] != 0;
    }

    /**
     * @dev Returns the number of values on the set. O(1).
     */
    function length(KeyBindingSet storage set) internal view returns (uint256) {
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
    function at(KeyBindingSet storage set, uint256 index) internal view returns (KeyBindingWithSignature memory) {
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
    function values(KeyBindingSet storage set) internal view returns (KeyBindingWithSignature[] memory) {
        return set._values;
    }

    /**
     * @dev Tries to returns the value associated with the key `ed25519_pub_key`. O(1).
     *      Does not revert if `ed25519_pub_key` is not in the map.
     * @return (bool, uint256, KeyBindingWithSignature memory)
     *          Returns (true, keyId, KeyBindingWithSignature) if the ed25519_pub_key is in the set,
     *          where keyId is the index of the KeyBindingWithSignature in the set (starting from 0).
     *          Returns (false, 0, empty KeyBindingWithSignature) if the ed25519_pub_key is not in the set.
     */
    /// forge-lint:disable-next-line(mixed-case-variable)
    function tryGet(KeyBindingSet storage set, bytes32 ed25519_pub_key) internal view returns (bool, uint256, KeyBindingWithSignature memory) {
        uint256 index = set._indexes[ed25519_pub_key];
        if (index == 0) {
            return (false, 0, KeyBindingWithSignature(
                bytes32(0),
                bytes32(0),
                bytes32(0),
                address(0)
            ));
        } else {
            return (true, index - 1, set._values[index - 1]);
        }
    }

    /**
     * @dev Returns the value associated with `ed25519_pub_key` key. O(1).
     *
     * Requirements:
     *
     * - `ed25519_pub_key` key must be in the map.
     */
    /// forge-lint:disable-next-line(mixed-case-variable)
    function get(KeyBindingSet storage set, bytes32 ed25519_pub_key) internal view returns (KeyBindingWithSignature memory) {
        uint256 index = set._indexes[ed25519_pub_key];
        if (index == 0) {
            revert NonExistentKey();
        }
        return set._values[index - 1];
    }
}
