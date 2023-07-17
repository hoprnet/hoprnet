// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import '../../../src/node-stake/permissioned-module/NodeManagementModule.sol';
import '../../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import "../../utils/CapabilityLibrary.sol";
import '../../../script/utils/SafeSuiteLib.sol';
import '../../utils/SafeSingleton.sol';
import '../../../src/interfaces/IAvatar.sol';
import 'forge-std/Test.sol';
import 'openzeppelin-contracts-4.8.3/token/ERC20/IERC20.sol';

/**
 * @dev This files tests both HoprNodeManagementModule and the CapabilityPermissions.sol
 */
contract HoprNodeManagementModuleTest is Test, CapabilityPermissionsLibFixtureTest, SafeSingletonFixtureTest {
    using stdStorage for StdStorage;

    HoprNodeManagementModule public moduleSingleton;
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

    function setUp() public virtual override(CapabilityPermissionsLibFixtureTest, SafeSingletonFixtureTest) {
        super.setUp();
        multiaddr = vm.addr(100); // make address(100) multiaddr
        safe = vm.addr(101); // make address(101) a safe
        channels = makeAddr("HoprChannels");
        token = makeAddr("HoprToken");

        moduleSingleton = new HoprNodeManagementModule();
        defaultFunctionPermission = new CapabilityPermission[](TargetUtils.NUM_CAPABILITY_PERMISSIONS);
        defaultFunctionPermission = [
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW,
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW,
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW,
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW,
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW,
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW,
            CapabilityPermission.SPECIFIC_FALLBACK_ALLOW,
            CapabilityPermission.SPECIFIC_FALLBACK_BLOCK,
            CapabilityPermission.SPECIFIC_FALLBACK_BLOCK
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

    /**
     * @dev Mock set avatar
     */
    function test_SetAvatar(address account) public {
        stdstore
            .target(address(moduleSingleton))
            .sig("avatar()")
            .checked_write(safe);

        assertEq(moduleSingleton.avatar(), safe);
    
        vm.prank(moduleSingleton.owner());
        vm.expectEmit(true, true, false, false, address(moduleSingleton));
        emit AvatarSet(safe, account);
        moduleSingleton.setAvatar(account);
    
        assertEq(moduleSingleton.avatar(), account);
        vm.clearMockedCalls();
    }

    /**
    * @dev fail to transfer ownership
    */
    function testRevert_TransferOwnership(address newAddress) public {
        address owner = moduleSingleton.owner();

        vm.startPrank(owner);
        vm.expectRevert(CannotChangeOwner.selector);
        moduleSingleton.transferOwnership(newAddress);
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

    function testRevert_AddNode(address[] memory accounts, uint256 index) public {
        vm.assume(accounts.length > 0);
        index = bound(index, 0, accounts.length - 1);

        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        _helperAddNodes(accounts);

        assertTrue(moduleSingleton.isNode(accounts[index]));
        vm.expectRevert(WithMembership.selector);
        moduleSingleton.addNode(accounts[index]);
        vm.stopPrank();
    }

    function testFuzz_RemoveNode(address[] memory accounts, uint256 index) public {
        vm.assume(accounts.length > 0);
        index = bound(index, 0, accounts.length - 1);

        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        _helperAddNodes(accounts);

        assertTrue(moduleSingleton.isNode(accounts[index]));
        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit NodeRemoved(accounts[index]);
        moduleSingleton.removeNode(accounts[index]);
        assertFalse(moduleSingleton.isNode(accounts[index]));
        vm.stopPrank();
    }

    function testRevert_RemoveNode(address[] memory accounts, address nodeAddress) public {
        address owner = moduleSingleton.owner();
        vm.startPrank(owner);

        _helperAddNodes(accounts);
        vm.assume(!moduleSingleton.isNode(nodeAddress));

        vm.expectRevert(HoprCapabilityPermissions.NoMembership.selector);
        moduleSingleton.removeNode(nodeAddress);
        vm.stopPrank();
    }

    function testFuzz_SetMultisend(address multisendAddr) public {
        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit SetMultisendAddress(multisendAddr);
        moduleSingleton.setMultisend(multisendAddr);
        vm.stopPrank();
        assertEq(moduleSingleton.multisend(), multisendAddr);
    }

    /**
    * @dev Add channels target(s) when the account is not address zero
    */
    function testFuzz_ScopeTargetChannelsFromModule(address channelsAddress) public {
        vm.assume(channelsAddress != address(0));
        address owner = moduleSingleton.owner();
        Target channelsTarget = TargetUtils.encodeDefaultPermissions(
            channelsAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit HoprCapabilityPermissions.ScopedTargetChannels(channelsAddress, channelsTarget);
        moduleSingleton.scopeTargetChannels(channelsTarget);
    }

    /**
    * @dev fail to channels target(s) when the account is address zero
    */
    function testRevert_ScopeZeroAddressTargetChannelsFromModule() public {
        address channelsAddress = address(0);
        address owner = moduleSingleton.owner();
        Target channelsTarget = TargetUtils.encodeDefaultPermissions(
            channelsAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        moduleSingleton.scopeTargetChannels(channelsTarget);
    }

    /**
    * @dev fail to add channels target(s) when the account has been scopec
    */
    function testRevert_ScopeExistingTargetChannelsFromModule(address[] memory channelsAddresses, uint256 randomIndex) public {
        vm.assume(channelsAddresses.length > 0);
        
        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) = _helperGetUniqueAddressArrayAndRandomItem(
            channelsAddresses,
            randomIndex
        );
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
            moduleSingleton.scopeTargetChannels(channelsTarget);
        }

        Target existingChannelsTarget = TargetUtils.encodeDefaultPermissions(
            oneAddress,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );
        vm.expectRevert(HoprCapabilityPermissions.TargetIsScoped.selector);
        moduleSingleton.scopeTargetChannels(existingChannelsTarget);
    }

    /**
    * @dev Add token target(s) when the account is not address zero
    */
    function testFuzz_ScopeTargetTokenFromModule(address tokenAddress) public {
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
    * @dev fail to token target(s) when the account is address zero
    */
    function testRevert_ScopeZeroAddressTargetTokenFromModule() public {
        address tokenAddress = address(0);
        address owner = moduleSingleton.owner();
        Target tokenTarget = TargetUtils.encodeDefaultPermissions(
            tokenAddress,
            Clearance.FUNCTION,
            TargetType.TOKEN,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        moduleSingleton.scopeTargetToken(tokenTarget);
    }

    /**
    * @dev fail to add token target(s) when the account has been scopec
    */
    function testRevert_ScopeExistingTargetTokenFromModule(address[] memory tokenAddresses, uint256 randomIndex) public {
        vm.assume(tokenAddresses.length > 0);
        
        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) = _helperGetUniqueAddressArrayAndRandomItem(
            tokenAddresses,
            randomIndex
        );
        vm.assume(results.length > 0);

        for (uint256 i = 0; i < results.length; i++) {
            vm.assume(results[i] != address(0));
            Target tokenTarget = TargetUtils.encodeDefaultPermissions(
                results[i],
                Clearance.FUNCTION,
                TargetType.TOKEN,
                TargetPermission.ALLOW_ALL,
                defaultFunctionPermission
            );

            moduleSingleton.scopeTargetToken(tokenTarget);
        }

        Target existingTokenTarget = TargetUtils.encodeDefaultPermissions(
                oneAddress,
                Clearance.FUNCTION,
                TargetType.TOKEN,
                TargetPermission.ALLOW_ALL,
                defaultFunctionPermission
            );

        vm.expectRevert(HoprCapabilityPermissions.TargetIsScoped.selector);
        moduleSingleton.scopeTargetToken(existingTokenTarget);
    }

    /**
    * @dev Add send target(s) when the account is a member
    */
    function testFuzz_ScopeTargetSendFromModule(address[] memory accounts, uint256 index) public {
        vm.assume(accounts.length > 0);
        index = bound(index, 0, accounts.length - 1);

        address owner = moduleSingleton.owner();
        vm.startPrank(owner);

        // add nodes and take one from the added node
        _helperAddNodes(accounts);
        assertTrue(moduleSingleton.isNode(accounts[index]));

        Target sendTarget = TargetUtils.encodeDefaultPermissions(
            accounts[index],
            Clearance.FUNCTION,
            TargetType.SEND,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit HoprCapabilityPermissions.ScopedTargetSend(accounts[index], sendTarget);
        moduleSingleton.scopeTargetSend(sendTarget);
    }

    /**
    * @dev fail to scope send target(s) when the account is address zero
    */
    function testRevert_ScopeNonMemberTargetSendFromModule() public {
        address tokenAddress = address(0);
        address owner = moduleSingleton.owner();
        Target sendTarget = TargetUtils.encodeDefaultPermissions(
            tokenAddress,
            Clearance.FUNCTION,
            TargetType.SEND,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);

        vm.expectRevert(HoprCapabilityPermissions.NoMembership.selector);
        moduleSingleton.scopeTargetSend(sendTarget);
    }
    /**
    * @dev fail to scope send target(s) when the account is address zero
    */
    function testRevert_ScopeZeroAddressTargetSendFromModule() public {
        address sendAddress = address(0);
        address owner = moduleSingleton.owner();
        Target sendTarget = TargetUtils.encodeDefaultPermissions(
            sendAddress,
            Clearance.FUNCTION,
            TargetType.SEND,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );

        vm.startPrank(owner);
        moduleSingleton.addNode(sendAddress);
        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        moduleSingleton.scopeTargetSend(sendTarget);
    }

    /**
    * @dev fail to add send target(s) when the account has been scopec
    */
    function testRevert_ScopeExistingTargetSendFromModule(address[] memory sendAddresses, uint256 randomIndex) public {
        vm.assume(sendAddresses.length > 0);
        
        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) = _helperGetUniqueAddressArrayAndRandomItem(
            sendAddresses,
            randomIndex
        );
        vm.assume(results.length > 0);
        // add nodes and take one from the added node
        _helperAddNodes(results);

        for (uint256 i = 0; i < results.length; i++) {
            vm.assume(results[i] != address(0));
            assertTrue(moduleSingleton.isNode(results[i]));
            Target sendTarget = TargetUtils.encodeDefaultPermissions(
                results[i],
                Clearance.FUNCTION,
                TargetType.SEND,
                TargetPermission.ALLOW_ALL,
                defaultFunctionPermission
            );

            moduleSingleton.scopeTargetSend(sendTarget);
        }

        Target existingSendTarget = TargetUtils.encodeDefaultPermissions(
                oneAddress,
                Clearance.FUNCTION,
                TargetType.SEND,
                TargetPermission.ALLOW_ALL,
                defaultFunctionPermission
            );

        vm.expectRevert(HoprCapabilityPermissions.TargetIsScoped.selector);
        moduleSingleton.scopeTargetSend(existingSendTarget);
    }

    /**
    * @dev Add Channels and Token targets, where channel is vm.addr()
    */
    function test_AddChannelsAndTokenTarget(uint256 targetUint) public {
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

    // test revokeTarget
    /**
    * @dev owner revoke a target
    */
    function testFuzz_RevokeTargetFromModule(address[] memory accounts, uint256 randomIndex) public {
        // scope some targets
        vm.assume(accounts.length > 0);
        address owner = moduleSingleton.owner();
        vm.startPrank(owner);
        (address[] memory results, address oneAddress) = _helperGetUniqueAddressArrayAndRandomItem(
            accounts,
            randomIndex
        );
        for (uint256 i = 0; i < results.length; i++) {
            vm.assume(results[i] != address(0));
            Target tokenTarget = TargetUtils.encodeDefaultPermissions(
                results[i],
                Clearance.FUNCTION,
                TargetType.TOKEN,
                TargetPermission.ALLOW_ALL,
                defaultFunctionPermission
            );

            moduleSingleton.scopeTargetToken(tokenTarget);
        }

        // remove target
        vm.expectEmit(true, false, false, false, address(moduleSingleton));
        emit HoprCapabilityPermissions.RevokedTarget(oneAddress);
        moduleSingleton.revokeTarget(oneAddress);
    }

    /**
    * @dev fail to remove a target that is not scoped
    */
    function testRevert_RevokeNonScopedTargetFromModule(address scopeTargetAddr) public {
        address owner = moduleSingleton.owner();

        vm.startPrank(owner);
        vm.expectRevert(HoprCapabilityPermissions.TargetIsNotScoped.selector);
        moduleSingleton.revokeTarget(scopeTargetAddr);
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
            permissions[i] = GranularPermission(uint8(uint256(bytes32(_randomFunctionSigs[i])) % (uint256(type(GranularPermission).max) + 1)));
        }
        (bytes32 encoded, uint256 length) = HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
        assertEq(length, _size);
        // scope capabilities
        vm.prank(moduleSingleton.owner());
        for (uint256 j = 0; j < length; j++) {
            if (functionSigs[j] != bytes4(hex"00")) {
                vm.expectEmit(true, true, false, false, address(moduleSingleton));
                emit HoprCapabilityPermissions.ScopedGranularChannelCapability(vm.addr(200), bytes32(hex"0200"), functionSigs[j], permissions[j]);
            }
        }
        moduleSingleton.scopeChannelsCapabilities(
            vm.addr(200),       // mocked targetAddress
            bytes32(hex"0200"), // mocked channelId
            encoded             // encodeSigsPermissions
        );
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
            permissions[i] = GranularPermission(uint8(uint256(bytes32(_randomFunctionSigs[i])) % (uint256(type(GranularPermission).max) + 1)));
        }
        (bytes32 encoded, uint256 length) = HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
        assertEq(length, _size);
        // scope capabilities
        vm.prank(moduleSingleton.owner());
        for (uint256 j = 0; j < length; j++) {
            if (functionSigs[j] != bytes4(hex"00") && j < 2) {
                vm.expectEmit(true, true, false, false, address(moduleSingleton));
                emit HoprCapabilityPermissions.ScopedGranularTokenCapability(vm.addr(200), vm.addr(201), vm.addr(202), functionSigs[j], permissions[j]);
            }
        }
        moduleSingleton.scopeTokenCapabilities(
            vm.addr(200),       // mocked nodeAddress
            vm.addr(201),       // mocked targetAddress
            vm.addr(202),       // mocked beneficiary
            encoded             // encodeSigsPermissions
        );
    }

    /**
     * @dev scope tokens for (source, destination)
     */
    function testFuzz_ScopeSendCapability(bytes4 functionSig) public {
        GranularPermission permission = GranularPermission(uint8(uint256(bytes32(functionSig)) % (uint256(type(GranularPermission).max) + 1)));

        // scope capabilities
        vm.prank(moduleSingleton.owner());
        if (functionSig != bytes4(hex"00")) {
            vm.expectEmit(true, true, false, false, address(moduleSingleton));
            emit HoprCapabilityPermissions.ScopedGranularSendCapability(vm.addr(200), vm.addr(201), permission);
        }
        moduleSingleton.scopeSendCapability(
            vm.addr(200),       // mocked nodeAddress
            vm.addr(201),       // mocked beneficiary
            permission          // encodeSigsPermissions
        );
    }

    /**
     * @dev call transaction execution from a non-member account
     */
    function testRevert_CallFromNonMember(address caller) public {
        vm.assume(caller != address(0));
        vm.assume(caller != vm.addr(301));
        address owner = moduleSingleton.owner();

        vm.prank(owner);
        // include some node as member
        moduleSingleton.addNode(vm.addr(301));
        // cannot call from
        assertFalse(moduleSingleton.isNode(caller));
        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.NoMembership.selector);
        moduleSingleton.execTransactionFromModule(
            token,
            0,
            hex"12345678",
            Enum.Operation.Call
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev call transaction execution from a non-member account
     */
    function testRevert_CallWithInvalidData() public {
        address owner = moduleSingleton.owner();
        address caller = vm.addr(301);

        vm.prank(owner);
        // include some node as member
        moduleSingleton.addNode(caller);
        // cannot call from
        vm.prank(caller);
        vm.expectRevert(HoprCapabilityPermissions.FunctionSignatureTooShort.selector);
        moduleSingleton.execTransactionFromModule(
            token,
            0,
            hex"00",
            Enum.Operation.Call
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute a transaction from the module to a scoped target
     */
    function test_ExecTransactionFromModuleToAScopedTarget() public {
        // scope channels and token contract
        address owner = moduleSingleton.owner();
        address msgSender = vm.addr(1);

        Target target = TargetUtils.encodeDefaultPermissions(
            channels,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );   // clerance: FUNCTION default ALLOW_ALL
        vm.startPrank(owner);
        stdstore
            .target(address(moduleSingleton))
            .sig("avatar()")
            .checked_write(safe);
        vm.mockCall(
            channels,
            abi.encodeWithSignature(
                'token()'
            ),
            abi.encode(token)
        );
        vm.mockCall(
            safe,
            abi.encodeWithSelector(IAvatar.execTransactionFromModule.selector),
            abi.encode(true)
        );

        // add token and channels as accept_all target
        moduleSingleton.addChannelsAndTokenTarget(channels, target);
        // include caller as node
        moduleSingleton.addNode(msgSender);

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100);
        vm.stopPrank();

        // execute function
        vm.prank(msgSender);
        bool result = moduleSingleton.execTransactionFromModule(
            token,
            0,
            data,
            Enum.Operation.Call
        );
        assertTrue(result);
        vm.clearMockedCalls();
    }

    /**
     * @dev should successfully execute a transaction from the module to a scoped target
     */
    function test_ExecTransactionFromModuleReturnData() public {
        // scope channels and token contract
        address owner = moduleSingleton.owner();
        address msgSender = vm.addr(1);

        Target target = TargetUtils.encodeDefaultPermissions(
            channels,
            Clearance.FUNCTION,
            TargetType.CHANNELS,
            TargetPermission.ALLOW_ALL,
            defaultFunctionPermission
        );   // clerance: FUNCTION default ALLOW_ALL
        vm.startPrank(owner);
        stdstore
            .target(address(moduleSingleton))
            .sig("avatar()")
            .checked_write(safe);
        vm.mockCall(
            channels,
            abi.encodeWithSignature(
                'token()'
            ),
            abi.encode(token)
        );
        vm.mockCall(
            safe,
            abi.encodeWithSelector(IAvatar.execTransactionFromModuleReturnData.selector),
            abi.encode(true, hex"12345678")
        );

        // add token and channels as accept_all target
        moduleSingleton.addChannelsAndTokenTarget(channels, target);
        // include caller as node
        moduleSingleton.addNode(msgSender);

        // prepare a simple token approve
        bytes memory data = abi.encodeWithSelector(IERC20.approve.selector, vm.addr(200), 100);
        vm.stopPrank();

        // execute function
        vm.prank(msgSender);
        (bool success, bytes memory result) = moduleSingleton.execTransactionFromModuleReturnData(
            token,
            0,
            data,
            Enum.Operation.Call
        );
        assertTrue(success);
        assertEq(result, hex"12345678");
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
            if (moduleSingleton.isNode(accounts[i])) continue;
            moduleSingleton.addNode(accounts[i]);
        }
    }


    /**
     * @dev return an array with all unique addresses which does not contain address zeo
     * return a random item
     */
    function _helperGetUniqueAddressArrayAndRandomItem(
        address[] memory addrs,
        uint256 randomIndex
    ) private returns (address[] memory, address) {
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
}
