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

    function getNumDefaultFunctionPermissions() public returns (uint256) {
        return TargetUtils.getNumDefaultFunctionPermissions();
    }

    function getTargetAddress() public returns (address) {
        return TargetUtils.getTargetAddress(target);
    }

    function getTargetClearance() public returns (Clearance) {
        return TargetUtils.getTargetClearance(target);
    }

    function getTargetType() public returns (TargetType) {
        return TargetUtils.getTargetType(target);
    }

    function isTargetType(TargetType targetType) public returns (bool) {
        return TargetUtils.isTargetType(target, targetType);
    }

    function getDefaultTargetPermission() public returns (TargetPermission) {
        return TargetUtils.getDefaultTargetPermission(target);
    }

    function getDefaultFunctionPermissionAt(uint256 position) public returns (FunctionPermission) {
        return TargetUtils.getDefaultFunctionPermissionAt(target, position);
    }

    function forceWriteAsTargetType(TargetType targetType) public returns (Target) {
        return TargetUtils.forceWriteAsTargetType(target, targetType);
    }

    function forceWriteTargetAddress(address targetAddress) public returns (Target) {
        return TargetUtils.forceWriteTargetAddress(target, targetAddress);
    }

    function encodeDefaultPermissions(
        address targetAddress,
        Clearance clearance,
        TargetType targetType,
        TargetPermission targetPermission,
        FunctionPermission[] memory functionPermissions
    ) public returns (Target) {
        return TargetUtils.encodeDefaultPermissions(
            targetAddress,
            clearance,
            targetType,
            targetPermission,
            functionPermissions
        );
    }

    function decodeDefaultPermissions() public returns (
        address targetAddress,
        Clearance clearance,
        TargetType targetType,
        TargetPermission targetPermission,
        FunctionPermission[] memory functionPermissions
    ) {
        return TargetUtils.decodeDefaultPermissions(
            target
        );
    }

    function convertFunctionToTargetPermission(FunctionPermission functionPermission) public returns (TargetPermission) {
        return TargetUtils.convertFunctionToTargetPermission(functionPermission);
    }
}