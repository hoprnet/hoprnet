// SPDX-License-Identifier: MIT
// OpenZeppelin Contracts (last updated v4.8.0) (utils/structs/EnumerableSet.sol)
// This file was procedurally generated from scripts/generate/templates/EnumerableSet.js.

pragma solidity ^0.8.0;

enum Clearance {
    NONE,
    FUNCTION
}

enum TargetType {
    TOKEN,
    CHANNELS,
    SEND
}

enum TargetPermission {
    BLOCK_ALL,
    SPECIFIC_FALLBACK_BLOCK,
    SPECIFIC_FALLBACK_ALLOW,
    ALLOW_ALL
}

enum CapabilityPermission {
    NONE,
    BLOCK_ALL,
    SPECIFIC_FALLBACK_BLOCK,
    SPECIFIC_FALLBACK_ALLOW,
    ALLOW_ALL
}

/**
 * @dev it stores the following information in uint256 = (160 + 8 * 12)
 * (address)              as uint160: targetAddress
 * (Clearance)            as uint8: clearance
 * (TargetType)           as uint8: targetType
 * (TargetPermission)     as uint8: defaultTargetPermission                                       (for the target)
 * (CapabilityPermission) as uint8: defaultRedeemTicketSafeFunctionPermisson                      (for Channels
 * contract)
 * (CapabilityPermission) as uint8: RESERVED FOR defaultBatchRedeemTicketsSafeFunctionPermisson   (for Channels
 * contract)
 * (CapabilityPermission) as uint8: defaultCloseIncomingChannelSafeFunctionPermisson              (for Channels
 * contract)
 * (CapabilityPermission) as uint8: defaultInitiateOutgoingChannelClosureSafeFunctionPermisson    (for Channels
 * contract)
 * (CapabilityPermission) as uint8: defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson    (for Channels
 * contract)
 * (CapabilityPermission) as uint8: defaultFundChannelMultiFunctionPermisson                      (for Channels
 * contract)
 * (CapabilityPermission) as uint8: defaultSetCommitmentSafeFunctionPermisson                     (for Channels
 * contract)
 * (CapabilityPermission) as uint8: defaultApproveFunctionPermisson                               (for Token contract)
 * (CapabilityPermission) as uint8: defaultSendFunctionPermisson                                  (for Token contract)
 */
type Target is uint256;

/// capability permissions exceed maximum length
error TooManyCapabilities();

/// cannot convert a function permission to target permission
error PermissionNotFound();

