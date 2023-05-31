// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import '../../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import 'forge-std/Test.sol';

contract HoprCapabilityPermissionsTest is Test {
    Role internal role;

    /**
    * Manually import events and errors
    */
    error AddressIsZero();
    event ScopedTargetToken(address targetAddress);

    function setUp() public virtual {}

    /**
    * @dev Failes to add token target(s) when the account is not address zero
    */
    function testRevert_WhenAddressZeroAddTargetToken() public {
        vm.expectRevert(AddressIsZero.selector);
        HoprCapabilityPermissions.scopeTargetToken(role, address(0));
    }

    /**
    * @dev Add token target(s) when the account is not address zero
    */
    function testFuzz_AddTargetToken(address account) public {
        vm.assume(account != address(0));

        vm.expectEmit(true, false, false, false, address(this));
        emit ScopedTargetToken(account);
        HoprCapabilityPermissions.scopeTargetToken(role, account);

        assertEq(uint256(role.targets[account].clearance), uint256(Clearance.Function), "wrong clearance added");
        assertEq(uint256(role.targets[account].targetType), uint256(TargetType.Token), "wrong target type added");
    }

    /**
    * @dev Encode an array of permission enums into uint256 and vice versa
    */
    function testFuzz_EncodePermissionEnums(uint256 length, bool startWithZero) public {
        // length must not exceed 256
        vm.assume(length <= 256);
        // create a permission array that alternates between 0 and 1
        uint256[] memory permissions = new uint256[](length);
        for (uint256 i = 0; i < length; i++) {
            permissions[i] = startWithZero == (i % 2 == 0) ? 0 : 1;
        }

        (uint256 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodePermissionEnums(permissions);
        (uint256[] memory decodedPermissions) = HoprCapabilityPermissions.decodePermissionEnums(encodedValue, encodedLength);

        assertEq(encodedLength, length, "Encoding length is wrong");
        assertEq(decodedPermissions.length, length, "Decoded length is wrong");

        for (uint256 j = 0; j < length; j++) {
            assertEq(permissions[j], decodedPermissions[j], "Element changes during the process");
        }
    }
}
