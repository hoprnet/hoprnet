// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import { Test } from "forge-std/Test.sol";

import { HoprNodeManagementModule } from "../../src/node-stake/permissioned-module/NodeManagementModule.sol";
import { HoprCapabilityPermissions } from "../../src/node-stake/permissioned-module/CapabilityPermissions.sol";
import { HoprNodeStakeFactory, HoprNodeStakeFactoryEvents } from "../../src/node-stake/NodeStakeFactory.sol";
import { Safe } from "safe-contracts-1.4.1/Safe.sol";
import { SafeSuiteLibV141 } from "../../src/utils/SafeSuiteLibV141.sol";
import { SafeSuiteLibV150 } from "../../src/utils/SafeSuiteLibV150.sol";
import { Enum, IAvatar } from "../../src/interfaces/IAvatar.sol";
import { SafeSingletonFixtureTest } from "../utils/SafeSingleton.sol";
import { Clones } from "openzeppelin-contracts-5.4.0/proxy/Clones.sol";
import { SafeProxyFactory } from "safe-contracts-1.4.1/proxies/SafeProxyFactory.sol";
import { HoprToken } from "../../src/static/HoprToken.sol";
import { ERC1820RegistryFixtureTest } from "../utils/ERC1820Registry.sol";
import { IERC1820Registry } from "openzeppelin-contracts-5.4.0/interfaces/IERC1820Registry.sol";
import { TargetUtils, Target } from "../../src/utils/TargetUtils.sol";

contract MaliciousModuleMock {
    function initialize(bytes calldata) public pure {
        return;
    }
}

