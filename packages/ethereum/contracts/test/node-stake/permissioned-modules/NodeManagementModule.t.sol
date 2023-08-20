// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "../../../src/node-stake/permissioned-module/NodeManagementModule.sol";
import "../../../src/node-stake/permissioned-module/CapabilityPermissions.sol";
import "../../utils/CapabilityLibrary.sol";
import "../../../src/utils/SafeSuiteLib.sol";
import "../../utils/SafeSingleton.sol";
import "../../../src/interfaces/IAvatar.sol";
import "../../../src/Crypto.sol";
import "forge-std/Test.sol";
import "openzeppelin-contracts-upgradeable/proxy/ClonesUpgradeable.sol";
import { SimplifiedModuleEvents } from "../../../src/node-stake/permissioned-module/SimplifiedModule.sol";

/**
 * @dev This files tests both HoprNodeManagementModule and the CapabilityPermissions.sol
 */
contract HoprNodeManagementModuleTest is
    Test,
    CapabilityPermissionsLibFixtureTest,
    SafeSingletonFixtureTest,
    SimplifiedModuleEvents
{
    using stdStorage for StdStorage;
    using TargetUtils for Target;
    using ClonesUpgradeable for address;

    HoprNodeManagementModule public moduleSingleton;
    HoprNodeManagementModule public moduleProxy;
    address public multiaddr;
    address public safe;
    address public channels;
    address public token;
    CapabilityPermission[] internal defaultFunctionPermission;
    /**
     * Manually import events and errors
     */

    event SetMultisendAddress(address indexed multisendAddress);
    event NodeAdded(address indexed node);
    event NodeRemoved(address indexed node);
    event AvatarSet(address indexed previousAvatar, address indexed newAvatar);
    event Upgraded(address indexed implementation);

    function setUp() public virtual override(CapabilityPermissionsLibFixtureTest, SafeSingletonFixtureTest) {
        super.setUp();
        multiaddr = vm.addr(100); // make address(100) multiaddr
        safe = vm.addr(101); // make address(101) a safe
        channels = makeAddr("HoprChannels");
        token = makeAddr("HoprToken");

        moduleSingleton = new HoprNodeManagementModule();
        moduleProxy = HoprNodeManagementModule(address(moduleSingleton).cloneDeterministic(bytes32(hex"abcd")));
        defaultFunctionPermission = new CapabilityPermission[](TargetUtils.NUM_CAPABILITY_PERMISSIONS);
        defaultFunctionPermission = [
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultRedeemTicketSafeFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultBatchRedeemTicketsSafeFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultCloseIncomingChannelSafeFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultInitiateOutgoingChannelClosureSafeFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultFundChannelMultiFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW, // defaultSetCommitmentSafeFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_BLOCK, // defaultApproveFunctionPermisson
            CapabilityPermission.SPECIFIC_FALLBACK_BLOCK // defaultSendFunctionPermisson
        ];
    }

    /**
     * @dev Failes to add token target(s) when the account is not address zero
     */
    function testRevert_CannotInitializeSingleton() public {
        emit log_named_address("capabilityLibraryLibAddress", capabilityLibraryLibAddress);

        vm.expectRevert(bytes("Initializable: contract is already initialized"));
        HoprNodeManagementModule(moduleSingleton).initialize(abi.encode(address(1), address(2), address(3)));
        vm.clearMockedCalls();
    }
    /**
     * @dev Anyone can initialize a proxy
     */

    function test_CanInitializeProxy() public {
        address _channels = 0x0101010101010101010101010101010101010101;
        address _token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(_channels, abi.encodeWithSignature("token()"), abi.encode(_token));
        emit SetMultisendAddress(multiaddr);
        moduleProxy.initialize(
            abi.encode(
                address(1), multiaddr, bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101")
            )
        );
        assertEq(moduleProxy.owner(), address(1));
        vm.clearMockedCalls();
    }

    // /**
    // * @dev Anyone can initialize a proxy
    // */
    // function test_CanUpgradeImplementation() public {
    //     HoprNodeManagementModule newImplementation = new HoprNodeManagementModule();
    //     address _channels = 0x0101010101010101010101010101010101010101;
    //     address _token = 0x1010101010101010101010101010101010101010;
    //     vm.mockCall(
    //         _channels,
    //         abi.encodeWithSignature(
    //             'token()'
    //         ),
    //         abi.encode(_token)
    //     );
    //     moduleProxy.initialize(abi.encode(address(1), multiaddr,
    // bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101")));

    //     // get implementation address from slot
    //     bytes32 _IMPLEMENTATION_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;
    //     bytes32 currentImplementation = vm.load(address(moduleProxy), _IMPLEMENTATION_SLOT);
    //     assertEq(address(uint160(uint256(currentImplementation))), address(moduleSingleton));
    //     vm.expectEmit(true, false, false, false, address(moduleProxy));

    //     vm.prank(moduleProxy.owner());
    //     vm.expectEmit(true, false, false, false, address(moduleProxy));
    //     emit Upgraded(address(newImplementation));
    //     moduleProxy.upgradeTo(address(newImplementation));
    //     vm.clearMockedCalls();
    // }

    function test_AddNode(address account) public {
        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit NodeAdded(account);
        moduleProxy.addNode(account);

        assertTrue(moduleProxy.isNode(account));
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testRevert_AddNode(address[] memory accounts, uint256 index) public {
        vm.assume(accounts.length > 0);
        index = bound(index, 0, accounts.length - 1);

        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        _helperAddNodes(accounts);

        assertTrue(moduleProxy.isNode(accounts[index]));
        vm.expectRevert(HoprNodeManagementModule.WithMembership.selector);
        moduleProxy.addNode(accounts[index]);
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testFuzz_RemoveNode(address[] memory accounts, uint256 index) public {
        vm.assume(accounts.length > 0);
        index = bound(index, 0, accounts.length - 1);

        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        _helperAddNodes(accounts);

        assertTrue(moduleProxy.isNode(accounts[index]));
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit NodeRemoved(accounts[index]);
        moduleProxy.removeNode(accounts[index]);
        assertFalse(moduleProxy.isNode(accounts[index]));
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testRevert_RemoveNode(address[] memory accounts, address nodeAddress) public {
        address owner = moduleProxy.owner();
        vm.startPrank(owner);

        _helperAddNodes(accounts);
        vm.assume(!moduleProxy.isNode(nodeAddress));

        vm.expectRevert(HoprCapabilityPermissions.NoMembership.selector);
        moduleProxy.removeNode(nodeAddress);
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testFuzz_SetMultisend(address multisendAddr) public {
        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit SetMultisendAddress(multisendAddr);
        moduleProxy.setMultisend(multisendAddr);
        vm.stopPrank();
        assertEq(moduleProxy.multisend(), multisendAddr);
        vm.clearMockedCalls();
    }

    /**
     * @dev Add channels target(s) when the account is not address zero
     */
    function testFuzz_ScopeTargetChannelsFromModule(address channelsAddress) public {
        vm.assume(channelsAddress != address(0));
        address owner = moduleProxy.owner();
        Target channelsTarget = TargetUtils.encodeDefaultPermissions(
            channelsAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.ScopedTargetChannels(channelsAddress, channelsTarget);
        moduleProxy.scopeTargetChannels(channelsTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail to channels target(s) when the account is address zero
     */
    function testRevert_ScopeZeroAddressTargetChannelsFromModule() public {
        address channelsAddress = address(0);
        address owner = moduleProxy.owner();
        Target channelsTarget = TargetUtils.encodeDefaultPermissions(
            channelsAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        moduleProxy.scopeTargetChannels(channelsTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail to add channels target(s) when the account has been scopec
     */
    function testRevert_ScopeExistingTargetChannelsFromModule(
        address[] memory channelsAddresses,
        uint256 randomIndex
    )
        public
    {
        vm.assume(channelsAddresses.length > 0);

        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) =
            _helperGetUniqueAddressArrayAndRandomItem(channelsAddresses, randomIndex);
        vm.assume(results.length > 0);

        for (uint256 i = 0; i < results.length; i++) {
            vm.assume(results[i] != address(0));
            Target channelsTarget = TargetUtils.encodeDefaultPermissions(
                results[i],
                Clearance.FUNCTION,
                TargetType.CHANNELS,
                TargetPermission.ALLOW_ALL,
                defaultFunctionPermission
            );
            moduleProxy.scopeTargetChannels(channelsTarget);
        }

        Target existingChannelsTarget = TargetUtils.encodeDefaultPermissions(
            oneAddress, Clearance.FUNCTION, TargetType.CHANNELS, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );
        vm.expectRevert(HoprCapabilityPermissions.TargetIsScoped.selector);
        moduleProxy.scopeTargetChannels(existingChannelsTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev Add token target(s) when the account is not address zero
     */
    function testFuzz_ScopeTargetTokenFromModule(address tokenAddress) public {
        vm.assume(tokenAddress != address(0));
        address owner = moduleProxy.owner();
        Target actualTokenTarget = TargetUtils.encodeDefaultPermissions(
            tokenAddress, Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.ScopedTargetToken(tokenAddress, actualTokenTarget);
        moduleProxy.scopeTargetToken(actualTokenTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail to token target(s) when the account is address zero
     */
    function testRevert_ScopeZeroAddressTargetTokenFromModule() public {
        address tokenAddress = address(0);
        address owner = moduleProxy.owner();
        Target tokenTarget = TargetUtils.encodeDefaultPermissions(
            tokenAddress, Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        moduleProxy.scopeTargetToken(tokenTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail to add token target(s) when the account has been scopec
     */
    function testRevert_ScopeExistingTargetTokenFromModule(
        address[] memory tokenAddresses,
        uint256 randomIndex
    )
        public
    {
        vm.assume(tokenAddresses.length > 0);

        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) =
            _helperGetUniqueAddressArrayAndRandomItem(tokenAddresses, randomIndex);
        vm.assume(results.length > 0);

        for (uint256 i = 0; i < results.length; i++) {
            vm.assume(results[i] != address(0));
            Target tokenTarget = TargetUtils.encodeDefaultPermissions(
                results[i], Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.ALLOW_ALL, defaultFunctionPermission
            );

            moduleProxy.scopeTargetToken(tokenTarget);
        }

        Target existingTokenTarget = TargetUtils.encodeDefaultPermissions(
            oneAddress, Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        vm.expectRevert(HoprCapabilityPermissions.TargetIsScoped.selector);
        moduleProxy.scopeTargetToken(existingTokenTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev Add send target(s) when the account is a member
     */
    function testFuzz_ScopeTargetSendFromModule(address[] memory accounts, uint256 index) public {
        vm.assume(accounts.length > 0);
        index = bound(index, 0, accounts.length - 1);

        address owner = moduleProxy.owner();
        vm.startPrank(owner);

        // add nodes and take one from the added node
        _helperAddNodes(accounts);

        vm.assume(accounts[index] != address(0));

        assertTrue(moduleProxy.isNode(accounts[index]));

        Target sendTarget = TargetUtils.encodeDefaultPermissions(
            accounts[index], Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.ScopedTargetSend(accounts[index], sendTarget);
        moduleProxy.scopeTargetSend(sendTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev Add send target(s) when the account is a member
     */
    function testFuzz_IncludeANode(address node) public {
        vm.assume(node != address(0));
        address owner = moduleProxy.owner();

        // add nodes and take one from the added node
        Target sendTarget = TargetUtils.encodeDefaultPermissions(
            node, Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        CapabilityPermission[] memory updatedPermission =
            new CapabilityPermission[](TargetUtils.NUM_CAPABILITY_PERMISSIONS);
        for (uint256 j = 0; j < updatedPermission.length; j++) {
            updatedPermission[j] = CapabilityPermission.NONE;
        }
        // add nodes and take one from the added node
        Target upatedTarget = TargetUtils.encodeDefaultPermissions(
            node, Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, updatedPermission
        );

        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit NodeAdded(node);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.ScopedTargetSend(node, upatedTarget);
        vm.expectEmit(true, true, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.ScopedGranularSendCapability(node, node, GranularPermission.ALLOW);
        moduleProxy.includeNode(sendTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail to scope send target(s) when the account is address zero
     */
    function testRevert_ScopeNonMemberTargetSendFromModule() public {
        address tokenAddress = address(0);
        address owner = moduleProxy.owner();
        Target sendTarget = TargetUtils.encodeDefaultPermissions(
            tokenAddress, Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        vm.prank(owner);
        vm.expectRevert(HoprCapabilityPermissions.NoMembership.selector);
        moduleProxy.scopeTargetSend(sendTarget);
        vm.clearMockedCalls();
    }
    /**
     * @dev fail to scope send target(s) when the account is address zero
     */

    function testRevert_ScopeZeroAddressTargetSendFromModule() public {
        address sendAddress = address(0);
        address owner = moduleProxy.owner();
        Target sendTarget = TargetUtils.encodeDefaultPermissions(
            sendAddress, Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        vm.startPrank(owner);
        moduleProxy.addNode(sendAddress);
        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        moduleProxy.scopeTargetSend(sendTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail to add send target(s) when the account has been scopec
     */
    function testRevert_ScopeExistingTargetSendFromModule(address[] memory sendAddresses, uint256 randomIndex) public {
        vm.assume(sendAddresses.length > 0);

        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) =
            _helperGetUniqueAddressArrayAndRandomItem(sendAddresses, randomIndex);
        vm.assume(results.length > 0);
        // add nodes and take one from the added node
        _helperAddNodes(results);

        for (uint256 i = 0; i < results.length; i++) {
            vm.assume(results[i] != address(0));
            assertTrue(moduleProxy.isNode(results[i]));
            Target sendTarget = TargetUtils.encodeDefaultPermissions(
                results[i], Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, defaultFunctionPermission
            );

            moduleProxy.scopeTargetSend(sendTarget);
        }

        Target existingSendTarget = TargetUtils.encodeDefaultPermissions(
            oneAddress, Clearance.FUNCTION, TargetType.SEND, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        );

        vm.expectRevert(HoprCapabilityPermissions.TargetIsScoped.selector);
        moduleProxy.scopeTargetSend(existingSendTarget);
        vm.clearMockedCalls();
    }

    /**
     * @dev Add Channels and Token targets, where channel is vm.addr()
     */
    function test_AddChannelsAndTokenTarget(uint256 targetUint) public {
        address owner = moduleProxy.owner();
        Target target = Target.wrap(targetUint);
        address channelAddress = target.getTargetAddress();
        address tokenAddress = vm.addr(201);
        vm.assume(channelAddress != address(0));
        vm.assume(channelAddress != tokenAddress);

        vm.startPrank(owner);
        vm.mockCall(channelAddress, abi.encodeWithSignature("token()"), abi.encode(tokenAddress));

        // token target overwritten mask
        // <         160 bits for address         >    <>              <func>
        // ffffffffffffffffffffffffffffffffffffffff0000ff00000000000000ffff
        // channels target overwritten mask
        // <         160 bits for address         >    <   functions  >
        // ffffffffffffffffffffffffffffffffffffffff0000ffffffffffffffff0000
        Target overwrittenTokenTarget =
            _helperTargetBitwiseAnd(target, hex"ffffffffffffffffffffffffffffffffffffffff0000ff00000000000000ffff");
        overwrittenTokenTarget = _helperTargetBitwiseOr(
            overwrittenTokenTarget, hex"0000000000000000000000000000000000000000010100000000000000000000"
        );
        Target overwrittenChannelTarget =
            _helperTargetBitwiseAnd(target, hex"ffffffffffffffffffffffffffffffffffffffff0000ffffffffffffffff0000");
        overwrittenChannelTarget = _helperTargetBitwiseOr(
            overwrittenChannelTarget, hex"0000000000000000000000000000000000000000010200000000000000000000"
        );
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.ScopedTargetChannels(channelAddress, overwrittenChannelTarget);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.ScopedTargetToken(tokenAddress, overwrittenTokenTarget);
        moduleProxy.addChannelsAndTokenTarget(target);
        vm.clearMockedCalls();
    }

    // test revokeTarget
    /**
     * @dev owner revoke a target
     */
    function testFuzz_RevokeTargetFromModule(address[] memory accounts, uint256 randomIndex) public {
        // scope some targets
        vm.assume(accounts.length > 0);
        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) =
            _helperGetUniqueAddressArrayAndRandomItem(accounts, randomIndex);
        for (uint256 i = 0; i < results.length; i++) {
            vm.assume(results[i] != address(0));
            Target tokenTarget = TargetUtils.encodeDefaultPermissions(
                results[i], Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.ALLOW_ALL, defaultFunctionPermission
            );

            moduleProxy.scopeTargetToken(tokenTarget);
        }

        // remove target
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit HoprCapabilityPermissions.RevokedTarget(oneAddress);
        moduleProxy.revokeTarget(oneAddress);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail to remove a target that is not scoped
     */
    function testRevert_RevokeNonScopedTargetFromModule(address scopeTargetAddr) public {
        address owner = moduleProxy.owner();

        vm.startPrank(owner);
        vm.expectRevert(HoprCapabilityPermissions.TargetIsNotScoped.selector);
        moduleProxy.revokeTarget(scopeTargetAddr);
        vm.clearMockedCalls();
    }

    /**
     * @dev scope channels for (source, destination)
     */
    function testFuzz_ScopeChannelsCapabilities(bytes4[] memory _randomFunctionSigs, uint256 _size) public {
        // create some capabilities
        _size = bound(_size, 0, 7);
        _size = _randomFunctionSigs.length < _size ? _randomFunctionSigs.length : _size;
        bytes4[] memory functionSigs = new bytes4[](_size);
        GranularPermission[] memory permissions = new GranularPermission[](_size);
        for (uint256 i = 0; i < _size; i++) {
            functionSigs[i] = _randomFunctionSigs[i];
            permissions[i] = GranularPermission(
                uint8(uint256(bytes32(_randomFunctionSigs[i])) % (uint256(type(GranularPermission).max) + 1))
            );
        }
        (bytes32 encoded, uint256 length) =
            HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
        assertEq(length, _size);
        // scope capabilities
        vm.prank(moduleProxy.owner());
        for (uint256 j = 0; j < length; j++) {
            if (functionSigs[j] != bytes4(hex"00")) {
                vm.expectEmit(true, true, false, false, address(moduleProxy));
                emit HoprCapabilityPermissions.ScopedGranularChannelCapability(
                    vm.addr(200), bytes32(hex"0200"), functionSigs[j], permissions[j]
                );
            }
        }
        moduleProxy.scopeChannelsCapabilities(
            vm.addr(200), // mocked targetAddress
            bytes32(hex"0200"), // mocked channelId
            encoded // encodeSigsPermissions
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev encode function permissions but revert due to ArrayTooLong
     */
    function testRevert_EncodeFunctionSigsAndPermissionsLengths() public {
        uint256 size = 8;
        bytes4[] memory functionSigs = new bytes4[](size);
        GranularPermission[] memory permissions = new GranularPermission[](size);

        for (uint256 i = 0; i < functionSigs.length; i++) {
            functionSigs[i] = bytes4(bytes32(i));
            permissions[i] = GranularPermission(
                uint8(uint256(bytes32(functionSigs[i])) % (uint256(type(GranularPermission).max) + 1))
            );
        }
        vm.expectRevert(HoprCapabilityPermissions.ArrayTooLong.selector);
        moduleProxy.encodeFunctionSigsAndPermissions(functionSigs, permissions);
    }
    /**
     * @dev encode function permissions but revert due to ArraysDifferentLength
     */

    function testRevert_EncodeFunctionSigsAndPermissionsMismatchedLengths() public {
        uint256 size = 6;
        bytes4[] memory functionSigs = new bytes4[](size);
        GranularPermission[] memory permissions = new GranularPermission[](size + 1);

        for (uint256 i = 0; i < functionSigs.length; i++) {
            functionSigs[i] = bytes4(bytes32(i));
            permissions[i] = GranularPermission(
                uint8(uint256(bytes32(functionSigs[i])) % (uint256(type(GranularPermission).max) + 1))
            );
        }
        permissions[6] = GranularPermission(0);
        vm.expectRevert(HoprCapabilityPermissions.ArraysDifferentLength.selector);
        moduleProxy.encodeFunctionSigsAndPermissions(functionSigs, permissions);
    }
    /**
     * @dev encode function permissions but revert due to ArraysDifferentLength
     */

    function test_EncodeFunctionSigsAndPermissionsMismatchedLengths() public {
        uint256 size = 6;
        bytes4[] memory functionSigs = new bytes4[](size);
        GranularPermission[] memory permissions = new GranularPermission[](size);

        for (uint256 i = 0; i < functionSigs.length; i++) {
            functionSigs[i] = bytes4(bytes32(i));
            permissions[i] = GranularPermission(
                uint8(uint256(bytes32(functionSigs[i])) % (uint256(type(GranularPermission).max) + 1))
            );
        }
        (bytes32 encoded, uint256 length) = moduleProxy.encodeFunctionSigsAndPermissions(functionSigs, permissions);
        (bytes4[] memory _functionSigs, GranularPermission[] memory _permissions) =
            moduleProxy.decodeFunctionSigsAndPermissions(encoded, length);
        assertEq(_functionSigs.length, size);
        assertEq(_permissions.length, size);
    }
    /**
     * @dev deecode function permissions but revert due to ArrayTooLong
     */

    function testRevert_DecodeFunctionSigsAndPermissionsLengths() public {
        vm.expectRevert(HoprCapabilityPermissions.ArrayTooLong.selector);
        moduleProxy.decodeFunctionSigsAndPermissions(bytes32(hex"1234567890"), 8);
    }

    /**
     * @dev scope tokens for (source, destination)
     */
    function testFuzz_ScopeTokenCapabilities(bytes4[] memory _randomFunctionSigs, uint256 _size) public {
        // create some capabilities
        _size = bound(_size, 0, 7);
        _size = _randomFunctionSigs.length < _size ? _randomFunctionSigs.length : _size;
        bytes4[] memory functionSigs = new bytes4[](_size);
        GranularPermission[] memory permissions = new GranularPermission[](_size);
        for (uint256 i = 0; i < _size; i++) {
            functionSigs[i] = _randomFunctionSigs[i];
            permissions[i] = GranularPermission(
                uint8(uint256(bytes32(_randomFunctionSigs[i])) % (uint256(type(GranularPermission).max) + 1))
            );
        }
        (bytes32 encoded, uint256 length) =
            HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
        assertEq(length, _size);
        // scope capabilities
        vm.prank(moduleProxy.owner());
        for (uint256 j = 0; j < length; j++) {
            if (functionSigs[j] != bytes4(hex"00") && j < 2) {
                vm.expectEmit(true, true, false, false, address(moduleProxy));
                emit HoprCapabilityPermissions.ScopedGranularTokenCapability(
                    vm.addr(200), vm.addr(201), vm.addr(202), functionSigs[j], permissions[j]
                );
            }
        }
        moduleProxy.scopeTokenCapabilities(
            vm.addr(200), // mocked nodeAddress
            vm.addr(201), // mocked targetAddress
            vm.addr(202), // mocked beneficiary
            encoded // encodeSigsPermissions
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev scope tokens for (source, destination)
     */
    function testFuzz_ScopeSendCapability(bytes4 functionSig) public {
        GranularPermission permission =
            GranularPermission(uint8(uint256(bytes32(functionSig)) % (uint256(type(GranularPermission).max) + 1)));

        // scope capabilities
        vm.prank(moduleProxy.owner());
        if (functionSig != bytes4(hex"00")) {
            vm.expectEmit(true, true, false, false, address(moduleProxy));
            emit HoprCapabilityPermissions.ScopedGranularSendCapability(vm.addr(200), vm.addr(201), permission);
        }
        moduleProxy.scopeSendCapability(
            vm.addr(200), // mocked nodeAddress
            vm.addr(201), // mocked beneficiary
            permission // encodeSigsPermissions
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev call transaction execution from a non-member account
     */
    function testRevert_CallFromNonMember(address caller) public {
        vm.assume(caller != address(0));
        vm.assume(caller != vm.addr(301));
        address owner = moduleProxy.owner();

        vm.prank(owner);
        // cannot call from
        assertFalse(moduleProxy.isNode(caller));
        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.NoMembership.selector);
        moduleProxy.execTransactionFromModule(token, 0, hex"12345678", Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev call transaction execution from a non-member account
     */
    function testRevert_CallWithInvalidData() public {
        address owner = moduleProxy.owner();
        address caller = vm.addr(301);

        vm.prank(owner);
        // include some node as member
        moduleProxy.addNode(caller);
        // cannot call from
        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.FunctionSignatureTooShort.selector);
        moduleProxy.execTransactionFromModule(token, 0, hex"00", Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev call transaction execution to a target that is not scoped
     */
    function testRevert_TargetAddressNotAllowed(address caller) public {
        vm.assume(caller != address(0));
        address owner = moduleProxy.owner();

        vm.startPrank(owner);
        // include some node as member
        moduleProxy.addNode(caller);
        // target exist but not the target address of the calling function
        Target target = TargetUtils.encodeDefaultPermissions(
            token, Clearance.NONE, TargetType.TOKEN, TargetPermission.BLOCK_ALL, defaultFunctionPermission
        );
        moduleProxy.scopeTargetToken(target);
        vm.stopPrank();

        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.TargetAddressNotAllowed.selector);
        moduleProxy.execTransactionFromModule(token, 0, hex"12345678", Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev call transaction to send native token but target is not scoped as SEND
     */
    function testRevert_SendNotAllowed(address caller) public {
        vm.assume(caller != address(0));
        address owner = moduleProxy.owner();

        vm.startPrank(owner);
        // include some node as member
        moduleProxy.addNode(caller);
        // target exist but not the target address of the calling function
        Target target = TargetUtils.encodeDefaultPermissions(
            caller, Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.BLOCK_ALL, defaultFunctionPermission
        );
        moduleProxy.scopeTargetToken(target);
        vm.stopPrank();

        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.SendNotAllowed.selector);
        moduleProxy.execTransactionFromModule(caller, 1, hex"", Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev call transaction to send native token to a scoped target but with data
     */
    function testRevert_SendWithDataNotAllowed(address caller) public {
        vm.assume(caller != address(0));
        address owner = moduleProxy.owner();

        vm.startPrank(owner);
        // include some node as member
        moduleProxy.addNode(caller);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // target exist but not the target address of the calling function
        Target target = TargetUtils.encodeDefaultPermissions(
            caller,
            Clearance.FUNCTION,
            TargetType.SEND,
            TargetPermission.SPECIFIC_FALLBACK_ALLOW,
            channelsTokenPermission
        );
        moduleProxy.scopeTargetSend(target);
        vm.stopPrank();

        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.ParameterNotAllowed.selector);
        moduleProxy.execTransactionFromModule(caller, 1, hex"12345678", Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev call transaction but delegate call is not allowed
     */
    function testRevert_DelegateCallNotAllowed(address caller) public {
        vm.assume(caller != address(0));
        address owner = moduleProxy.owner();

        vm.startPrank(owner);
        // include some node as member
        moduleProxy.addNode(caller);
        // target exist but not the target address of the calling function
        Target target = TargetUtils.encodeDefaultPermissions(
            token, Clearance.FUNCTION, TargetType.TOKEN, TargetPermission.BLOCK_ALL, defaultFunctionPermission
        );
        moduleProxy.scopeTargetToken(target);
        vm.stopPrank();

        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.DelegateCallNotAllowed.selector);
        moduleProxy.execTransactionFromModule(token, 0, hex"12345678", Enum.Operation.DelegateCall);
        vm.clearMockedCalls();
    }

    /**
     * @dev revert due to default permission not allowed
     */
    function testRevert_ExecTransactionButDefaultPermissionRejects() public {
        // scope channels and token contract
        address msgSender = vm.addr(1);

        Target target = TargetUtils.encodeDefaultPermissions(
            channels, Clearance.FUNCTION, TargetType.CHANNELS, TargetPermission.BLOCK_ALL, defaultFunctionPermission
        ); // clerance: FUNCTION default ALLOW_ALL
        stdstore.target(address(moduleProxy)).sig("owner()").checked_write(safe);
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));
        address owner = moduleProxy.owner();
        vm.startPrank(owner);

        // add token and channels as accept_all target
        moduleProxy.addChannelsAndTokenTarget(target);
        // include caller as node
        moduleProxy.addNode(msgSender);

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100);
        vm.stopPrank();

        // execute function
        vm.prank(msgSender);
        vm.expectRevert(HoprCapabilityPermissions.DefaultPermissionRejected.selector);
        moduleProxy.execTransactionFromModule(token, 0, data, Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute a transaction from the module to a scoped target
     */
    function test_ExecTransactionFromModuleToAScopedTarget() public {
        // scope channels and token contract
        address msgSender = vm.addr(1);

        Target target = TargetUtils.encodeDefaultPermissions(
            channels, Clearance.FUNCTION, TargetType.CHANNELS, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        ); // clerance: FUNCTION default ALLOW_ALL
        stdstore.target(address(moduleProxy)).sig("owner()").checked_write(safe);
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));
        address owner = moduleProxy.owner();
        vm.startPrank(owner);

        // add token and channels as accept_all target
        moduleProxy.addChannelsAndTokenTarget(target);
        // include caller as node
        moduleProxy.addNode(msgSender);

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100);
        vm.stopPrank();

        // execute function
        vm.prank(msgSender);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit SimplifiedModuleEvents.ExecutionSuccess();
        bool result = moduleProxy.execTransactionFromModule(token, 0, data, Enum.Operation.Call);
        assertTrue(result);
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute a transaction from the module to a scoped target
     */
    function test_ExecTransactionFromModuleButGranularPermissionReject() public {
        // scope channels and token contract
        address msgSender = vm.addr(1);

        Target target = TargetUtils.encodeDefaultPermissions(
            channels,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.SPECIFIC_FALLBACK_ALLOW,
            defaultFunctionPermission
        ); // clerance: FUNCTION default ALLOW_ALL
        stdstore.target(address(moduleProxy)).sig("owner()").checked_write(safe);
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));

        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        // add token and channels as accept_all target
        moduleProxy.addChannelsAndTokenTarget(target);
        // include caller as node
        moduleProxy.addNode(msgSender);

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100);
        vm.stopPrank();

        // execute function
        vm.prank(msgSender);
        vm.expectRevert(HoprCapabilityPermissions.GranularPermissionRejected.selector);
        moduleProxy.execTransactionFromModule(token, 0, data, Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute a transaction from the module to a scoped target
     */
    function test_ExecTransactionFromModuleReturnData() public {
        // scope channels and token contract
        address msgSender = vm.addr(1);

        Target target = TargetUtils.encodeDefaultPermissions(
            channels, Clearance.FUNCTION, TargetType.CHANNELS, TargetPermission.ALLOW_ALL, defaultFunctionPermission
        ); // clerance: FUNCTION default ALLOW_ALL
        stdstore.target(address(moduleProxy)).sig("owner()").checked_write(safe);
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));
        vm.mockCall(
            safe,
            abi.encodeWithSelector(IAvatar.execTransactionFromModuleReturnData.selector),
            abi.encode(true, hex"12345678")
        );
        address owner = moduleProxy.owner();
        vm.startPrank(owner);
        // add token and channels as accept_all target
        moduleProxy.addChannelsAndTokenTarget(target);
        // include caller as node
        moduleProxy.addNode(msgSender);

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100);
        vm.stopPrank();

        // execute function
        vm.prank(msgSender);
        (bool success, bytes memory result) =
            moduleProxy.execTransactionFromModuleReturnData(token, 0, data, Enum.Operation.Call);
        assertTrue(success);
        assertEq(result, hex"12345678");
        vm.clearMockedCalls();
    }

    /**
     * @dev Fail to call to multisend address
     */
    function testRevert_InvalidMultiSendData() public {
        // set the multisend address to be multiaddr
        stdstore.target(address(moduleProxy)).sig("multisend()").checked_write(multiaddr);

        address owner = moduleProxy.owner();
        address caller = vm.addr(301);

        vm.prank(owner);
        // include some node as member
        moduleProxy.addNode(caller);
        // cannot call from
        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.UnacceptableMultiSendOffset.selector);
        moduleProxy.execTransactionFromModule(
            multiaddr, 0, abi.encodePacked(bytes32(hex"12"), bytes32(hex"34")), Enum.Operation.DelegateCall
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute transactions to a scoped target via multisend
     */
    function test_ExecuteMultiSendTransaction() public {
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        channelsTokenPermission[8] = CapabilityPermission.SPECIFIC_FALLBACK_BLOCK;
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, channelsTokenPermission);

        // prepare a simple token approve and a native token transfer
        bytes[] memory data = new bytes[](2);
        data[0] = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100); // approve on token
        uint8[] memory txOperations = new uint8[](2);
        txOperations[0] = 0;
        txOperations[1] = 0;
        address[] memory txTos = new address[](2);
        txTos[0] = token;
        txTos[1] = msgSender;
        uint256[] memory txValues = new uint256[](2);
        txValues[0] = 0;
        txValues[1] = 1 ether;
        uint256[] memory dataLengths = new uint256[](2);
        dataLengths[0] = data[0].length;
        dataLengths[1] = 0;

        bytes memory safeTxData = _helperBuildMultiSendTx(txOperations, txTos, txValues, dataLengths, data);

        // execute function
        vm.prank(msgSender);
        bool result = moduleProxy.execTransactionFromModule(multiaddr, 0, safeTxData, Enum.Operation.DelegateCall);
        assertTrue(result);
        vm.clearMockedCalls();
    }

    /**
     * @dev fail when node address is not provided correctly
     */
    function testRevert_ExecTransactionFromModuleButGranularPermissionRejectNodePermissionRejected() public {
        // scope channels and token contract
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, channelsTokenPermission);
        // make execTransactionFromModule go through
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));

        // add another node
        address owner = moduleProxy.owner();
        vm.prank(owner);
        address anotherNode = vm.addr(201);
        moduleProxy.addNode(anotherNode);

        // prepare a simple token approve
        bytes memory data =
            abi.encodeWithSelector(HoprChannels.closeIncomingChannelSafe.selector, anotherNode, vm.addr(404));

        // execute function
        vm.prank(msgSender);
        vm.expectRevert(HoprCapabilityPermissions.NodePermissionRejected.selector);
        moduleProxy.execTransactionFromModule(channels, 0, data, Enum.Operation.Call);
        vm.clearMockedCalls();
    }
    /**
     * @dev fail when calling the wrong target (channels)
     */

    function testRevert_ExecTransactionFromModuleButGranularPermissionRejectParameterNotAllowedForChannels() public {
        // scope channels and token contract
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, channelsTokenPermission);
        // make execTransactionFromModule go through
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));

        // add another node
        address owner = moduleProxy.owner();
        vm.prank(owner);
        address anotherNode = vm.addr(201);
        moduleProxy.addNode(anotherNode);

        // prepare a simple token approve
        bytes memory data =
            abi.encodeWithSelector(HoprChannels.closeIncomingChannelSafe.selector, anotherNode, vm.addr(404));

        // execute function
        vm.prank(msgSender);
        vm.expectRevert(HoprCapabilityPermissions.ParameterNotAllowed.selector);
        moduleProxy.execTransactionFromModule(token, 0, data, Enum.Operation.Call);
        vm.clearMockedCalls();
    }
    /**
     * @dev fail when calling the wrong target (token)
     */

    function testRevert_ExecTransactionFromModuleButGranularPermissionRejectParameterNotAllowedForToken() public {
        // scope channels and token contract
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, channelsTokenPermission);
        // make execTransactionFromModule go through
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));

        // add another node
        address owner = moduleProxy.owner();
        vm.prank(owner);
        address anotherNode = vm.addr(201);
        moduleProxy.addNode(anotherNode);

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(IERC20.approve.selector, msgSender, 100);

        // execute function
        vm.prank(msgSender);
        vm.expectRevert(HoprCapabilityPermissions.ParameterNotAllowed.selector);
        moduleProxy.execTransactionFromModule(channels, 0, data, Enum.Operation.Call);
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute transactions to a scoped target via multisend
     */
    function test_ExecuteChannelTransactions() public {
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, channelsTokenPermission);
        // make execTransactionFromModule go through
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));

        // try all functions on tokens
        uint256 size = 7;
        bytes[] memory data = new bytes[](size);
        uint8[] memory txOperations = new uint8[](size);
        address[] memory txTos = new address[](size);
        uint256[] memory txValues = new uint256[](size);
        uint256[] memory dataLengths = new uint256[](size);

        HoprChannels.TicketData memory dummyTicketData = HoprChannels.TicketData(
            bytes32(hex"11"),
            HoprChannels.Balance.wrap(1),
            HoprChannels.TicketIndex.wrap(1),
            HoprChannels.TicketIndexOffset.wrap(1),
            HoprChannels.ChannelEpoch.wrap(1),
            HoprChannels.WinProb.wrap(1)
        );
        HoprChannels.CompactSignature memory dummyCompactSignature =
            HoprCrypto.CompactSignature(bytes32(hex"22"), bytes32(hex"33"));
        HoprChannels.RedeemableTicket memory dummyRedeemableTicket =
            HoprChannels.RedeemableTicket(dummyTicketData, dummyCompactSignature, uint256(bytes32(hex"44")));

        data[0] = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100);
        data[1] = abi.encodeWithSignature("send(address,uint256,bytes)", vm.addr(200), vm.addr(201), hex"ff");
        data[2] = abi.encodeWithSelector(HoprChannels.redeemTicketSafe.selector, msgSender, dummyRedeemableTicket);
        data[3] = abi.encodeWithSelector(HoprChannels.closeIncomingChannelSafe.selector, msgSender, vm.addr(404));
        data[4] =
            abi.encodeWithSelector(HoprChannels.initiateOutgoingChannelClosureSafe.selector, msgSender, vm.addr(404));
        data[5] =
            abi.encodeWithSelector(HoprChannels.finalizeOutgoingChannelClosureSafe.selector, msgSender, vm.addr(404));
        data[6] = abi.encodeWithSelector(
            HoprChannels.fundChannelSafe.selector, msgSender, vm.addr(404), HoprChannels.Balance.wrap(66)
        );
        // data[7] nothing
        // data[8] nothing

        emit log_named_bytes("data[2]", data[2]);

        for (uint256 i = 0; i < size; i++) {
            txTos[i] = channels;
            txOperations[i] = 0;
            txValues[i] = 0;
            dataLengths[i] = data[i].length;
        }
        txTos[0] = token;
        txTos[1] = token;
        // txTos[8] = msgSender;
        // txValues[8] = 1 ether;

        bytes memory safeTxData = _helperBuildMultiSendTx(txOperations, txTos, txValues, dataLengths, data);

        // execute function
        vm.prank(msgSender);
        bool result = moduleProxy.execTransactionFromModule(multiaddr, 0, safeTxData, Enum.Operation.DelegateCall);
        assertTrue(result);
        vm.clearMockedCalls();
    }
    /**
     * @dev fail to retrieve data at a specific index (Bytes32)
     */

    function testRevert_CalldataOutOfBoundsWhenPluckingOneBytes32() public {
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, channelsTokenPermission);
        // make execTransactionFromModule go through
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(HoprChannels.redeemTicketSafe.selector, msgSender);

        // execute function
        vm.prank(msgSender);
        vm.expectRevert(HoprCapabilityPermissions.CalldataOutOfBounds.selector);
        bool result = moduleProxy.execTransactionFromModule(channels, 0, data, Enum.Operation.Call);
        // must revert
        assertFalse(result);
        vm.clearMockedCalls();
    }
    /**
     * @dev fail to retrieve data at a specific index (address)
     */

    function testRevert_CalldataOutOfBoundsWhenPluckingOneStaticAddress() public {
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, channelsTokenPermission);
        // make execTransactionFromModule go through
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(HoprChannels.redeemTicketSafe.selector);

        // execute function
        vm.prank(msgSender);
        vm.expectRevert(HoprCapabilityPermissions.CalldataOutOfBounds.selector);
        bool result = moduleProxy.execTransactionFromModule(channels, 0, data, Enum.Operation.Call);
        // must revert
        assertFalse(result);
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute transactions to a scoped target via multisend
     */
    function test_ExecuteMultiSendTransactionOfTwo() public {
        address msgSender = vm.addr(1);
        CapabilityPermission[] memory channelsTokenPermission = new CapabilityPermission[](9);
        for (uint256 i = 0; i < channelsTokenPermission.length; i++) {
            channelsTokenPermission[i] = CapabilityPermission.SPECIFIC_FALLBACK_ALLOW;
        }
        // scope channels and token contract
        _helperAddTokenAndChannelTarget(msgSender, channelsTokenPermission, defaultFunctionPermission);

        // prepare a simple token approve and a native token transfer
        bytes[] memory data = new bytes[](2);
        data[0] = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100); // approve on token
        uint8[] memory txOperations = new uint8[](2);
        txOperations[0] = 0;
        txOperations[1] = 0;
        address[] memory txTos = new address[](2);
        txTos[0] = token;
        txTos[1] = msgSender;
        uint256[] memory txValues = new uint256[](2);
        txValues[0] = 0;
        txValues[1] = 1 ether;
        uint256[] memory dataLengths = new uint256[](2);
        dataLengths[0] = data[0].length;
        dataLengths[1] = 0;

        bytes memory safeTxData = _helperBuildMultiSendTx(txOperations, txTos, txValues, dataLengths, data);

        // execute function
        vm.prank(msgSender);
        bool result = moduleProxy.execTransactionFromModule(multiaddr, 0, safeTxData, Enum.Operation.DelegateCall);
        assertTrue(result);
        vm.clearMockedCalls();
    }

    // ===================== helper functions =====================

    function _helperTargetBitwiseAnd(Target target, bytes32 mask) private pure returns (Target) {
        return Target.wrap(uint256(bytes32(Target.unwrap(target)) & mask));
    }

    function _helperTargetBitwiseOr(Target target, bytes32 mask) private pure returns (Target) {
        return Target.wrap(uint256(bytes32(Target.unwrap(target)) | mask));
    }

    function _helperAddNodes(address[] memory accounts) private {
        for (uint256 i = 0; i < accounts.length; i++) {
            emit log_named_uint("i", i);
            if (moduleProxy.isNode(accounts[i])) continue;
            moduleProxy.addNode(accounts[i]);
        }
    }

    /**
     * @dev return an array with all unique addresses which does not contain address zeo
     * return a random item
     */
    function _helperGetUniqueAddressArrayAndRandomItem(
        address[] memory addrs,
        uint256 randomIndex
    )
        private
        view
        returns (address[] memory, address)
    {
        if (addrs.length == 0) {
            return (new address[](0), address(0));
        } else if (addrs.length == 1) {
            return (addrs, addrs[0]);
        }

        // for addrs are more
        address[] memory results = addrs;
        for (uint256 i = 0; i < results.length; i++) {
            address cur = results[i];
            for (uint256 j = i + 1; j < results.length; j++) {
                if (cur == results[j]) {
                    delete results[i];
                    break;
                }
            }
        }

        randomIndex = bound(randomIndex, 0, results.length - 1);
        return (results, results[randomIndex]);
    }

    function _helperBuildMultiSendTx(
        uint8[] memory txOperations,
        address[] memory txTos,
        uint256[] memory txValues,
        uint256[] memory dataLengths,
        bytes[] memory data
    )
        private
        pure
        returns (bytes memory)
    {
        bytes memory encodePacked;
        for (uint256 i = 0; i < txOperations.length; i++) {
            encodePacked =
                abi.encodePacked(encodePacked, txOperations[i], txTos[i], txValues[i], dataLengths[i], data[i]);
        }
        return abi.encodeWithSignature("multiSend(bytes)", encodePacked);
    }

    function _helperAddTokenAndChannelTarget(
        address caller,
        CapabilityPermission[] memory channelsTokenPermission,
        CapabilityPermission[] memory nodePermission
    )
        private
    {
        // scope channels and token contract
        Target tokenChannelsTarget = TargetUtils.encodeDefaultPermissions(
            channels,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.SPECIFIC_FALLBACK_ALLOW,
            channelsTokenPermission
        ); // clerance: FUNCTION default ALLOW_ALL
        Target nodeTarget = TargetUtils.encodeDefaultPermissions(
            caller, Clearance.FUNCTION, TargetType.SEND, TargetPermission.SPECIFIC_FALLBACK_ALLOW, nodePermission
        ); // clerance: FUNCTION default ALLOW_ALL

        // set the multisend address to be multiaddr
        stdstore.target(address(moduleProxy)).sig("owner()").checked_write(safe);
        stdstore.target(address(moduleProxy)).sig("multisend()").checked_write(multiaddr);

        address owner = moduleProxy.owner();
        vm.startPrank(owner);

        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));
        vm.mockCall(safe, abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector), abi.encode(true));
        vm.deal(safe, 1 ether);
        assertEq(safe.balance, 1 ether);

        // add token and channels as accept_all target
        moduleProxy.addChannelsAndTokenTarget(tokenChannelsTarget);
        // include caller as node
        moduleProxy.addNode(caller);
        // add the node as a scoped target
        moduleProxy.scopeTargetSend(nodeTarget);
        vm.stopPrank();
    }
}
