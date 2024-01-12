// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "../mocks/TargetUtilsMock.sol";

contract TargetUtilsTest is Test {
    bytes32 private constant TARGET_ADDRESS_MASK = hex"ffffffffffffffffffffffffffffffffffffffff000000000000000000000000";
    bytes32 private constant TARGET_CLEARANCE_MASK =
        hex"0000000000000000000000000000000000000000ff0000000000000000000000";
    bytes32 private constant TARGET_TYPE_MASK = hex"000000000000000000000000000000000000000000ff00000000000000000000";
    bytes32 private constant TARGET_DEFAULT_MASK = hex"00000000000000000000000000000000000000000000ff000000000000000000";

    TargetUtilsMock public targetUtilsMock;

    function setUp() public {
        targetUtilsMock = new TargetUtilsMock();
    }

    function test_GetNumDefaultFunctionPermissions() public {
        uint256 num = targetUtilsMock.getNumCapabilityPermissions();
        assertEq(num, 9);
    }

    function testFuzz_GetTargetAddress(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedTargetAddress = bytes32(targetVal) & TARGET_ADDRESS_MASK;

        assertEq(bytes32(uint256(uint160(targetUtilsMock.getTargetAddress())) << 96), maskedTargetAddress);
        assertEq(targetUtilsMock.getTargetAddress(), address(uint160(uint256(maskedTargetAddress >> 96))));
    }

    function testFuzz_GetTargetClearance(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedTargetClearance = bytes32(targetVal) & TARGET_CLEARANCE_MASK;

        uint8 convertedMaskedTargetClearance = uint8(uint256(maskedTargetClearance >> 88));
        vm.assume(convertedMaskedTargetClearance <= uint8(type(Clearance).max)); // valid target clearance

        assertEq(bytes32(uint256(uint8(targetUtilsMock.getTargetClearance())) << 88), maskedTargetClearance);
        assertEq(uint8(targetUtilsMock.getTargetClearance()), convertedMaskedTargetClearance);
    }

    function testRevert_GetTargetClearance(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedTargetClearance = bytes32(targetVal) & TARGET_CLEARANCE_MASK;

        uint8 convertedMaskedTargetClearance = uint8(uint256(maskedTargetClearance >> 88));
        vm.assume(convertedMaskedTargetClearance > uint8(type(Clearance).max)); // valid target clearance

        vm.expectRevert(stdError.enumConversionError);
        targetUtilsMock.getTargetClearance();
    }

    function testFuzz_GetTargetType(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType <= uint8(type(TargetType).max)); // valid target type

        assertEq(bytes32(uint256(uint8(targetUtilsMock.getTargetType())) << 80), maskedTargetType);
        assertEq(uint8(targetUtilsMock.getTargetType()), convertedMaskedTargetType);
    }

    function testRevert_GetTargetType(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType > uint8(type(TargetType).max)); // valid target type

        vm.expectRevert(stdError.enumConversionError);
        targetUtilsMock.getTargetType();
    }

    function testFuzz_IsTargetType(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType <= uint8(type(TargetType).max)); // valid target type
        assertTrue(targetUtilsMock.isTargetType(TargetType(convertedMaskedTargetType)));
        assertFalse(
            targetUtilsMock.isTargetType(TargetType((convertedMaskedTargetType + 1) % uint8(type(TargetType).max)))
        );
    }

    function testRevert_IsTargetType(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedTargetType = bytes32(targetVal) & TARGET_TYPE_MASK;

        uint8 convertedMaskedTargetType = uint8(uint256(maskedTargetType >> 80));
        vm.assume(convertedMaskedTargetType > uint8(type(TargetType).max));

        vm.expectRevert(stdError.enumConversionError);
        targetUtilsMock.isTargetType(TargetType(convertedMaskedTargetType));
    }

    function testFuzz_GetDefaultTargetPermission(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);
        bytes32 maskedDefaultPermission = bytes32(targetVal) & TARGET_DEFAULT_MASK;

        uint8 convertedMaskedDefaultPermission = uint8(uint256(maskedDefaultPermission >> 72));
        vm.assume(convertedMaskedDefaultPermission <= uint8(type(TargetPermission).max)); // valid target permission

        assertEq(bytes32(uint256(uint8(targetUtilsMock.getDefaultTargetPermission())) << 72), maskedDefaultPermission);
        assertEq(uint8(targetUtilsMock.getDefaultTargetPermission()), convertedMaskedDefaultPermission);
    }

    function testFuzz_GetDefaultPermissionAt(uint256 targetVal, uint256 position) public {
        targetUtilsMock.setTarget(targetVal);
        uint256 limit = targetUtilsMock.getNumCapabilityPermissions() - 1;
        position = bound(position, 0, limit);

        uint8 convertedMaskedDefaultPermissionAt = uint8((targetVal << 184 + position * 8) >> 248);
        vm.assume(convertedMaskedDefaultPermissionAt <= uint8(type(CapabilityPermission).max)); // valid target
            // permission

        CapabilityPermission permission = targetUtilsMock.getDefaultCapabilityPermissionAt(position);
        assertEq(uint8(permission), convertedMaskedDefaultPermissionAt);
    }

    function testRevert_GetDefaultPermissionAt(uint256 offset) public {
        offset = bound(offset, 1, 1e36);

        uint256 position = targetUtilsMock.getNumCapabilityPermissions() + offset;
        vm.expectRevert(TooManyCapabilities.selector);
        targetUtilsMock.getDefaultCapabilityPermissionAt(position);
    }

    function testFuzz_WriteAsTargetType(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions,
        uint8 asTargetType
    )
        public
    {
        // bound target type
        asTargetType = uint8(bound(asTargetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
        TargetType newTargetType = TargetType(asTargetType);
        // get valid target
        (
            uint8 boundClearance,
            uint8 boundTargetType,
            uint8 boundTargetPermission,
            uint8[] memory boundFunctionPermissions
        ) = _helperCreateValidTarget(targetAddress, clearance, targetType, targetPermission, functionPermissions);
        // force write
        Target newTarget = targetUtilsMock.forceWriteAsTargetType(newTargetType);
        TargetUtilsMock newTargetUtilsMock = new TargetUtilsMock();
        newTargetUtilsMock.setTarget(Target.unwrap(newTarget));

        // verify invariant state updates
        assertEq(newTargetUtilsMock.getTargetAddress(), targetUtilsMock.getTargetAddress());
        assertEq(uint8(newTargetUtilsMock.getTargetClearance()), uint8(targetUtilsMock.getTargetClearance()));
        assertEq(
            uint8(newTargetUtilsMock.getDefaultTargetPermission()), uint8(targetUtilsMock.getDefaultTargetPermission())
        );
        assertEq(uint8(newTargetUtilsMock.getTargetType()), uint8(newTargetType));
        // depending on the asTargetType, certain permissions are overwritten
        if (newTargetType == TargetType.CHANNELS) {
            for (uint256 i = 0; i < 7; i++) {
                assertEq(
                    uint8(newTargetUtilsMock.getDefaultCapabilityPermissionAt(i)),
                    uint8(targetUtilsMock.getDefaultCapabilityPermissionAt(i))
                );
            }
            // indexes 7 and 8 are overwritten
            assertEq(uint8(newTargetUtilsMock.getDefaultCapabilityPermissionAt(7)), uint8(CapabilityPermission.NONE));
            assertEq(uint8(newTargetUtilsMock.getDefaultCapabilityPermissionAt(8)), uint8(CapabilityPermission.NONE));
        } else if (newTargetType == TargetType.TOKEN) {
            // indexes 0 - 6 are overwritten
            for (uint256 j = 0; j < 7; j++) {
                assertEq(
                    uint8(newTargetUtilsMock.getDefaultCapabilityPermissionAt(j)), uint8(CapabilityPermission.NONE)
                );
            }
            assertEq(
                uint8(newTargetUtilsMock.getDefaultCapabilityPermissionAt(7)),
                uint8(targetUtilsMock.getDefaultCapabilityPermissionAt(7))
            );
            assertEq(
                uint8(newTargetUtilsMock.getDefaultCapabilityPermissionAt(8)),
                uint8(targetUtilsMock.getDefaultCapabilityPermissionAt(8))
            );
        } else {
            // TargetType.SEND
            for (uint256 k = 0; k < targetUtilsMock.getNumCapabilityPermissions(); k++) {
                assertEq(
                    uint8(newTargetUtilsMock.getDefaultCapabilityPermissionAt(k)), uint8(CapabilityPermission.NONE)
                );
            }
        }
    }

    function testFuzz_WriteTargetAddress(uint256 targetVal, address newTargetAddress) public {
        targetUtilsMock.setTarget(targetVal);
        Target newTarget = targetUtilsMock.forceWriteTargetAddress(newTargetAddress);
        TargetUtilsMock newTargetUtilsMock = new TargetUtilsMock();
        newTargetUtilsMock.setTarget(Target.unwrap(newTarget));
        assertEq(newTargetUtilsMock.getTargetAddress(), newTargetAddress);
    }

    function testFuzz_EncodeDefaultPermissions(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions
    )
        public
    {
        (
            uint8 boundClearance,
            uint8 boundTargetType,
            uint8 boundTargetPermission,
            uint8[] memory boundFunctionPermissions
        ) = _helperCreateValidTarget(targetAddress, clearance, targetType, targetPermission, functionPermissions);
        // evaluate that target equals
        assertEq(targetUtilsMock.getTargetAddress(), targetAddress);
        assertEq(uint8(targetUtilsMock.getTargetClearance()), boundClearance);
        assertEq(uint8(targetUtilsMock.getTargetType()), boundTargetType);
        assertEq(uint8(targetUtilsMock.getDefaultTargetPermission()), boundTargetPermission);
        for (uint256 index = 0; index < functionPermissions.length; index++) {
            assertEq(uint8(targetUtilsMock.getDefaultCapabilityPermissionAt(index)), boundFunctionPermissions[index]);
        }
    }

    function testRevert_EncodeDefaultPermissions(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions
    )
        public
    {
        vm.assume(functionPermissions.length > targetUtilsMock.getNumCapabilityPermissions());
        // bound to each enum type
        CapabilityPermission[] memory funcPermissions = new CapabilityPermission[](functionPermissions.length);
        clearance = uint8(bound(clearance, uint256(type(Clearance).min), uint256(type(Clearance).max)));
        targetType = uint8(bound(targetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
        targetPermission =
            uint8(bound(targetPermission, uint256(type(TargetPermission).min), uint256(type(TargetPermission).max)));
        for (uint256 i = 0; i < functionPermissions.length; i++) {
            functionPermissions[i] = uint8(
                bound(
                    functionPermissions[i],
                    uint256(type(CapabilityPermission).min),
                    uint256(type(CapabilityPermission).max)
                )
            );
            funcPermissions[i] = CapabilityPermission(functionPermissions[i]);
        }

        vm.expectRevert(TooManyCapabilities.selector);
        // get the target
        targetUtilsMock.encodeDefaultPermissions(
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
    )
        public
    {
        (
            uint8 boundClearance,
            uint8 boundTargetType,
            uint8 boundTargetPermission,
            uint8[] memory boundFunctionPermissions
        ) = _helperCreateValidTarget(targetAddress, clearance, targetType, targetPermission, functionPermissions);
        // evaluate that target equals
        assertEq(targetUtilsMock.getTargetAddress(), targetAddress);
        assertEq(uint8(targetUtilsMock.getTargetClearance()), boundClearance);
        assertEq(uint8(targetUtilsMock.getTargetType()), boundTargetType);
        assertEq(uint8(targetUtilsMock.getDefaultTargetPermission()), boundTargetPermission);
        for (uint256 index = 0; index < functionPermissions.length; index++) {
            assertEq(uint8(targetUtilsMock.getDefaultCapabilityPermissionAt(index)), boundFunctionPermissions[index]);
        }
    }

    function testRevert_DecodeDefaultPermissions(uint256 targetVal) public {
        targetUtilsMock.setTarget(targetVal);

        if ((targetVal << 160) >> 248 > uint256(type(Clearance).max)) {
            // clearance is incorrect
            vm.expectRevert(stdError.enumConversionError);
            targetUtilsMock.decodeDefaultPermissions();
        }

        if ((targetVal << 168) >> 248 > uint256(type(TargetType).max)) {
            // target type is incorrect
            vm.expectRevert(stdError.enumConversionError);
            targetUtilsMock.decodeDefaultPermissions();
        }

        if ((targetVal << 176) >> 248 > uint256(type(TargetPermission).max)) {
            // target permission is incorrect
            vm.expectRevert(stdError.enumConversionError);
            targetUtilsMock.decodeDefaultPermissions();
        }

        for (uint256 i = 0; i < targetUtilsMock.getNumCapabilityPermissions(); i++) {
            if (targetVal << (176 + 8 * i) >> 248 > uint256(type(CapabilityPermission).max)) {
                // target permission is incorrect
                vm.expectRevert(stdError.enumConversionError);
                targetUtilsMock.decodeDefaultPermissions();
            }
        }
    }

    function testFuzz_ConvertFunctionToTargetPermissions(uint8 functionPermissionVal) public {
        functionPermissionVal = uint8(
            bound(
                functionPermissionVal, uint256(type(CapabilityPermission).min), uint256(type(CapabilityPermission).max)
            )
        );
        vm.assume(functionPermissionVal != 0);

        TargetPermission targetPermission =
            targetUtilsMock.convertFunctionToTargetPermission(CapabilityPermission(functionPermissionVal));
        assertEq(uint8(targetPermission), functionPermissionVal - 1);
    }

    function testRevert_ConvertFunctionToTargetPermissions(uint8 functionPermissionVal) public {
        if (functionPermissionVal > uint8(type(CapabilityPermission).max)) {
            vm.expectRevert(stdError.enumConversionError);
            targetUtilsMock.convertFunctionToTargetPermission(CapabilityPermission(functionPermissionVal));
        }

        if (functionPermissionVal == 0) {
            vm.expectRevert(PermissionNotFound.selector);
            targetUtilsMock.convertFunctionToTargetPermission(CapabilityPermission(functionPermissionVal));
        }
    }

    function _helperCreateValidTarget(
        address targetAddress,
        uint8 clearance,
        uint8 targetType,
        uint8 targetPermission,
        uint8[] memory functionPermissions
    )
        private
        returns (
            uint8 boundClearance,
            uint8 boundTargetType,
            uint8 boundTargetPermission,
            uint8[] memory boundFunctionPermissions
        )
    {
        // otherwise revert
        vm.assume(functionPermissions.length <= targetUtilsMock.getNumCapabilityPermissions());

        // bound to each enum type
        boundFunctionPermissions = new uint8[](functionPermissions.length);
        CapabilityPermission[] memory funcPermissions = new CapabilityPermission[](functionPermissions.length);
        boundClearance = uint8(bound(clearance, uint256(type(Clearance).min), uint256(type(Clearance).max)));
        boundTargetType = uint8(bound(targetType, uint256(type(TargetType).min), uint256(type(TargetType).max)));
        boundTargetPermission =
            uint8(bound(targetPermission, uint256(type(TargetPermission).min), uint256(type(TargetPermission).max)));
        for (uint256 i = 0; i < functionPermissions.length; i++) {
            boundFunctionPermissions[i] = uint8(
                bound(
                    functionPermissions[i],
                    uint256(type(CapabilityPermission).min),
                    uint256(type(CapabilityPermission).max)
                )
            );
            funcPermissions[i] = CapabilityPermission(boundFunctionPermissions[i]);
        }

        // get the target
        Target target = targetUtilsMock.encodeDefaultPermissions(
            targetAddress,
            Clearance(boundClearance),
            TargetType(boundTargetType),
            TargetPermission(boundTargetPermission),
            funcPermissions
        );
        // set to mock
        targetUtilsMock.setTarget(Target.unwrap(target));
    }
}
