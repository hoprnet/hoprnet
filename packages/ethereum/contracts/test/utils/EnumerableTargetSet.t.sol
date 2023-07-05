pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Test.sol";
import "../../src/utils/TargetUtils.sol";

contract EnumerableTargetSetTest is Test {
    address public targetUtilsLibAddress;
    bytes32 private TARGET_ADDRESS_MASK =   hex"ffffffffffffffffffffffffffffffffffffffff000000000000000000000000";
    bytes32 private TARGET_CLEARANCE_MASK = hex"0000000000000000000000000000000000000000ff0000000000000000000000";
    bytes32 private TARGET_TYPE_MASK =      hex"000000000000000000000000000000000000000000ff00000000000000000000";
    bytes32 private TARGET_DEFAULT_MASK =   hex"00000000000000000000000000000000000000000000ff000000000000000000";
 
    using TargetUtils for Target;

    function setUp() public {
        targetUtilsLibAddress = deployCode("TargetUtils.sol:TargetUtils");
    }

    function test_GetNumDefaultFunctionPermissions() public {
        uint256 num = TargetUtils.getNumDefaultFunctionPermissions();
        assertEq(num, 9);
    }

    function testFuzz_GetTargetAddress(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedTargetAddress = bytes32(targetVal) & TARGET_ADDRESS_MASK;
        assertEq(bytes32(uint256(uint160(TargetUtils.getTargetAddress(target))) << 96), maskedTargetAddress);
        assertEq(TargetUtils.getTargetAddress(target), address(uint160(uint256(maskedTargetAddress >> 96))));
    }

    function testFuzz_GetTargetClearance(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedTargetClearance = bytes32(targetVal) & TARGET_CLEARANCE_MASK;

        uint8 convertedMaskedTargetClearance = uint8(uint256(maskedTargetClearance >> 88));
        vm.assume(convertedMaskedTargetClearance <= uint8(type(Clearance).max)); // valid target clearance

        assertEq(bytes32(uint256(uint8(TargetUtils.getTargetClearance(target))) << 88), maskedTargetClearance);
        assertEq(uint8(TargetUtils.getTargetClearance(target)), convertedMaskedTargetClearance);
    }

    function testRevert_GetTargetClearance(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedTargetClearance = bytes32(targetVal) & TARGET_CLEARANCE_MASK;

        uint8 convertedMaskedTargetClearance = uint8(uint256(maskedTargetClearance >> 88));
        vm.assume(convertedMaskedTargetClearance > uint8(type(Clearance).max)); // valid target clearance

        vm.expectRevert(stdError.enumConversionError);
        TargetUtils.getTargetClearance(target);
    }

    function testFuzz_GetTargetType(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType <= uint8(type(TargetType).max)); // valid target type

        assertEq(bytes32(uint256(uint8(TargetUtils.getTargetType(target))) << 80), maskedTargetType);
        assertEq(uint8(TargetUtils.getTargetType(target)), convertedMaskedTargetType);
    }

    function testRevert_GetTargetType(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType > uint8(type(TargetType).max)); // valid target type

        vm.expectRevert(stdError.enumConversionError);
        TargetUtils.getTargetType(target);
    }

    function testFuzz_IsTargetType(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType <= uint8(type(TargetType).max)); // valid target type
        assertTrue(TargetUtils.isTargetType(target, TargetType(convertedMaskedTargetType)));
        assertFalse(TargetUtils.isTargetType(target, TargetType((convertedMaskedTargetType + 1) % uint8(type(TargetType).max))));
    }

    function testRevert_IsTargetType(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType > uint8(type(TargetType).max));

        vm.expectRevert(stdError.enumConversionError);
        TargetUtils.isTargetType(target, TargetType(convertedMaskedTargetType));
    }

    function testFuzz_GetDefaultTargetPermission(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        bytes32 maskedDefaultPermission = bytes32(targetVal) & TARGET_DEFAULT_MASK;

        uint8 convertedMaskedDefaultPermission = uint8(uint256(maskedDefaultPermission >> 72));
        vm.assume(convertedMaskedDefaultPermission <= uint8(type(TargetPermission).max)); // valid target permission

        assertEq(bytes32(uint256(uint8(TargetUtils.getDefaultTargetPermission(target))) << 72), maskedDefaultPermission);
        assertEq(uint8(TargetUtils.getDefaultTargetPermission(target)), convertedMaskedDefaultPermission);
    }

    function testFuzz_GetDefaultPermissionAt(uint256 targetVal, uint256 position) public {
        Target target = Target.wrap(targetVal);
        uint256 limit = TargetUtils.getNumDefaultFunctionPermissions() - 1;
        position = bound(position, 0, limit);

        uint8 convertedMaskedDefaultPermissionAt = uint8((targetVal << 184 + position * 8) >> 248);
        vm.assume(convertedMaskedDefaultPermissionAt <= uint8(type(FunctionPermission).max)); // valid target permission

        FunctionPermission permission = TargetUtils.getDefaultFunctionPermissionAt(target, position);
        assertEq(uint8(permission), convertedMaskedDefaultPermissionAt);
    }

    function testRevert_GetDefaultPermissionAt(uint256 offset) public {
        offset = bound(offset, 1, 1e36);
        
        uint256 position = TargetUtils.getNumDefaultFunctionPermissions() + offset;
        vm.expectRevert(FunctionPermissionsTooMany.selector);
        TargetUtils.getDefaultFunctionPermissionAt(Target.wrap(0), position);
    }

    function testFuzz_WriteAsTargetType(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions,
        uint8 asTargetType
    ) public {
        // bound target type
        asTargetType = uint8(bound(asTargetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
        TargetType newTargetType = TargetType(asTargetType); 
        // get valid target
        (Target target, uint8 boundClearance, uint8 boundTargetType, uint8 boundTargetPermission, uint8[] memory boundFunctionPermissions) = _helperCreateValidTarget(targetAddress, clearance, targetType, targetPermission, functionPermissions);
        // force write 262
        uint256 gasStart = gasleft();
        Target newTarget = TargetUtils.forceWriteAsTargetType(target, newTargetType);
        uint256 gasEnd = gasleft();
        emit log_named_uint("gas used", gasStart - gasEnd);

        // verify invariant state updates 
        assertEq(TargetUtils.getTargetAddress(newTarget), TargetUtils.getTargetAddress(target));
        assertEq(uint8(TargetUtils.getTargetClearance(newTarget)), uint8(TargetUtils.getTargetClearance(target)));
        assertEq(uint8(TargetUtils.getDefaultTargetPermission(newTarget)), uint8(TargetUtils.getDefaultTargetPermission(target)));
        assertEq(uint8(TargetUtils.getTargetType(newTarget)), uint8(newTargetType));
        // depending on the asTargetType, certain permissions are overwritten
        if (newTargetType == TargetType.CHANNELS) {
            for (uint256 i = 0; i < 7; i++) {
                assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, i)), uint8(TargetUtils.getDefaultFunctionPermissionAt(target, i)));
            }
            // indexes 7 and 8 are overwritten
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 7)), uint8(FunctionPermission.NONE));
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 8)), uint8(FunctionPermission.NONE));
        } else if (newTargetType == TargetType.TOKEN) {
            // indexes 0 - 6 are overwritten
            for (uint256 j = 0; j < 7; j++) {
                assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, j)), uint8(FunctionPermission.NONE));
            }
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 7)), uint8(TargetUtils.getDefaultFunctionPermissionAt(target, 7)));
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 8)), uint8(TargetUtils.getDefaultFunctionPermissionAt(target, 8)));
        } else {
            // TargetType.SEND
            for (uint256 k = 0; k < TargetUtils.getNumDefaultFunctionPermissions(); k++) {
                assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, k)), uint8(FunctionPermission.NONE));
            }
        }
    }
    function testFuzz_WriteAsTargetType2(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions,
        uint8 asTargetType
    ) public {
        // bound target type
        asTargetType = uint8(bound(asTargetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
        TargetType newTargetType = TargetType(asTargetType); 
        // get valid target
        (Target target, uint8 boundClearance, uint8 boundTargetType, uint8 boundTargetPermission, uint8[] memory boundFunctionPermissions) = _helperCreateValidTarget(targetAddress, clearance, targetType, targetPermission, functionPermissions);
        // force write 262
        uint256 gasStart = gasleft();
        Target newTarget = TargetUtils.forceWriteAsTargetType2(target, newTargetType);
        uint256 gasEnd = gasleft();
        emit log_named_uint("gas used", gasStart - gasEnd);

        // verify invariant state updates 
        assertEq(TargetUtils.getTargetAddress(newTarget), TargetUtils.getTargetAddress(target));
        assertEq(uint8(TargetUtils.getTargetClearance(newTarget)), uint8(TargetUtils.getTargetClearance(target)));
        assertEq(uint8(TargetUtils.getDefaultTargetPermission(newTarget)), uint8(TargetUtils.getDefaultTargetPermission(target)));
        assertEq(uint8(TargetUtils.getTargetType(newTarget)), uint8(newTargetType));
        // depending on the asTargetType, certain permissions are overwritten
        if (newTargetType == TargetType.CHANNELS) {
            for (uint256 i = 0; i < 7; i++) {
                assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, i)), uint8(TargetUtils.getDefaultFunctionPermissionAt(target, i)));
            }
            // indexes 7 and 8 are overwritten
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 7)), uint8(FunctionPermission.NONE));
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 8)), uint8(FunctionPermission.NONE));
        } else if (newTargetType == TargetType.TOKEN) {
            // indexes 0 - 6 are overwritten
            for (uint256 j = 0; j < 7; j++) {
                assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, j)), uint8(FunctionPermission.NONE));
            }
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 7)), uint8(TargetUtils.getDefaultFunctionPermissionAt(target, 7)));
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, 8)), uint8(TargetUtils.getDefaultFunctionPermissionAt(target, 8)));
        } else {
            // TargetType.SEND
            for (uint256 k = 0; k < TargetUtils.getNumDefaultFunctionPermissions(); k++) {
                assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(newTarget, k)), uint8(FunctionPermission.NONE));
            }
        }
    }

    function testFuzz_EncodeDefaultPermissions(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions
    ) public {     
        (Target target, uint8 boundClearance, uint8 boundTargetType, uint8 boundTargetPermission, uint8[] memory boundFunctionPermissions) = _helperCreateValidTarget(targetAddress, clearance, targetType, targetPermission, functionPermissions);
        // evaluate that target equals
        assertEq(TargetUtils.getTargetAddress(target), targetAddress);
        assertEq(uint8(TargetUtils.getTargetClearance(target)), boundClearance);
        assertEq(uint8(TargetUtils.getTargetType(target)), boundTargetType);
        assertEq(uint8(TargetUtils.getDefaultTargetPermission(target)), boundTargetPermission);
        for (uint256 index = 0; index < functionPermissions.length; index++) {
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(target, index)), boundFunctionPermissions[index]);
        }
    }

    function testRevert_EncodeDefaultPermissions(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions
    ) public {
        
        vm.assume(functionPermissions.length > TargetUtils.getNumDefaultFunctionPermissions());
        // bound to each enum type
        FunctionPermission[] memory funcPermissions = new FunctionPermission[](functionPermissions.length);
        clearance = uint8(bound(clearance, uint256(type(Clearance).min), uint256(type(Clearance).max)));
        targetType = uint8(bound(targetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
        targetPermission = uint8(bound(targetPermission, uint256(type(TargetPermission).min), uint256(type(TargetPermission).max)));
        for (uint256 i = 0; i < functionPermissions.length; i++) {
            functionPermissions[i] = uint8(bound(functionPermissions[i], uint256(type(FunctionPermission).min), uint256(type(FunctionPermission).max)));
            funcPermissions[i] = FunctionPermission(functionPermissions[i]);
        }

        vm.expectRevert(FunctionPermissionsTooMany.selector);
        // get the target
        TargetUtils.encodeDefaultPermissions(
            targetAddress, 
            Clearance(clearance), 
            TargetType(targetType), 
            TargetPermission(targetPermission), 
            funcPermissions
        );
    }

    function testFuzz_DecodeDefaultPermissions(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions
    ) public {
        (Target target, uint8 boundClearance, uint8 boundTargetType, uint8 boundTargetPermission, uint8[] memory boundFunctionPermissions) = _helperCreateValidTarget(targetAddress, clearance, targetType, targetPermission, functionPermissions);
        // evaluate that target equals
        assertEq(TargetUtils.getTargetAddress(target), targetAddress);
        assertEq(uint8(TargetUtils.getTargetClearance(target)), boundClearance);
        assertEq(uint8(TargetUtils.getTargetType(target)), boundTargetType);
        assertEq(uint8(TargetUtils.getDefaultTargetPermission(target)), boundTargetPermission);
        for (uint256 index = 0; index < functionPermissions.length; index++) {
            assertEq(uint8(TargetUtils.getDefaultFunctionPermissionAt(target, index)), boundFunctionPermissions[index]);
        }
    }

    function _helperCreateValidTarget(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions
    ) private returns (
        Target target,
        uint8 boundClearance,
        uint8 boundTargetType,
        uint8 boundTargetPermission,
        uint8[] memory boundFunctionPermissions
    ){
        // otherwise revert
        vm.assume(functionPermissions.length <= TargetUtils.getNumDefaultFunctionPermissions());
   
        // bound to each enum type
        boundFunctionPermissions = new uint8[](functionPermissions.length);
        FunctionPermission[] memory funcPermissions = new FunctionPermission[](functionPermissions.length);
        boundClearance = uint8(bound(clearance, uint256(type(Clearance).min), uint256(type(Clearance).max)));
        boundTargetType = uint8(bound(targetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
        boundTargetPermission = uint8(bound(targetPermission, uint256(type(TargetPermission).min), uint256(type(TargetPermission).max)));
        for (uint256 i = 0; i < functionPermissions.length; i++) {
            boundFunctionPermissions[i] = uint8(bound(functionPermissions[i], uint256(type(FunctionPermission).min), uint256(type(FunctionPermission).max)));
            funcPermissions[i] = FunctionPermission(boundFunctionPermissions[i]);
        }

        // get the target
        target = TargetUtils.encodeDefaultPermissions(
            targetAddress, 
            Clearance(boundClearance), 
            TargetType(boundTargetType), 
            TargetPermission(boundTargetPermission), 
            funcPermissions
        );
    }

    // function _helperBindTypes(
    //     uint8 clearance,
    //     uint8 targetType,
    //     uint8 targetPermission,
    //     uint8[] memory functionPermissions
    // ) private returns (
    //     uint8 boundClearance,
    //     uint8 boundTargetType,
    //     uint8 boundTargetPermission,
    //     uint8[] memory boundFunctionPermissions,
    //     FunctionPermission[] memory funcPermissions
    // ){
    //     // bound to each enum type
    //     boundFunctionPermissions = new uint8[](functionPermissions.length);
    //     funcPermissions = new FunctionPermission[](functionPermissions.length);
    //     FunctionPermission[] memory funcPermissions = new FunctionPermission[](functionPermissions.length);
    //     boundClearance = uint8(bound(clearance, uint256(type(Clearance).min), uint256(type(Clearance).max)));
    //     boundTargetType = uint8(bound(targetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
    //     boundTargetPermission = uint8(bound(targetPermission, uint256(type(TargetPermission).min), uint256(type(TargetPermission).max)));
    //     for (uint256 i = 0; i < functionPermissions.length; i++) {
    //         boundFunctionPermissions[i] = uint8(bound(functionPermissions[i], uint256(type(FunctionPermission).min), uint256(type(FunctionPermission).max)));
    //         funcPermissions[i] = FunctionPermission(boundFunctionPermissions[i]);
    //     }
    // }
}
