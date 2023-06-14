// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import '../../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import "../../utils/CapabilityLibrary.sol";
import 'forge-std/Test.sol';

contract HoprCapabilityPermissionsTest is Test, CapabilityPermissionsLibFixtureTest {
    Role internal role;
    DefaultPermissions internal defaultPermissions;
    /**
    * Manually import events and errors
    */
    error AddressIsZero();
    event RevokedTarget(address indexed targetAddress);
    event ScopedTarget(address indexed targetAddress, TargetType targetType, DefaultPermissions defaultPermission);
    event ScopedGranularChannelCapability(
        address indexed targetAddress,
        bytes32 indexed channelId,
        bytes4 selector,
        GranularPermission permission
    );
    event ScopedGranularTokenCapability(
        address indexed targetAddress,
        address indexed recipientAddress,
        bytes4 selector,
        GranularPermission permission
    );
    event ScopedGranularSendCapability(
        address indexed recipientAddress,
        GranularPermission permission
    );

    function setUp() public virtual override(CapabilityPermissionsLibFixtureTest) {
        super.setUp();
        defaultPermissions = DefaultPermissions({
            defaultTargetPermission: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultRedeemTicketSafeFunctionPermisson: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultBatchRedeemTicketsSafeFunctionPermisson: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultCloseIncomingChannelSafeFunctionPermisson: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultInitiateOutgoingChannelClosureSafeFunctionPermisson: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultFundChannelMultiFunctionPermisson: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultSetCommitmentSafeFunctionPermisson: Permission.SPECIFIC_FALLBACK_ALLOW,
            defaultApproveFunctionPermisson: Permission.SPECIFIC_FALLBACK_BLOCK,
            defaultSendFunctionPermisson: Permission.SPECIFIC_FALLBACK_BLOCK
        });
    }

    /**
    * @dev Failes to add token target(s) when the account is not address zero
    */
    function testRevert_WhenAddressZeroAddTargetToken() public {
        vm.expectRevert(AddressIsZero.selector);
        HoprCapabilityPermissions.scopeTargetToken(role, address(0), defaultPermissions);
    }

    /**
    * @dev Add token target(s) when the account is not address zero
    */
    function testFuzz_AddTargetToken(address account) public {
        vm.assume(account != address(0));
        vm.expectEmit(true, false, false, false, address(this));
        
        emit ScopedTarget(account, TargetType.TOKEN, defaultPermissions);
        HoprCapabilityPermissions.scopeTargetToken(role, account, defaultPermissions);

        assertEq(uint256(role.targets[account].clearance), uint256(Clearance.FUNCTION), "wrong clearance added");
        assertEq(uint256(role.targets[account].targetType), uint256(TargetType.TOKEN), "wrong target type added");
    }

    /**
    * @dev Encode an array of funciton signatures (max. 7) into a bytes32. Test with 6
    */
    function test_EncodeAndDecodeSigsAndPermissions(bool startWithZero) public {
        uint256 length = 6;
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
        functionSigs[1] = HoprCapabilityPermissions.BATCH_REDEEM_TICKETS_SELECTOR;
        functionSigs[2] = HoprCapabilityPermissions.CLOSE_INCOMING_CHANNEL_SELECTOR;
        functionSigs[3] = HoprCapabilityPermissions.INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR;
        functionSigs[4] = HoprCapabilityPermissions.FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR;
        functionSigs[5] = HoprCapabilityPermissions.FUND_CHANNEL_MULTI_SELECTOR;
        functionSigs[6] = HoprCapabilityPermissions.SET_COMMITMENT_SELECTOR;
    }
}