library TargetUtils {
    uint256 internal constant NUM_CAPABILITY_PERMISSIONS = 9;

    function getNumCapabilityPermissions() internal pure returns (uint256) {
        return NUM_CAPABILITY_PERMISSIONS;
    }

    function getTargetAddress(Target target) internal pure returns (address) {
        return address(uint160(Target.unwrap(target) >> 96));
    }

    function getTargetClearance(Target target) internal pure returns (Clearance) {
        // left shift 160 bits then right shift 256 - 8 bits
        return Clearance(uint8((Target.unwrap(target) << 160) >> 248));
    }

    function getTargetType(Target target) internal pure returns (TargetType) {
        // left shift 160 + 8 bits then right shift 256 - 8 bits
        return TargetType(uint8((Target.unwrap(target) << 168) >> 248));
    }

    function isTargetType(Target target, TargetType targetType) internal pure returns (bool) {
        // compare if the target type is expectec
        return getTargetType(target) == targetType;
    }

    function getDefaultTargetPermission(Target target) internal pure returns (TargetPermission) {
        // left shift 160 + 8 + 8 bits then right shift 256 - 8 bits
        return TargetPermission(uint8((Target.unwrap(target) << 176) >> 248));
    }

    function getDefaultCapabilityPermissionAt(
        Target target,
        uint256 position
    )
        internal
        pure
        returns (CapabilityPermission)
    {
        if (position >= NUM_CAPABILITY_PERMISSIONS) {
            revert TooManyCapabilities();
        }
        // left shift 160 + 8 + 8 + 8 + 8 * pos bits then right shift 256 - 8 bits
        uint256 leftShiftBy = 184 + (8 * position);
        return CapabilityPermission(uint8((Target.unwrap(target) << leftShiftBy) >> 248));
    }

    function forceWriteAsTargetType(Target target, TargetType targetType) internal pure returns (Target) {
        // remove value at TargetType position (22/32 bytes from left)
        // remove function permissions
        uint256 updatedTarget;
        uint256 typeMask;
        if (targetType == TargetType.CHANNELS) {
            /**
             * remove all the default token function permissions (uint16). Equivalent to
             *          updatedTarget = (Target.unwrap(target) >> 16) << 16;
             *          updatedTarget &= ~targetTypeMask;
             */
            typeMask = uint256(bytes32(hex"ffffffffffffffffffffffffffffffffffffffffff00ffffffffffffffff0000"));
        } else if (targetType == TargetType.TOKEN) {
            /**
             * remove all the default function permissions (uint72)
             *          add the last 16 bits (from right) back. Equivalent to
             *          updatedTarget = (Target.unwrap(target) >> 72) << 72;
             *          updatedTarget |= (Target.unwrap(target) << 240) >> 240;
             *          updatedTarget &= ~targetTypeMask;
             */
            typeMask = uint256(bytes32(hex"ffffffffffffffffffffffffffffffffffffffffff00ff00000000000000ffff"));
        } else {
            /**
             * remove all the default function permissions (uint72). Equivalent to
             *          updatedTarget = (Target.unwrap(target) >> 72) << 72;
             *          updatedTarget &= ~targetTypeMask;
             */
            typeMask = uint256(bytes32(hex"ffffffffffffffffffffffffffffffffffffffffff00ff000000000000000000"));
        }
        updatedTarget = Target.unwrap(target) & typeMask;

        // force clear target type and overwrite with expected one
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
     * @param capabilityPermissions Array of default function permissions
     * Returns the wrapped target
     */
    function encodeDefaultPermissions(
        address targetAddress,
        Clearance clearance,
        TargetType targetType,
        TargetPermission targetPermission,
        CapabilityPermission[] memory capabilityPermissions
    )
        internal
        pure
        returns (Target target)
    {
        if (capabilityPermissions.length > NUM_CAPABILITY_PERMISSIONS) {
            revert TooManyCapabilities();
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
        // include the CapabilityPermissions to the last 8 * 9 = 72 bits
        for (uint256 i = 0; i < capabilityPermissions.length; i++) {
            // left shift 72 - 8 - 8 * i bits
            _target |= uint256(capabilityPermissions[i]) << (64 - (8 * i));
        }
        return Target.wrap(_target);
    }

    /**
     * @dev Decode the target type to target address, clearance, target type and default permissions
     * @param target the wrapped target
     */
    function decodeDefaultPermissions(Target target)
        internal
        pure
        returns (
            address targetAddress,
            Clearance clearance,
            TargetType targetType,
            TargetPermission targetPermission,
            CapabilityPermission[] memory capabilityPermissions
        )
    {
        // take the first 160 bits and parse it as address
        targetAddress = address(uint160(Target.unwrap(target) >> 96));
        // take the next 8 bits as clearance
        clearance = Clearance(uint8((Target.unwrap(target) << 160) >> 248));
        // take the next 8 bits as targetType
        targetType = TargetType(uint8((Target.unwrap(target) << 168) >> 248));
        // decode default target permissions
        targetPermission = TargetPermission(uint8((Target.unwrap(target) << 176) >> 248));

        // there are 1 default target permission and 8 default function permissions
        capabilityPermissions = new CapabilityPermission[](NUM_CAPABILITY_PERMISSIONS);
        // decode function permissions. By default, 8 function permissions
        for (uint256 i = 0; i < NUM_CAPABILITY_PERMISSIONS; i++) {
            // first left offset byt 184 + 8 * i bits
            // where 184 = 160 (address) + 8 (Clearance) + 8 (TargetType) + 8 (TargetPermission)
            // then RIGHT shift 256 - 8 = 248 bits
            capabilityPermissions[i] = CapabilityPermission(uint8((Target.unwrap(target) << (184 + (8 * i))) >> 248));
        }
    }

    function convertFunctionToTargetPermission(CapabilityPermission capabilityPermission)
        internal
        pure
        returns (TargetPermission)
    {
        uint8 permissionIndex = uint8(capabilityPermission);
        if (permissionIndex == 0) {
            revert PermissionNotFound();
        }
        return TargetPermission(permissionIndex - 1);
    }
}
