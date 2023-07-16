// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import '../../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import "../../utils/CapabilityLibrary.sol";
import 'forge-std/Test.sol';

contract HoprCapabilityPermissionsTest is Test, CapabilityPermissionsLibFixtureTest {
    using TargetUtils for Target;
    using EnumerableTargetSet for TargetSet;

    using HoprCapabilityPermissions for Role;
    Role internal role;

    CapabilityPermission[] internal defaultFunctionPermission;
    uint256 maxCapabilitySize;

    // /**
    // * Manually import events and errors
    // */
    // event RevokedTarget(address indexed targetAddress);
    // event ScopedTargetChannels(address indexed targetAddress, Target target);
    // event ScopedTargetToken(address indexed targetAddress, Target target);
    // event ScopedTargetSend(address indexed targetAddress, Target target);
    // event ScopedGranularChannelCapability(
    //     address indexed targetAddress,
    //     bytes32 indexed channelId,
    //     bytes4 selector,
    //     GranularPermission permission
    // );
    // event ScopedGranularTokenCapability(
    //     address indexed targetAddress,
    //     address indexed recipientAddress,
    //     bytes4 selector,
    //     GranularPermission permission
    // );
    // event ScopedGranularSendCapability(
    //     address indexed recipientAddress,
    //     GranularPermission permission
    // );

    function setUp() public virtual override(CapabilityPermissionsLibFixtureTest) {
        super.setUp();
        maxCapabilitySize = TargetUtils.getNumCapabilityPermissions();
        defaultFunctionPermission = _helperCreateDefaultFunctionPermissionsArray(maxCapabilitySize, 0);
    }

    function test_CheckCapabilitySize() public {
        assertGt(maxCapabilitySize, 8);
    }

    /** FIXME: expect revert data
    * @dev Failes to add token target(s) when the account is not address zero
    */
    function testRevert_WhenAddressZeroAddTargetToken() public {
        Target tokenTarget = TargetUtils.encodeDefaultPermissions(
            address(0),
            Clearance.FUNCTION,
            TargetType.TOKEN,
            TargetPermission.BLOCK_ALL,
            defaultFunctionPermission
        );
        // vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector); // FIXME: revert with specific error
        vm.expectRevert();
        role.scopeTargetToken(tokenTarget);

        // alternative for testing this function is with:
        // (bool success, bytes memory result) = address(capabilityLibraryLibAddress).delegatecall(
        //   abi.encodeWithSelector(HoprCapabilityPermissions.scopeTargetToken.selector, tokenTarget)
        // );
        // assertFalse(success);
        // assertEq(bytes32(result), HoprCapabilityPermissions.AddressIsZero.selector);
    }

    /**
     *@dev test if the overwriting for token type results in the same value
    */
    function testFuzz_forceWriteAsTokenTargetType(address targetAddress) public {
        Target tokenTarget = TargetUtils.encodeDefaultPermissions(
            targetAddress,
            Clearance.FUNCTION,
            TargetType.TOKEN,
            TargetPermission.BLOCK_ALL,
            defaultFunctionPermission
        );

        CapabilityPermission[] memory actualFunctionPermissions = new CapabilityPermission[](9);
        actualFunctionPermissions[0] = CapabilityPermission.NONE;
        actualFunctionPermissions[1] = CapabilityPermission.NONE;
        actualFunctionPermissions[2] = CapabilityPermission.NONE;
        actualFunctionPermissions[3] = CapabilityPermission.NONE;
        actualFunctionPermissions[4] = CapabilityPermission.NONE;
        actualFunctionPermissions[5] = CapabilityPermission.NONE;
        actualFunctionPermissions[7] = defaultFunctionPermission[7];
        actualFunctionPermissions[8] = defaultFunctionPermission[8];

        Target actualTokenTarget = TargetUtils.encodeDefaultPermissions(
            targetAddress,
            Clearance.FUNCTION,
            TargetType.TOKEN,
            TargetPermission.BLOCK_ALL,
            actualFunctionPermissions
        );

        // evaluate the target mask equals
        assertEq(bytes32(Target.unwrap(actualTokenTarget)), bytes32(Target.unwrap(tokenTarget)) & hex"ffffffffffffffffffffffffffffffffffffffffffffff00000000000000ffff", "target is overwritten with mask as expected");
        assertEq(bytes32(Target.unwrap(actualTokenTarget)), bytes32(Target.unwrap(tokenTarget.forceWriteAsTargetType(TargetType.TOKEN))), "target is overwritten by forceWriteTargetAddress as expected");
    }

    // /**
    // * @dev Add token target(s) when the account is not address zero
    // */
    // function test_AddTargetToken() public {
    //     address targetAddress = vm.addr(1);
    //     Target tokenTarget = TargetUtils.encodeDefaultPermissions(
    //         targetAddress,
    //         Clearance.FUNCTION,
    //         TargetType.TOKEN,
    //         TargetPermission.BLOCK_ALL,
    //         defaultFunctionPermission
    //     );
    //     // CapabilityPermission[] memory actualFunctionPermissions = new CapabilityPermission[](defaultFunctionPermission.length);
    //     CapabilityPermission[] memory actualFunctionPermissions = defaultFunctionPermission;
    //     assertEq(actualFunctionPermissions.length, defaultFunctionPermission.length);
    //     assertEq(actualFunctionPermissions.length, 9);

    //     for (uint256 i = 0; i < 7; i++) {
    //         actualFunctionPermissions[i] = CapabilityPermission.NONE;
    //     }
    //     actualFunctionPermissions[7] = defaultFunctionPermission[7];
    //     actualFunctionPermissions[8] = defaultFunctionPermission[8];

    //     Target actualTokenTarget = TargetUtils.encodeDefaultPermissions(
    //         targetAddress,
    //         Clearance.FUNCTION,
    //         TargetType.TOKEN,
    //         TargetPermission.BLOCK_ALL,
    //         actualFunctionPermissions
    //     );
    //     emit log_named_bytes32("tokenTarget", bytes32(Target.unwrap(tokenTarget)));
    //     emit log_named_bytes32("actualTokenTarget", bytes32(Target.unwrap(actualTokenTarget)));

    //     // evaluate the target mask equals
    //     assertEq(bytes32(Target.unwrap(actualTokenTarget)), bytes32(Target.unwrap(tokenTarget)) & hex"ffffffffffffffffffffffffffffffffffffffffffffff00000000000000ffff", "target is not overwritten as expected");
    //     emit log_string("Passed: evaluate the target mask equals");

    //     // vm.expectEmit(true, false, false, true, address(this));
    //     // emit HoprCapabilityPermissions.ScopedTargetToken(targetAddress, actualTokenTarget);
    //     role.scopeTargetToken(tokenTarget);

    //     // (bool success, bytes memory result) = address(capabilityLibraryLibAddress).delegatecall(
    //     //   abi.encodeWithSelector(HoprCapabilityPermissions.scopeTargetToken.selector, tokenTarget)
    //     // );
    //     // assertTrue(success);
    //     // emit log_named_bytes("result", result);
    //     // assertEq(bytes32(result), HoprCapabilityPermissions.AddressIsZero.selector);

    //     // assertEq(uint256(role.targets.get(targetAddress).getTargetClearance()), uint256(Clearance.FUNCTION), "wrong clearance added");
    //     // emit log_string("Passed: evaluate getTargetClearance");
    //     // assertEq(uint256(role.targets.get(targetAddress).getTargetType()), uint256(TargetType.TOKEN), "wrong target type added");
    // }

    /**
    * @dev Encode an array of funciton signatures (max. 7) into a bytes32. Test with 6
    */
    function test_EncodeAndDecodeSigsAndPermissions(bool startWithZero) public {
        uint256 length = 7;
        // create an array of funciton sigatures
        bytes4[] memory functionSigs = _helperCreateHoprChannelsFunctionSigArray();
        GranularPermission[] memory permissions = _helperCreateHoprChannelsPermissionsArray(length, startWithZero);

        (bytes32 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
        emit log_named_bytes32("encodedValue", encodedValue);
        (bytes4[] memory decodedSigs, GranularPermission[] memory decodedPermissions) = HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedValue, encodedLength);

        assertEq(encodedLength, length, "Encoding length is wrong");
        assertEq(decodedSigs.length, length, "Decoded sigs length is wrong");
        assertEq(decodedPermissions.length, length, "Decoded permissions length is wrong");

        for (uint256 j = 0; j < length; j++) {
            assertEq(decodedSigs[j], functionSigs[j], "Sig changes during the process");
        }
        for (uint256 k = 0; k < length; k++) {
            assertEq(uint256(decodedPermissions[k]), uint256(permissions[k]), "Permission changes during the process");
        }
    }

    /**
     * @dev create a permission array for all the functions default permissions
     */
    function _helperCreateDefaultFunctionPermissionsArray(uint256 length, uint256 startingIndexOffset) private pure returns (CapabilityPermission[] memory permissions) {
        permissions = new CapabilityPermission[](length);
        for (uint256 i = 0; i < length; i++) {
            uint8 permissionIndex = uint8((i + startingIndexOffset) % (uint8(type(CapabilityPermission).max) + 1));
            permissions[i] = CapabilityPermission(permissionIndex);
        }
    }

    /**
     * @dev create a permission array that alternates between GranularPermission.ALLOW and GranularPermission.BLOCK
     */
    function _helperCreateHoprChannelsPermissionsArray(uint256 length, bool startWithZero) private pure returns (GranularPermission[] memory permissions) {
        permissions = new GranularPermission[](length);
        for (uint256 i = 0; i < length; i++) {
            permissions[i] = startWithZero == (i % 2 == 0) ? GranularPermission.ALLOW : GranularPermission.BLOCK;
        }
    }

    /**
     * @dev create an array of funciton sigatures for HoprChannels
     */
    function _helperCreateHoprChannelsFunctionSigArray() private pure returns (bytes4[] memory functionSigs) {
        functionSigs = new bytes4[](7);
        functionSigs[0] = HoprCapabilityPermissions.REDEEM_TICKET_SELECTOR;
        functionSigs[1] = HoprCapabilityPermissions.CLOSE_INCOMING_CHANNEL_SELECTOR;
        functionSigs[2] = HoprCapabilityPermissions.INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR;
        functionSigs[3] = HoprCapabilityPermissions.FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR;
        functionSigs[4] = HoprCapabilityPermissions.FUND_CHANNEL_SELECTOR;
        functionSigs[5] = HoprCapabilityPermissions.SET_COMMITMENT_SELECTOR;
    }
}
