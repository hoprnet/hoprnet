// SPDX-License-Identifier: MIT
// OpenZeppelin Contracts (last updated v4.8.0) (utils/structs/EnumerableSet.sol)
// This file was procedurally generated from scripts/generate/templates/EnumerableSet.js.

pragma solidity ^0.8.0;

enum Clearance { NONE, FUNCTION }

enum TargetType { TOKEN, CHANNELS, SEND }

enum TargetPermission { BLOCK_ALL, SPECIFIC_FALLBACK_BLOCK, SPECIFIC_FALLBACK_ALLOW, ALLOW_ALL}

enum FunctionPermission { NONE, BLOCK_ALL, SPECIFIC_FALLBACK_BLOCK, SPECIFIC_FALLBACK_ALLOW, ALLOW_ALL}

/**
 * @dev it stores the following information in uint256 = (160 + 8 * 12)
 * (address) as uint160: targetAddress 
 * (Clearance) as uint8: clearance 
 * (TargetType) as uint8: targetType 
 * (TargetPermission) as uint8: defaultTargetPermission                                       (for the target)
 * (FunctionPermission) as uint8: defaultRedeemTicketSafeFunctionPermisson                      (for Channels contract)
 * (FunctionPermission) as uint8: defaultBatchRedeemTicketsSafeFunctionPermisson                (for Channels contract)
 * (FunctionPermission) as uint8: defaultCloseIncomingChannelSafeFunctionPermisson              (for Channels contract)
 * (FunctionPermission) as uint8: defaultInitiateOutgoingChannelClosureSafeFunctionPermisson    (for Channels contract)
 * (FunctionPermission) as uint8: defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson    (for Channels contract)
 * (FunctionPermission) as uint8: defaultFundChannelMultiFunctionPermisson                      (for Channels contract)
 * (FunctionPermission) as uint8: defaultSetCommitmentSafeFunctionPermisson                     (for Channels contract)
 * (FunctionPermission) as uint8: defaultApproveFunctionPermisson                               (for Token contract)
 * (FunctionPermission) as uint8: defaultSendFunctionPermisson                                  (for Token contract)
 */
type Target is uint256;

/// function permissions exceed maximum length
error FunctionPermissionsTooMany();

/// cannot convert a NONe function permission to target permission
error PermissionNotFound();

