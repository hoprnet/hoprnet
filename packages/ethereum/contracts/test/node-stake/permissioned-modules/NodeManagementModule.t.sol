// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import '../../../src/node-stake/permissioned-module/NodeManagementModule.sol';
import '../../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import "../../utils/CapabilityLibrary.sol";
import '../../../script/utils/SafeSuiteLib.sol';
import '../../utils/SafeSingleton.sol';
import 'forge-std/Test.sol';

contract HoprNodeManagementModuleTest is Test, CapabilityPermissionsLibFixtureTest, SafeSingletonFixtureTest {
    HoprNodeManagementModule public moduleSingleton;
    address public multiaddr;
    address public safe;
    FunctionPermission[] internal defaultFunctionPermission;
    /**
    * Manually import events and errors
    */
    event SetMultisendAddress(address multisendAddress);
    event NodeAdded(address indexed node);
    event NodeRemoved(address indexed node);

    function setUp() public virtual override(CapabilityPermissionsLibFixtureTest, SafeSingletonFixtureTest) {
        super.setUp();
        multiaddr = vm.addr(100); // make address(100) multiaddr
        safe = vm.addr(101); // make address(101) a safe

        moduleSingleton = new HoprNodeManagementModule();
        defaultFunctionPermission = new FunctionPermission[](TargetUtils.NUM_DEFAULT_FUNCTION_PERMISSIONS);
        defaultFunctionPermission = [
            FunctionPermission.SPECIFIC_FALLBACK_ALLOW,
            FunctionPermission.SPECIFIC_FALLBACK_ALLOW,
            FunctionPermission.SPECIFIC_FALLBACK_ALLOW,
            FunctionPermission.SPECIFIC_FALLBACK_ALLOW,
            FunctionPermission.SPECIFIC_FALLBACK_ALLOW,
            FunctionPermission.SPECIFIC_FALLBACK_ALLOW,
            FunctionPermission.SPECIFIC_FALLBACK_ALLOW,
            FunctionPermission.SPECIFIC_FALLBACK_BLOCK,
            FunctionPermission.SPECIFIC_FALLBACK_BLOCK
        ];
    }

    /**
    * @dev Failes to add token target(s) when the account is not address zero
    */
    function testRevert_CannotInitializeSingleton() public {
        emit log_named_address("capabilityLibraryLibAddress", capabilityLibraryLibAddress);
        // bytes memory libCreationCode = type(HoprCapabilityPermissions).creationCode;
        // emit log_named_bytes("libCreationCode", libCreationCode);
        // bytes memory libRuntimeCode = type(HoprCapabilityPermissions).runtimeCode;
        // emit log_named_bytes("libRuntimeCode", libRuntimeCode);
        // bytes memory creationCode = type(HoprNodeManagementModule).creationCode;
        // emit log_named_bytes("creationCode", creationCode);

        vm.expectRevert(bytes("Initializable: contract is already initialized"));
        HoprNodeManagementModule(moduleSingleton).initialize(abi.encode(address(1), address(2)));
    }

    function test_AddNode(address account) public {
        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit NodeAdded(account);
        moduleSingleton.addNode(account);

        assertTrue(moduleSingleton.isNode(account));
        vm.stopPrank();
    }

    /**
    * @dev Add token target(s) when the account is not address zero
    */
    function testFuzz_AddTargetTokenFromModule(address tokenAddress) public {
        vm.assume(tokenAddress != address(0));
        address owner = moduleSingleton.owner();
        Target actualTokenTarget = TargetUtils.encodeDefaultPermissions(
            tokenAddress,
            Clearance.FUNCTION,
            TargetType.TOKEN,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit HoprCapabilityPermissions.ScopedTargetToken(tokenAddress, actualTokenTarget);
        moduleSingleton.scopeTargetToken(actualTokenTarget);
    }

    /**
    * @dev Add Channels and Token targets, where channel is vm.addr()
    */
    function test_AddChannelsAndTokenTarget(uint256 targetUint) public {
        address channels = makeAddr("HoprChannels");
        address token = makeAddr("HoprToken");
        address owner = moduleSingleton.owner();

        Target target = Target.wrap(targetUint);
        vm.startPrank(owner);
        vm.mockCall(
            channels,
            abi.encodeWithSignature(
                'token()'
            ),
            abi.encode(token)
        );

        // token target overwritten mask
        // <         160 bits for address         >    <>              <func>
        // ffffffffffffffffffffffffffffffffffffffff0000ff00000000000000ffff
        // channels target overwritten mask
        // <         160 bits for address         >    <   functions  >
        // ffffffffffffffffffffffffffffffffffffffff0000ffffffffffffffff0000
        Target overwrittenTokenTarget = _helperTargetBitwiseAnd(target, hex"ffffffffffffffffffffffffffffffffffffffff0000ff00000000000000ffff");
        overwrittenTokenTarget = _helperTargetBitwiseOr(overwrittenTokenTarget, hex"0000000000000000000000000000000000000000010100000000000000000000");
        Target overwrittenChannelTarget = _helperTargetBitwiseAnd(target, hex"ffffffffffffffffffffffffffffffffffffffff0000ffffffffffffffff0000");
        overwrittenChannelTarget = _helperTargetBitwiseOr(overwrittenChannelTarget, hex"0000000000000000000000000000000000000000010200000000000000000000");
        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit HoprCapabilityPermissions.ScopedTargetChannels(channels, overwrittenChannelTarget);
        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit HoprCapabilityPermissions.ScopedTargetToken(token, overwrittenTokenTarget);
        moduleSingleton.addChannelsAndTokenTarget(channels, target);
    }



    // /**
    // * @dev Encode an array of permission enums into uint256 and vice versa
    // */
    // function testFuzz_EncodeAndDecodePermissionEnums(uint256 length, bool startWithZero) public {
    //     // length must not exceed 256
    //     vm.assume(length <= 256);
    //     // create a permission array that alternates between 0 and 1
    //     uint256[] memory permissions = _helperCreateHoprChannelsPermissionsArray(length, startWithZero);

    //     (uint256 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodePermissionEnums(permissions);
    //     (uint256[] memory decodedPermissions) = HoprCapabilityPermissions.decodePermissionEnums(encodedValue, encodedLength);

    //     assertEq(encodedLength, length, "Encoding length is wrong");
    //     assertEq(decodedPermissions.length, length, "Decoded length is wrong");

    //     for (uint256 j = 0; j < length; j++) {
    //         assertEq(decodedPermissions[j], permissions[j], "Element changes during the process");
    //     }
    // }

    // /**
    // * @dev Encode an array of funciton signatures (max. 7) into a bytes32
    // */
    // function test_EncodeAndDecodeFunctionSigs() public {
    //     // create an array of funciton sigatures
    //     bytes4[] memory functionSigs = _helperCreateHoprChannelsFunctionSigArray();

    //     (bytes32 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodeFunctionSigs(functionSigs);
    //     (bytes4[] memory decoded) = HoprCapabilityPermissions.decodeFunctionSigs(encodedValue, encodedLength);

    //     assertEq(encodedLength, decoded.length, "Length is wrong");

    //     for (uint256 j = 0; j < encodedLength; j++) {
    //         assertEq(decoded[j], functionSigs[j], "Element changes during the process");
    //     }
    // }

    // /**
    // * @dev Encode an array of funciton signatures (max. 7) into a bytes32. Test with 6
    // */
    // function test_EncodeAndDecodeSigsAndPermissions(bool startWithZero) public {
    //     uint256 length = 6;
    //     // create an array of funciton sigatures
    //     bytes4[] memory functionSigs = _helperCreateHoprChannelsFunctionSigArray();
    //     uint256[] memory permissions = _helperCreateHoprChannelsPermissionsArray(length, startWithZero);

    //     (bytes32 encodedValue, uint256 encodedLength) = HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
    //     emit log_named_bytes32("encodedValue", encodedValue);
    //     (bytes4[] memory decodedSigs, uint256[] memory decodedPermissions) = HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedValue, encodedLength);

    //     assertEq(encodedLength, length, "Encoding length is wrong");
    //     assertEq(decodedSigs.length, length, "Decoded sigs length is wrong");
    //     assertEq(decodedPermissions.length, length, "Decoded permissions length is wrong");

    //     for (uint256 j = 0; j < length; j++) {
    //         assertEq(decodedSigs[j], functionSigs[j], "Sig changes during the process");
    //     }
    //     for (uint256 k = 0; k < length; k++) {
    //         assertEq(decodedPermissions[k], permissions[k], "Permission changes during the process");
    //     }
    // }

    // /**
    //  * @dev create a permission array that alternates between 0 and 1
    //  */
    // function _helperCreateHoprChannelsPermissionsArray(uint256 length, bool startWithZero) private returns (uint256[] memory permissions) {
    //     permissions = new uint256[](length);
    //     for (uint256 i = 0; i < length; i++) {
    //         permissions[i] = startWithZero == (i % 2 == 0) ? 0 : 1;
    //     }
    // }

    // /**
    //  * @dev create an array of funciton sigatures for HoprChannels
    //  */
    // function _helperCreateHoprChannelsFunctionSigArray() private returns (bytes4[] memory functionSigs) {
    //     functionSigs = new bytes4[](6);
    //     functionSigs[0] = HoprCapabilityPermissions.FUND_CHANNEL_MULTI_SELECTOR;
    //     functionSigs[1] = HoprCapabilityPermissions.REDEEM_TICKET_SELECTOR;
    //     functionSigs[2] = HoprCapabilityPermissions.REDEEM_TICKETS_SELECTOR;
    //     functionSigs[3] = HoprCapabilityPermissions.INITIATE_CHANNEL_CLOSURE_SELECTOR;
    //     functionSigs[4] = HoprCapabilityPermissions.FINALIZE_CHANNEL_CLOSURE_SELECTOR;
    //     functionSigs[5] = HoprCapabilityPermissions.BUMP_CHANNEL_SELECTOR;
    // }

    function _helperTargetBitwiseAnd(Target target, bytes32 mask) private pure returns (Target) {
        return Target.wrap(uint256(bytes32(Target.unwrap(target)) & mask));
    }
    function _helperTargetBitwiseOr(Target target, bytes32 mask) private pure returns (Target) {
        return Target.wrap(uint256(bytes32(Target.unwrap(target)) | mask));
    }
}
