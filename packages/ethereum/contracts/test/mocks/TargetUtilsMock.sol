// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.0;

import "../../src/utils/TargetUtils.sol";

/** 
 * @dev Mock contract to test internal library of TargetUtils
 * Each function from the libarray has a wrapper in the mock contract
 */
contract TargetUtilsMock {
    using TargetUtils for Target;

    Target public target;

    function setTarget(uint256 targetVal) public {
        target = Target.wrap(targetVal);
    }

    function getNumCapabilityPermissions() pure public returns (uint256) {
        return TargetUtils.getNumCapabilityPermissions();
    }

    function getTargetAddress() view public returns (address) {
        return TargetUtils.getTargetAddress(target);
    }

    function getTargetClearance() view public returns (Clearance) {
        return TargetUtils.getTargetClearance(target);
    }

    function getTargetType() view public returns (TargetType) {
        return TargetUtils.getTargetType(target);
    }

    function isTargetType(TargetType targetType) view public returns (bool) {
        return TargetUtils.isTargetType(target, targetType);
    }

    function getDefaultTargetPermission() view public returns (TargetPermission) {
        return TargetUtils.getDefaultTargetPermission(target);
    }

    function getDefaultCapabilityPermissionAt(uint256 position) view public returns (CapabilityPermission) {
        return TargetUtils.getDefaultCapabilityPermissionAt(target, position);
    }

    function forceWriteAsTargetType(TargetType targetType) view public returns (Target) {
        return TargetUtils.forceWriteAsTargetType(target, targetType);
    }

    function forceWriteTargetAddress(address targetAddress) view public returns (Target) {
        return TargetUtils.forceWriteTargetAddress(target, targetAddress);
    }

    function encodeDefaultPermissions(
        address targetAddress,
        Clearance clearance,
        TargetType targetType,
        TargetPermission targetPermission,
        CapabilityPermission[] memory functionPermissions
    ) pure public returns (Target) {
        return TargetUtils.encodeDefaultPermissions(
            targetAddress,
            clearance,
            targetType,
            targetPermission,
            functionPermissions
        );
    }

    function decodeDefaultPermissions() view public returns (
        address targetAddress,
        Clearance clearance,
        TargetType targetType,
        TargetPermission targetPermission,
        CapabilityPermission[] memory functionPermissions
    ) {
        return TargetUtils.decodeDefaultPermissions(
            target
        );
    }

    function convertFunctionToTargetPermission(CapabilityPermission functionPermission) pure public returns (TargetPermission) {
        return TargetUtils.convertFunctionToTargetPermission(functionPermission);
    }
}