library TargetUtils {
    uint256 internal constant NUM_DEFAULT_FUNCTION_PERMISSIONS = 9;

    function getTargetAddress(Target target) internal pure returns (address) {
        return address(uint160(Target.unwrap(target) >> 96));
    }

    function getTargetType(Target target) internal pure returns (TargetType) {
        // left shift 160 + 8 bits then right shift 256 - 8 bits
        return TargetType(uint8((Target.unwrap(target) << 168) >> 248));
    }

    function getTargetClearance(Target target) internal pure returns (Clearance) {
        // left shift 160 bits then right shift 256 - 8 bits
        return Clearance(uint8((Target.unwrap(target) << 160) >> 248));
    }

    function isTargetType(Target target, TargetType targetType) internal pure returns (bool) {
        // compare if the target type is expectec
        return getTargetType(target) == targetType;
    }

    function getDefaultTargetPermission(Target target) internal pure returns (TargetPermission) {
        // left shift 160 + 8 + 8 bits then right shift 256 - 8 bits
        return TargetPermission(uint8((Target.unwrap(target) << 176) >> 248));
    }

    function getDefaultFunctionPermissionAt(Target target, uint256 position) internal pure returns (FunctionPermission) {
        if (position > NUM_DEFAULT_FUNCTION_PERMISSIONS) {
            revert FunctionPermissionsTooMany();
        }
        // left shift 160 + 8 + 8 + 8 + 8 * pos bits then right shift 256 - 8 bits
        uint256 leftShiftBy = 184 + 8 * position;
        return FunctionPermission(uint8((Target.unwrap(target) << leftShiftBy) >> 248));
    }

    function forceWriteAsTargetType(Target target, TargetType targetType) internal pure returns (Target) {
        uint256 targetTypeMask = 255 << 80;
        uint256 updatedTarget;
        if (targetType == TargetType.CHANNELS) {
            // remove all the default token function permissions (uint16)
            updatedTarget = (Target.unwrap(target) >> 16) << 16;  
        } else if (targetType == TargetType.TOKEN) {
            // remove all the default function permissions (uint72)
            updatedTarget = (Target.unwrap(target) >> 72) << 72;
            // add the last 16 bits (from right) back
            updatedTarget |= (Target.unwrap(target) << 240) >> 240;
        } else {
            // remove all the default function permissions (uint72)
            updatedTarget = (Target.unwrap(target) >> 72) << 72;
        }

        // force clear target type and overwrite with expected one 
        updatedTarget &= ~targetTypeMask;
        updatedTarget |= uint256(targetType) << 80;   
        return Target.wrap(updatedTarget);
    }

    function forceWriteTargetAddress(Target target, address targetAddress) internal pure returns (Target) {
        // remove the 160 bits from left
        uint256 updatedTarget = (Target.unwrap(target) << 160) >> 160;
        // add the target address to the left
        updatedTarget |= uint256(uint160(targetAddress)) << 96;
        return Target.wrap(updatedTarget);
    }

    /**
     * @dev Encode the target address, clearance, target type and default permissions to a Target type
     * @param targetAddress addres of the target
     * @param clearance clearance of the target
     * @param targetType Type of the target
     * @param targetPermission default target permissions
     * @param functionPermissions Array of default function permissions
     * Returns the wrapped target
     */
    function encodeDefaultPermissions(
        address targetAddress,
        Clearance clearance,
        TargetType targetType,
        TargetPermission targetPermission,
        FunctionPermission[] memory functionPermissions
    ) internal pure returns (Target target) {
        if (functionPermissions.length > NUM_DEFAULT_FUNCTION_PERMISSIONS) {
            revert FunctionPermissionsTooMany();
        }
        
        uint256 _target;
        // include address to the first 160 bits
        _target |= uint256(uint160(targetAddress)) << 96;
        // include clearance to the next 8 bits (256 - 160 - 8 = 88)
        _target |= uint256(clearance) << 88;
        // inclue targetType to the next 8 bits (258 - 160 - 8 - 8 = 80)
        _target |= uint256(targetType) << 80;
        // inclue TargetPermission to the next 8 bits (258 - 160 - 8 - 8 - 8 = 72)
        _target |= uint256(targetPermission) << 72;
        // include the functionPermissions to the last 8 * 9 = 72 bits
        for (uint256 i = 0; i < functionPermissions.length; i++) {
            // left shift 72 - 8 - 8 * i bits
            _target |= uint256(functionPermissions[i]) << (64 - 8 * i);
        }
        return Target.wrap(_target);
    }

    /**
     * @dev Decode the target type to target address, clearance, target type and default permissions
     * @param target the wrapped target
     */
    function decodeDefaultPermissions(
        Target target
    ) internal pure returns (
        address targetAddress,
        Clearance clearance,
        TargetType targetType,
        TargetPermission targetPermission,
        FunctionPermission[] memory functionPermissions
    ) {
        // take the first 160 bits and parse it as address
        targetAddress = address(uint160(Target.unwrap(target) >> 96));
        // take the next 8 bits as clearance
        clearance = Clearance(uint8((Target.unwrap(target) << 160) >> 248));
        // take the next 8 bits as targetType
        targetType = TargetType(uint8((Target.unwrap(target) << 168) >> 248));
        // decode default target permissions
        targetPermission = TargetPermission(uint8(Target.unwrap(target) << 176 >> 248));
        
        // there are 1 default target permission and 8 default function permissions
        functionPermissions = new FunctionPermission[](NUM_DEFAULT_FUNCTION_PERMISSIONS);
        // decode function permissions. By default, 8 function permissions
        for (uint256 i = 0; i < NUM_DEFAULT_FUNCTION_PERMISSIONS; i++) {
            // first left shift 160 + 8 + 8  + 8 * i = 176 + 8 * i bits
            // then RIGHT shift 256 - 8 = 248 bits
            functionPermissions[i] = FunctionPermission(uint8(Target.unwrap(target) << (176 + 8 * i) >> 248));
        }
    }

    function convertFunctionToTargetPermission(FunctionPermission functionPermission) internal pure returns (TargetPermission) {
        uint8 permissionIndex = uint8(functionPermission);
        if (permissionIndex == 0) {
            revert PermissionNotFound();
        }
        return TargetPermission(permissionIndex - 1);
    }
}

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
            return (true, set._values[index]);
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
        if (index != 0) {
            revert NonExistentKey();
        }
        return set._values[index];
    }
}
