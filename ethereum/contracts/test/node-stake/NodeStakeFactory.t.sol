// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import { Test } from "forge-std/Test.sol";

import { HoprNodeManagementModule } from "../../src/node-stake/permissioned-module/NodeManagementModule.sol";
import { HoprCapabilityPermissions } from "../../src/node-stake/permissioned-module/CapabilityPermissions.sol";
import { HoprNodeStakeFactory, HoprNodeStakeFactoryEvents } from "../../src/node-stake/NodeStakeFactory.sol";
import { Safe } from "safe-contracts-1.4.1/Safe.sol";
import { SafeSuiteLibV141 } from "../../src/utils/SafeSuiteLibV141.sol";
import { Enum, ISafe } from "../../src/utils/ISafe.sol";
import { SafeSingletonFixtureTest } from "../utils/SafeSingleton.sol";
import { ClonesUpgradeable } from "openzeppelin-contracts-upgradeable-4.9.2/proxy/ClonesUpgradeable.sol";
import { SafeProxyFactory } from "safe-contracts-1.4.1/proxies/SafeProxyFactory.sol";

contract MaliciousModuleMock {
    function initialize(bytes calldata) public pure {
        return;
    }
}

contract HoprNodeStakeFactoryTest is Test, SafeSingletonFixtureTest, HoprNodeStakeFactoryEvents {
    using ClonesUpgradeable for address;

    HoprNodeManagementModule public moduleSingleton;
    HoprNodeStakeFactory public factory;
    address public caller;
    address public admin;
    address public module;
    address payable public safe;

    bytes32 public constant DEFAULT_TARGET =
        bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101");

    /**
     * Manually import events and errors
     */
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event SetMultisendAddress(address indexed multisendAddress);

    function setUp() public override(SafeSingletonFixtureTest) {
        super.setUp();
        // deploy safe suites
        deployEntireSafeSuite();

        caller = vm.addr(101); // make make address(101) a caller
        admin = vm.addr(102); // make make address(102) an admin
        moduleSingleton = new HoprNodeManagementModule();
        factory = new HoprNodeStakeFactory(address(moduleSingleton), admin);
    }

    modifier mockTokenChannel() {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));
        _;
    }

    /**
     * @dev preflight check if all the safe contracts are well set
     */
    function test_SafeSuiteSetup() public {
        // there's code in Singleton contract
        assertTrue(hasSingletonContract());
        // there's code in Safe Singleton
        assertGt(SafeSuiteLibV141.SAFE_Safe_ADDRESS.code.length, 0);
        // there's code in Safe Proxy Factory
        assertGt(SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS.code.length, 0);
        // there's code in Safe MultiSendCallOnly
        assertGt(SafeSuiteLibV141.SAFE_MultiSendCallOnly_ADDRESS.code.length, 0);
        // there's code in Safe CompatibilityFallbackHandler
        assertGt(SafeSuiteLibV141.SAFE_CompatibilityFallbackHandler_ADDRESS.code.length, 0);
        // safe version matches
        assertEq(factory.safeVersion(), SafeSuiteLibV141.SAFE_VERSION);
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
        emit NewHoprNodeStakeModule(expectedModuleAddress);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress);

        (module, safe) = factory.clone(
            nonce,
            DEFAULT_TARGET,
            admins
        );

        assertEq(module, expectedModuleAddress, "module address mismatch");
        assertEq(safe, expectedSafeAddress, "safe address mismatch");

        _ensureSafeAndModuleAreWired(module, payable(safe), admins);

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
            admins0[i] = vm.addr(300 + i);
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
        emit NewHoprNodeStakeModule(expectedModuleAddress3);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress3);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress4);
        vm.expectEmit(true, false, false, true, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress4);
        // The first two Safe txns were used during the deployment of safe1
        _helperSafeTxnToMultiSend(ISafe(safe1), 400, 0, safeTxData);

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
                safeAddr, multisendAddr, DEFAULT_TARGET
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
                address(1), address(2), DEFAULT_TARGET
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
    function _helperSafeTxnToMultiSend(ISafe safeInstance, uint256 senderPrivateKey, uint256 nonce, bytes memory data) private {
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
            address multiSendAddress
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

    function _helperPredictSafeAndModule(address[] memory admins, address caller, uint256 nonce) private view returns (
        address expectedSafeAddress,
        address expectedModuleAddress
    ) {
        expectedSafeAddress = factory.predictSafeAddress(admins, nonce);
        expectedModuleAddress = factory.predictModuleAddress(caller, nonce, expectedSafeAddress, DEFAULT_TARGET);
    }
}
