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

contract MaliciousModuleMock {
    function initialize(bytes calldata) public {
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
    function testRevert_CloneSafeAndModuleWithFewOwner() public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));

        uint256 nonce = 0;
        address[] memory admins = new address[](0);

        vm.prank(caller);
        vm.expectRevert(HoprNodeStakeFactory.TooFewOwners.selector);
        (module, safe) = factory.clone(
            nonce,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"),
            admins
        );
        vm.clearMockedCalls();
    }

    function testRevert_CloneSafeAndModuleWithStakeFactoryAsOwner() public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));

        uint256 nonce = 0;
        address[] memory admins = new address[](2);
        admins[0] = vm.addr(103); // add another admin
        admins[1] = address(factory); // add factory address as an admin

        vm.prank(caller);
        vm.expectRevert(HoprNodeStakeFactory.InvalidOwner.selector);
        (module, safe) = factory.clone(
            nonce,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"),
            admins
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev Clone a safe and a module and they are wired
     */
    function test_CloneSafeAndModule() public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));

        uint256 nonce = 0;
        address[] memory admins = new address[](10);
        for (uint256 i = 0; i < admins.length; i++) {
            admins[i] = vm.addr(200 + i);
        }
        address expectedModuleAddress =
            factory.predictModuleAddress(keccak256(abi.encodePacked(caller, nonce)));
        address expectedSafeAddress = factory.predictSafeAddress(admins, nonce);

        vm.startPrank(caller);
        vm.expectEmit(true, true, false, false, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress);
        vm.expectEmit(true, true, false, false, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress);

        (module, safe) = factory.clone(
            nonce,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"),
            admins
        );

        _ensureSafeAndModuleAreWired(module, payable(safe), admins);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Clone multiple safes and modules to ensure the nonce and salt works
     *      and the deployed addresses are unique
     */
    function test_CloneMultipleSafesAndModules() public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));

        // Deploy first safe and module, with 3 admins
        uint256 nonce0 = 0;
        address[] memory admins0 = new address[](3);
        for (uint256 i = 0; i < admins0.length; i++) {
            admins0[i] = vm.addr(300 + i);
        }
        (address module0, address payable safe0) = factory.clone(
            nonce0,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"),
            admins0
        );
        _ensureSafeAndModuleAreWired(module0, payable(safe0), admins0);

        // Deploy second safe and module, with 3 admins
        uint256 nonce1 = 1;
        address[] memory admins1 = new address[](1);
        admins1[0] = vm.addr(400);
        (address module1, address payable safe1) = factory.clone(
            nonce1,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"),
            admins1
        );
        _ensureSafeAndModuleAreWired(module1, payable(safe1), admins1);

        // Test with multisend to create two safe-module pairs, from the second safe that's just deployed
        vm.startPrank(admins1[0]);
        uint256 nonce3 = 0;
        address[] memory admins3 = new address[](3);
        for (uint256 i = 0; i < admins3.length; i++) {
            admins3[i] = vm.addr(500 + i);
        }
        uint256 nonce4 = 1;
        address[] memory admins4 = new address[](2);
        for (uint256 i = 0; i < admins4.length; i++) {
            admins4[i] = vm.addr(600 + i);
        }
        bytes[] memory data = new bytes[](2);
        data[0] = abi.encodeWithSelector(HoprNodeStakeFactory.clone.selector, nonce3, bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"), admins3); // approve on token
        data[1] = abi.encodeWithSelector(HoprNodeStakeFactory.clone.selector, nonce4, bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"), admins4); // approve on token
        uint256[] memory dataLengths = new uint256[](2);
        dataLengths[0] = data[0].length;
        dataLengths[1] = data[1].length;

        // safe1 uses multisend to deploy another safe and module
        bytes memory safeTxData = _helperBuildMultiSendTxForSafeModuleClone(address(factory), dataLengths, data);
        address expectedModuleAddress3 =
            factory.predictModuleAddress(keccak256(abi.encodePacked(address(safe1), nonce3)));
        address expectedSafeAddress3 = factory.predictSafeAddress(admins3, nonce3);
        address expectedModuleAddress4 =
            factory.predictModuleAddress(keccak256(abi.encodePacked(address(safe1), nonce4)));
        address expectedSafeAddress4 = factory.predictSafeAddress(admins4, nonce4);

        vm.expectEmit(true, true, false, false, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress3);
        vm.expectEmit(true, true, false, false, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress3);
        vm.expectEmit(true, true, false, false, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress4);
        vm.expectEmit(true, true, false, false, address(factory));
        emit NewHoprNodeStakeSafe(expectedSafeAddress4);
        // The first two Safe txns were used during the deployment of safe1
        _helperSafeTxnToMultiSend(ISafe(safe1), 400, 2, safeTxData);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testFuzz_InitializeModuleProxy(uint256 nonce, address safeAddr, address multisendAddr) public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));
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
                safeAddr, multisendAddr, bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101")
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

    function testRevert_CloneButFailToInitializeTwice(uint256 nonce, address safeAddr, address multisendAddr) public {
        vm.assume(safeAddr != address(0) && multisendAddr != address(0));

        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));

        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);

        // initialize module proxy with valid variables
        bytes memory moduleInitializer = abi.encodeWithSignature(
            "initialize(bytes)",
            abi.encode(
                address(1), address(2), bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101")
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

    function testRevert_SafeSetupReuseNonce() public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("TOKEN()"), abi.encode(token));
        
        uint256 nonce = 0;
        address[] memory admins = new address[](10);
        for (uint256 i = 0; i < admins.length; i++) {
            admins[i] = vm.addr(200 + i);
        }

        vm.startPrank(admins[0]);
        factory.clone(
            nonce,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"),
            admins
        );

        vm.expectRevert(bytes("ERC1167: create2 failed"));
        factory.clone(
            nonce,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101"),
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
}