contract HoprNodeStakeFactoryTest is Test, ERC1820RegistryFixtureTest, SafeSingletonFixtureTest, HoprNodeStakeFactoryEvents {
    using Clones for address;
    using TargetUtils for Target;

    HoprNodeManagementModule public moduleSingleton;
    HoprNodeStakeFactory public factory;
    HoprToken public hoprToken;
    address public caller;
    address public admin;
    address public module;
    address payable public safe;

    address constant CHANNELS = 0x0101010101010101010101010101010101010101;
    address constant ANNOUNCEMENT = 0x0202020202020202020202020202020202020202;
    bytes32 public constant ANNOUNCEMENT_TARGET =
        bytes32(hex"0202020202020202020202020202020202020202010101010101010101010000");
    bytes32 public constant DEFAULT_TARGET =
        bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101");
    bytes4 public deploySafeAndModuleSelector = bytes4(keccak256("_deploySafeAndModule(uint256,bytes32,address,address,uint256,address[])"));
    bytes4 public deploySafeAndModuleAndIncludeNodesSelector = bytes4(keccak256("_deploySafeAndModuleAndIncludeNodes(uint256,bytes32,address,address,uint256,address[])"));

    /**
     * Manually import events and errors
     */
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event SetMultisendAddress(address indexed multisendAddress);

    function setUp() public override(ERC1820RegistryFixtureTest, SafeSingletonFixtureTest) {
        ERC1820RegistryFixtureTest.setUp();
        SafeSingletonFixtureTest.setUp();
        // deploy safe suites
        deployEntireSafeSuite();

        caller = vm.addr(101); // make make address(101) a caller
        admin = vm.addr(102); // make make address(102) an admin

        hoprToken = new HoprToken();
        moduleSingleton = new HoprNodeManagementModule();
        factory = new HoprNodeStakeFactory(address(moduleSingleton), ANNOUNCEMENT, admin);

        // grant minter role to the test contract itself
        vm.prank(address(this));
        hoprToken.grantRole(hoprToken.MINTER_ROLE(), address(this));
    }

    modifier mockTokenChannel() {
        vm.mockCall(CHANNELS, abi.encodeWithSignature("TOKEN()"), abi.encode(address(hoprToken)));
    
        (, uint256 defaultAllowance, bytes32 defaultAnnouncement) = factory.defaultHoprNetwork();
        vm.prank(admin);
        factory.updateHoprNetwork(HoprNodeStakeFactory.HoprNetwork({
            tokenAddress: address(hoprToken),
            defaultAnnouncementTarget: defaultAnnouncement,
            defaultTokenAllowance: defaultAllowance
        }));
        _;
    }

    /**
     * @dev preflight check if all the safe contracts are well set
     */
    function test_SafeSuiteSetup() public {
        // there's code in Singleton contract
        assertTrue(hasSingletonContract());
        // there's code in ERC1820 contract
        assertTrue(hasErc1820Registry());
        // there's code in Safe Singleton
        assertGt(SafeSuiteLibV141.SAFE_Safe_ADDRESS.code.length, 0);
        // there's code in Safe Proxy Factory
        assertGt(SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS.code.length, 0);
        // there's code in Safe MultiSendCallOnly
        assertGt(SafeSuiteLibV141.SAFE_MultiSendCallOnly_ADDRESS.code.length, 0);
        // there's code in Safe ExtensibleFallbackHandler, v1.5.0
        assertGt(SafeSuiteLibV150.SAFE_ExtensibleFallbackHandler_ADDRESS.code.length, 0);
        // safe version matches
        assertEq(factory.safeVersion(), SafeSuiteLibV141.SAFE_VERSION);
    }

    function test_UpdateHoprNetwork() public {
        HoprNodeStakeFactory.HoprNetwork memory newHoprNetwork = HoprNodeStakeFactory.HoprNetwork({
            tokenAddress: address(hoprToken),
            defaultTokenAllowance: 2000 ether,
            defaultAnnouncementTarget: bytes32(uint256(uint160(ANNOUNCEMENT))) << 96 | bytes32(uint256(0x010103030303030303030000))
        });

        vm.prank(admin);
        vm.expectEmit(true, false, false, true, address(factory));
        emit HoprNodeStakeHoprNetworkUpdated(newHoprNetwork);
        factory.updateHoprNetwork(newHoprNetwork);

        (address token, uint256 allowance, bytes32 announcement) = factory.defaultHoprNetwork();
        assertEq(token, address(hoprToken), "wrong token address");
        assertEq(allowance, newHoprNetwork.defaultTokenAllowance, "wrong allowance");
        assertEq(announcement, newHoprNetwork.defaultAnnouncementTarget, "wrong announcement");
        vm.clearMockedCalls();
    }

    /**
     * @dev Fail to clone a safe when there's not event one admin
     */
    function testRevert_CloneSafeAndModuleWithFewOwner() public mockTokenChannel {
        uint256 nonce = 0;
        address[] memory admins = new address[](0);

        vm.prank(caller);
        vm.expectRevert(HoprNodeStakeFactory.TooFewOwners.selector);
        (module, safe) = factory.clone(
            nonce,
            DEFAULT_TARGET,
            admins
        );
        vm.clearMockedCalls();
    }

    function testRevert_CloneSafeAndModuleWithStakeFactoryAsOwner() public mockTokenChannel {
        uint256 nonce = 0;
        address[] memory admins = new address[](2);
        admins[0] = vm.addr(103); // add another admin
        admins[1] = address(factory); // add factory address as an admin

        vm.prank(caller);
        vm.expectRevert(HoprNodeStakeFactory.InvalidOwner.selector);
        (module, safe) = factory.clone(
            nonce,
            DEFAULT_TARGET,
            admins
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev Clone a safe and a module and they are wired
     */
    function test_CloneSafeAndModule() public mockTokenChannel {
        uint256 nonce = 0;
        address[] memory admins = new address[](10);
        for (uint256 i = 0; i < admins.length; i++) {
            admins[i] = vm.addr(200 + i);
        }
        (address expectedSafeAddress, address expectedModuleAddress) = _helperPredictSafeAndModule(admins, caller, nonce);

        vm.startPrank(caller);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress);

        (module, safe) = factory.clone(
            nonce,
            DEFAULT_TARGET,
            admins
        );

        assertEq(module, expectedModuleAddress, "module address mismatch");
        assertEq(safe, expectedSafeAddress, "safe address mismatch");

        _ensureSafeAndModuleAreWired(module, payable(safe), admins);

        // compare token allowance
        assertEq(Target.wrap(uint256(DEFAULT_TARGET)).getTargetAddress(), CHANNELS, "wrong channels address");
        (, uint256 defaultAllowance, ) = factory.defaultHoprNetwork();
        assertEq(
            hoprToken.allowance(safe, CHANNELS),
            defaultAllowance,
            "wrong token allowance"
        );
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Clone a safe and a module, and fund the safe with tokens, all in one transaction
     */
    function test_OneClickDeploySafeWithFundToken() public mockTokenChannel {
        // prepare admins
        address[] memory admins = new address[](3);
        for (uint256 i = 0; i < admins.length; i++) {
            admins[i] = vm.addr(300 + i);
        }
        uint256 nonce = 13;
        uint256 amount = 5000 ether;
        // mint some tokens to caller
        vm.prank(address(this));
        hoprToken.mint(caller, amount, "", "");
        assertEq(hoprToken.balanceOf(caller), amount, "caller should have some tokens");

        // prepare userData
        bytes memory userData = abi.encode(factory.DEPLOYSAFEMODULE_FUNCTION_IDENTIFIER(), nonce, DEFAULT_TARGET, admins);

        // calculate expected safe and module address
        (address expectedSafeAddress, address expectedModuleAddress) = _helperPredictSafeAndModule(admins, caller, nonce);

        vm.startPrank(caller);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress);

        // deploy safe and module, and fund the safe with tokens, all in one transaction
        hoprToken.send(address(factory), amount, userData);

        // safe should receive all the token
        assertEq(hoprToken.balanceOf(expectedSafeAddress), amount, "safe should receive all the tokens");
        assertEq(hoprToken.balanceOf(caller), 0, "caller should not have any tokens");
        // channel could transfer some tokens from the safe
        (, uint256 defaultAllowance, ) = factory.defaultHoprNetwork();
        assertEq(hoprToken.allowance(expectedSafeAddress, CHANNELS), defaultAllowance, "wrong token allowance");
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function test_OneClickDeploySafeWithFundTokenAndIncludeNodes() public mockTokenChannel {
        // prepare admins
        address[] memory admins = new address[](2);
        admins[0] = vm.addr(700);
        admins[1] = vm.addr(701);
        uint256 nonce = 14;
        uint256 amount = 8000 ether;
        // mint some tokens to caller
        vm.prank(address(this));
        hoprToken.mint(caller, amount, "", "");
        assertEq(hoprToken.balanceOf(caller), amount, "caller should have some tokens");

        // prepare userData
        bytes memory userData = abi.encode(factory.DEPLOYSAFEANDMODULEANDINCLUDENODES_IDENTIFIER(), nonce, DEFAULT_TARGET, admins);
        emit log_bytes(userData);

        // calculate expected safe and module address
        (address expectedSafeAddress, address expectedModuleAddress) = _helperPredictSafeAndModule(admins, caller, nonce);

        vm.startPrank(caller);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress);

        // deploy safe and module, and fund the safe with tokens, all in one transaction
        hoprToken.send(address(factory), amount, userData);

        // safe should receive all the token
        assertEq(hoprToken.balanceOf(expectedSafeAddress), amount, "safe should receive all the tokens");
        assertEq(hoprToken.balanceOf(caller), 0, "caller should not have any tokens");
        // channel could transfer some tokens from the safe
        (, uint256 defaultAllowance, ) = factory.defaultHoprNetwork();
        assertEq(hoprToken.allowance(expectedSafeAddress, CHANNELS), defaultAllowance, "wrong token allowance");

        // check admins are nodes being included in the module
        for (uint256 i = 0; i < admins.length; i++) {
            (bool success, bytes memory retdata) = expectedModuleAddress.call(abi.encodeWithSignature("isNode(address)", admins[i]));
            assertTrue(success && abi.decode(retdata, (bool)), "admin should be included in the module");
        }
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Clone multiple safes and modules to ensure the nonce and salt works
     *      and the deployed addresses are unique
     */
    function test_CloneMultipleSafesAndModules() public mockTokenChannel {
        // Deploy first safe and module, with 3 admins
        uint256 nonce0 = 0;
        address[] memory admins0 = new address[](3);
        for (uint256 i = 0; i < admins0.length; i++) {
            admins0[i] = vm.addr(350 + i);
        }
        (address module0, address payable safe0) = factory.clone(
            nonce0,
            DEFAULT_TARGET,
            admins0
        );
        _ensureSafeAndModuleAreWired(module0, payable(safe0), admins0);

        // Deploy second safe and module, with 3 admins
        uint256 nonce1 = 1;
        address[] memory admins1 = new address[](1);
        admins1[0] = vm.addr(400);
        (address module1, address payable safe1) = factory.clone(
            nonce1,
            DEFAULT_TARGET,
            admins1
        );
        _ensureSafeAndModuleAreWired(module1, payable(safe1), admins1);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function test_CloneMultipleSafesAndModulesWithMultiSend() public mockTokenChannel {
        // create a safe to call multisend
        address safe1 = _helperDeployASafe(vm.addr(400));

        // Test with multisend to create two safe-module pairs, from the previously deployed safe
        vm.startPrank(vm.addr(400));
        (
            address expectedSafeAddress3,
            address expectedModuleAddress3,
            address expectedSafeAddress4,
            address expectedModuleAddress4,
            bytes memory safeTxData
        ) = _helperMultiSendDeploy(safe1);

        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress3);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress3);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress4);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress4);
        // The first two Safe txns were used during the deployment of safe1
        _helperSafeTxnToMultiSend(IAvatar(safe1), 400, 0, safeTxData);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function test_CloneOnlyAModuleFromAnExistingSafe() public mockTokenChannel {
        address owner = vm.addr(666);
        uint256 nonce = 666;
        address[] memory admins = new address[](1);
        admins[0] = owner;
        // create a safe to call multisend
        address safe1 = _helperDeployASafe(owner);

        // predict the module address
        address expectedModuleAddress = factory.predictModuleAddress(safe1, nonce, safe1, DEFAULT_TARGET);

        // Test with multisend to create two safe-module pairs, from the previously deployed safe
        vm.startPrank(safe1);

        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModuleForSafe(expectedModuleAddress, safe1);
        // The first two Safe txns were used during the deployment of safe1
        factory.deployModule(safe1, DEFAULT_TARGET, nonce);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testFuzz_InitializeModuleProxy(uint256 nonce, address safeAddr, address multisendAddr) public mockTokenChannel {
        vm.assume(safeAddr != address(0));
        vm.assume(multisendAddr != address(0));
        vm.assume(multisendAddr != safeAddr);
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);
        // add Safe and multisend to the module
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(
                safeAddr, multisendAddr, ANNOUNCEMENT_TARGET, DEFAULT_TARGET
            )
        );

        vm.expectEmit(true, true, false, false, address(moduleProxy));
        emit OwnershipTransferred(address(0), safeAddr);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit SetMultisendAddress(multisendAddr);
        (bool success,) = moduleProxy.call(moduleInitializer);
        assertTrue(success);
        vm.clearMockedCalls();
    }

    function testRevert_CloneButFailToInitializeWithSafeAddressZero(uint256 nonce, address multisendAddr) public {
        vm.assume(multisendAddr != address(0));

        address safeAddr = address(0);

        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);

        // initialize module proxy with invalid variables
        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        // add Safe and multisend to the module
        bytes memory moduleInitializer =
            abi.encodeWithSignature("initialize(bytes)", abi.encode(safeAddr, multisendAddr));
        (bool success,) = moduleProxy.call(moduleInitializer);
        assertFalse(success);
        vm.clearMockedCalls();
    }

    function testRevert_CloneButFailToInitializeWithMultisendAddressZero(uint256 nonce, address safeAddr) public {
        vm.assume(safeAddr != address(0));

        address multisendAddr = address(0);

        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);

        // initialize module proxy with invalid variables
        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        // add Safe and multisend to the module
        bytes memory moduleInitializer =
            abi.encodeWithSignature("initialize(bytes)", abi.encode(safeAddr, multisendAddr));
        (bool success,) = moduleProxy.call(moduleInitializer);
        assertFalse(success);
        vm.clearMockedCalls();
    }

    function testRevert_CloneButFailToInitializeWithMultisendSameAddress(uint256 nonce, address safeAddr) public {
        vm.assume(safeAddr != address(0));

        address multisendAddr = safeAddr;

        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);

        // initialize module proxy with invalid variables
        vm.expectRevert(HoprNodeManagementModule.SafeMultisendSameAddress.selector);
        // add Safe and multisend to the module
        bytes memory moduleInitializer =
            abi.encodeWithSignature("initialize(bytes)", abi.encode(safeAddr, multisendAddr));
        (bool success,) = moduleProxy.call(moduleInitializer);
        assertFalse(success);
        vm.clearMockedCalls();
    }

    function testRevert_CloneButFailToInitializeTwice(uint256 nonce, address safeAddr, address multisendAddr) public mockTokenChannel {
        vm.assume(safeAddr != address(0) && multisendAddr != address(0));

        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));

        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);

        // initialize module proxy with valid variables
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(
                address(1), address(2), ANNOUNCEMENT_TARGET, DEFAULT_TARGET
            )
        );
        (bool result,) = moduleProxy.call(moduleInitializer);
        // must not revert
        assertTrue(result);
        // re-initialize module proxy with variables
        bytes memory moduleReinitializer =
            abi.encodeWithSignature("initialize(bytes)", abi.encode(safeAddr, multisendAddr));
        // vm.expectRevert(AlreadyInitialized.selector);
        (bool secondResult,) = moduleProxy.call(moduleReinitializer);
        // must revert
        assertFalse(secondResult);
        vm.clearMockedCalls();
    }

    function testRevert_SafeSetupReuseNonce() public mockTokenChannel {
        uint256 nonce = 0;
        address[] memory admins = new address[](10);
        for (uint256 i = 0; i < admins.length; i++) {
            admins[i] = vm.addr(200 + i);
        }

        vm.startPrank(admins[0]);
        factory.clone(
            nonce,
            DEFAULT_TARGET,
            admins
        );

        vm.expectRevert(bytes("Create2 call failed"));
        factory.clone(
            nonce,
            DEFAULT_TARGET,
            admins
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev ensure the fallback handler is the can receive minting of ERC777 tokens
     */
    function test_FallbackHandler() public mockTokenChannel {
        // create a safe with the factory
        address[] memory admins = new address[](10);
        for (uint256 i = 0; i < admins.length; i++) {
            admins[i] = vm.addr(230 + i);
        }
        uint256 nonce = 12345;
        (module, safe) = factory.clone(
            nonce,
            DEFAULT_TARGET,
            admins
        );

        // get interface implementer
        address implementer = IERC1820Registry(ERC1820_REGISTRY_ADDRESS).getInterfaceImplementer(safe, keccak256("ERC777TokensRecipient"));
        assertEq(implementer, safe, "safe is not the ERC777TokensRecipient implementer");

        assertEq(hoprToken.balanceOf(safe), 0, "safe should not have any tokens");
        // mint some tokens to the safe
        hoprToken.mint(safe, 100000, "", "");

        assertEq(hoprToken.balanceOf(safe), 100000, "safe should have 100000 tokens");
        vm.clearMockedCalls();
    }

    /**
     * @dev internal function to ensure the safe and module are properly wired after cloning
     */
    function _ensureSafeAndModuleAreWired(address moduleAddr, address payable safeAddr, address[] memory admins) internal view {
        // Safe should have module enabled
        assertTrue(Safe(safeAddr).isModuleEnabled(moduleAddr));
        // Safe should have 1 threshold
        assertEq(Safe(safeAddr).getThreshold(), 1, "Wrong threshold");
        // Safe should have admin as the only owner
        address[] memory owners = Safe(safeAddr).getOwners();
        assertEq(owners.length, admins.length, "Wrong number of owners");
        for (uint256 j = 0; j < admins.length; j++) {
            assertTrue(Safe(safeAddr).isOwner(admins[j]));
        }
        assertFalse(Safe(safeAddr).isOwner(address(factory)));
        // module owner should be safe
        assertEq(HoprNodeManagementModule(moduleAddr).owner(), safeAddr, "Wrong module owner");
        // module multisend should beSafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS
        assertEq(
            HoprNodeManagementModule(moduleAddr).multisend(),
            SafeSuiteLibV141.SAFE_MultiSend_ADDRESS,
            "Wrong SafeMultiSend contract"
        );
    }

    /**
     * @dev internal function to help build the multiSend transaction data  
     */
    function _helperBuildMultiSendTxForSafeModuleClone(
        address factoryAddress,
        uint256[] memory dataLengths,
        bytes[] memory data
    )
        private
        pure
        returns (bytes memory)
    {
        bytes memory encodePacked;
        for (uint256 i = 0; i < dataLengths.length; i++) {
            encodePacked = abi.encodePacked(
                encodePacked,
                uint8(0), // txOperations[i] is CALL
                factoryAddress, // txTos[i],
                uint256(0), // txValues[i],
                dataLengths[i],
                data[i]
            );
        }
        return abi.encodeWithSignature("multiSend(bytes)", encodePacked);
    }

    /**
     * @dev when caller is owner of safe instance, prepare a signature and execute the transaction
     */
    function _helperSafeTxnToMultiSend(IAvatar safeInstance, uint256 senderPrivateKey, uint256 nonce, bytes memory data) private {
        address sender = vm.addr(senderPrivateKey);
        bytes32 dataHash =
            safeInstance.getTransactionHash(SafeSuiteLibV141.SAFE_MultiSend_ADDRESS, 0, data, Enum.Operation.DelegateCall, 0, 0, 0, address(0), sender, nonce);

        // sign dataHash
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(senderPrivateKey, dataHash);
        safeInstance.execTransaction(
            SafeSuiteLibV141.SAFE_MultiSend_ADDRESS,
            0,
            data,
            Enum.Operation.DelegateCall,
            0,
            0,
            0,
            address(0),
            payable(sender),
            abi.encodePacked(r, s, v)
        );
    }

    function _helperDeployASafe(address admin0) private returns (address safeProxyAddr) {
        address[] memory admins = new address[](1);
        (
            address safeAddress,
            address safeProxyFactoryAddress,
            address compatibilityFallbackHandlerAddress,
        ) = factory.safeLibAddresses();
        admins[0] = admin0;
        bytes memory safeInitializer = abi.encodeWithSignature(
            "setup(address[],uint256,address,bytes,address,address,uint256,address)",
            admins,
            1, // threshold
            address(0),
            hex"00",
            compatibilityFallbackHandlerAddress,
            address(0),
            0,
            address(0)
        );

        return address(SafeProxyFactory(safeProxyFactoryAddress).createProxyWithNonce(
            safeAddress, safeInitializer, 0
        ));
    }

    function _helperMultiSendDeploy(address callerSafe) private returns (
        address expectedSafeAddress1,
        address expectedModuleAddress1,
        address expectedSafeAddress2,
        address expectedModuleAddress2,
        bytes memory safeTxData
    ){
        uint256 nonce1 = 7;
        uint256 nonce2 = 8;
        address[] memory admins1 = new address[](3);
        admins1[0] = vm.addr(500);
        admins1[1] = vm.addr(501);
        admins1[2] = vm.addr(502);
        address[] memory admins2 = new address[](2);
        admins2[0] = vm.addr(600);
        admins2[1] = vm.addr(601);
    
        bytes[] memory data = new bytes[](2);
        data[0] = abi.encodeWithSelector(HoprNodeStakeFactory.clone.selector, nonce1, DEFAULT_TARGET, admins1); // approve on token
        data[1] = abi.encodeWithSelector(HoprNodeStakeFactory.clone.selector, nonce2, DEFAULT_TARGET, admins2); // approve on token
        uint256[] memory dataLengths = new uint256[](2);
        dataLengths[0] = data[0].length;
        dataLengths[1] = data[1].length;

        // safe1 uses multisend to deploy another safe and module
        safeTxData = _helperBuildMultiSendTxForSafeModuleClone(address(factory), dataLengths, data);
        (expectedSafeAddress1, expectedModuleAddress1) = _helperPredictSafeAndModule(admins1, callerSafe, nonce1);
        (expectedSafeAddress2, expectedModuleAddress2) = _helperPredictSafeAndModule(admins2, callerSafe, nonce2);
    }

    function _helperPredictSafeAndModule(address[] memory admins, address txCaller, uint256 nonce) private view returns (
        address expectedSafeAddress,
        address expectedModuleAddress
    ) {
        expectedSafeAddress = factory.predictSafeAddress(admins, nonce);
        expectedModuleAddress = factory.predictModuleAddress(txCaller, nonce, expectedSafeAddress, DEFAULT_TARGET);
    }
}
