// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import '../../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import "../../utils/CapabilityLibrary.sol";
import 'forge-std/Test.sol';

contract HoprCapabilityPermissionsTest is Test, CapabilityPermissionsLibFixtureTest {
    Role internal role;

    /**
    * Manually import events and errors
    */
    error AddressIsZero();
    event ScopedTargetToken(address targetAddress);

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
    function testFuzz_EncodeAndDecodePermissionEnums(uint256 length, bool startWithZero) public {
        // length must not exceed 256
        vm.assume(length <= 256);
        // create a permission array that alternates between 0 and 1
        uint256[] memory permissions = _helperCreateHoprChannelsPermissionsArray(length, startWithZero);

        (uint256 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodePermissionEnums(permissions);
        (uint256[] memory decodedPermissions) = HoprCapabilityPermissions.decodePermissionEnums(encodedValue, encodedLength);

        assertEq(encodedLength, length, "Encoding length is wrong");
        assertEq(decodedPermissions.length, length, "Decoded length is wrong");

        for (uint256 j = 0; j < length; j++) {
            assertEq(decodedPermissions[j], permissions[j], "Element changes during the process");
        }
    }

    /**
    * @dev Encode an array of funciton signatures (max. 7) into a bytes32
    */
    function test_EncodeAndDecodeFunctionSigs() public {
        // create an array of funciton sigatures
        bytes4[] memory functionSigs = _helperCreateHoprChannelsFunctionSigArray();

        (bytes32 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodeFunctionSigs(functionSigs);
        (bytes4[] memory decoded) = HoprCapabilityPermissions.decodeFunctionSigs(encodedValue, encodedLength);

        assertEq(encodedLength, decoded.length, "Length is wrong");

        for (uint256 j = 0; j < encodedLength; j++) {
            assertEq(decoded[j], functionSigs[j], "Element changes during the process");
        }
    }

    /**
    * @dev Encode an array of funciton signatures (max. 7) into a bytes32. Test with 6
    */
    function test_EncodeAndDecodeSigsAndPermissions(bool startWithZero) public {
        uint256 length = 6;
        // create an array of funciton sigatures
        bytes4[] memory functionSigs = _helperCreateHoprChannelsFunctionSigArray();
        uint256[] memory permissions = _helperCreateHoprChannelsPermissionsArray(length, startWithZero);

        (bytes32 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
        emit log_named_bytes32("encodedValue", encodedValue);
        (bytes4[] memory decodedSigs, uint256[] memory decodedPermissions) = HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedValue, encodedLength);

        assertEq(encodedLength, length, "Encoding length is wrong");
        assertEq(decodedSigs.length, length, "Decoded sigs length is wrong");
        assertEq(decodedPermissions.length, length, "Decoded permissions length is wrong");

        for (uint256 j = 0; j < length; j++) {
            assertEq(decodedSigs[j], functionSigs[j], "Sig changes during the process");
        }
        for (uint256 k = 0; k < length; k++) {
            assertEq(decodedPermissions[k], permissions[k], "Permission changes during the process");
        }
    }

    /**
     * @dev create a permission array that alternates between 0 and 1
     */
    function _helperCreateHoprChannelsPermissionsArray(uint256 length, bool startWithZero) private pure returns (uint256[] memory permissions) {
        permissions = new uint256[](length);
        for (uint256 i = 0; i < length; i++) {
            permissions[i] = startWithZero == (i % 2 == 0) ? 0 : 1;
        }
    }

    /**
     * @dev create an array of funciton sigatures for HoprChannels
     */
    function _helperCreateHoprChannelsFunctionSigArray() private pure returns (bytes4[] memory functionSigs) {
        functionSigs = new bytes4[](7);
        functionSigs[0] = HoprCapabilityPermissions.REDEEM_TICKET_SELECTOR;
        functionSigs[1] = HoprCapabilityPermissions.BATCH_REDEEM_TICKETS_SELECTOR;
        functionSigs[2] = HoprCapabilityPermissions.CLOSE_INCOMING_CHANNEL_SELECTOR;
        functionSigs[3] = HoprCapabilityPermissions.INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR;
        functionSigs[4] = HoprCapabilityPermissions.FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR;
        functionSigs[5] = HoprCapabilityPermissions.FUND_CHANNEL_MULTI_SELECTOR;
        functionSigs[6] = HoprCapabilityPermissions.SET_COMMITMENT_SELECTOR;
    }
}
