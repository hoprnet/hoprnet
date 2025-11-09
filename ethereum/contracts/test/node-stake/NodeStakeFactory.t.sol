// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Test.sol";

import { HoprNodeManagementModule } from "../../src/node-stake/permissioned-module/NodeManagementModule.sol";
import { HoprCapabilityPermissions } from "../../src/node-stake/permissioned-module/CapabilityPermissions.sol";
import { HoprNodeStakeFactory, HoprNodeStakeFactoryEvents } from "../../src/node-stake/NodeStakeFactory.sol";
import { Safe } from "safe-contracts/Safe.sol";
import { SafeSuiteLib } from "../../src/utils/SafeSuiteLib.sol";
import { SafeSingletonFixtureTest } from "../utils/SafeSingleton.sol";
import { ClonesUpgradeable } from "openzeppelin-contracts-upgradeable/proxy/ClonesUpgradeable.sol";

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
        factory = new HoprNodeStakeFactory();
    }

    /**
     * @dev preflight check if all the safe contracts are well set
     */
    function test_SafeSuiteSetup() public {
        // there's code in Singleton contract
        assertTrue(hasSingletonContract());
        // there's code in Safe Singleton
        assertGt(SafeSuiteLib.SAFE_Safe_ADDRESS.code.length, 0);
        // there's code in Safe Proxy Factory
        assertGt(SafeSuiteLib.SAFE_SafeProxyFactory_ADDRESS.code.length, 0);
        // there's code in Safe MultiSendCallOnly
        assertGt(SafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS.code.length, 0);
        // there's code in Safe CompatibilityFallbackHandler
        assertGt(SafeSuiteLib.SAFE_CompatibilityFallbackHandler_ADDRESS.code.length, 0);
        // safe version matches
        assertEq(factory.safeVersion(), SafeSuiteLib.SAFE_VERSION);
    }

    /**
     * @dev Fail to clone a safe when there's not event one admin
     */
    function testRevert_CloneSafeAndModuleWithFewOwner() public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));

        uint256 nonce = 0;
        address[] memory admins = new address[](0);

        vm.prank(caller);
        vm.expectRevert(HoprNodeStakeFactory.TooFewOwners.selector);
        (module, safe) = factory.clone(
            address(moduleSingleton),
            admins,
            nonce,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101")
        );
        vm.clearMockedCalls();
    }

    /**
     * @dev Clone a safe and a module and they are wired
     */
    function test_CloneSafeAndModule() public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));

        uint256 nonce = 0;
        address expectedModuleAddress =
            factory.predictDeterministicAddress(address(moduleSingleton), keccak256(abi.encodePacked(caller, nonce)));

        vm.startPrank(caller);
        vm.expectEmit(true, true, false, false, address(factory));
        emit NewHoprNodeStakeModule(address(moduleSingleton), expectedModuleAddress);
        address[] memory admins = new address[](10);
        for (uint256 i = 0; i < admins.length; i++) {
            admins[i] = vm.addr(200 + i);
        }
        (module, safe) = factory.clone(
            address(moduleSingleton),
            admins,
            nonce,
            bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101")
        );

        // Safe should have module enabled
        assertTrue(Safe(safe).isModuleEnabled(module));
        // Safe should have 1 threshold and admin as the only owner
        assertEq(Safe(safe).getThreshold(), 1, "Wrong threshold");
        address[] memory owners = Safe(safe).getOwners();
        assertEq(owners.length, admins.length, "Wrong number of owners");
        for (uint256 j = 0; j < admins.length; j++) {
            assertTrue(Safe(safe).isOwner(admins[j]));
        }
        assertFalse(Safe(safe).isOwner(address(factory)));
        // module owner should be safe
        assertEq(HoprNodeManagementModule(module).owner(), safe, "Wrong module owner");
        // module multisend should beSafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS
        assertEq(
            HoprNodeManagementModule(module).multisend(),
            SafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS,
            "Wrong module owner"
        );

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    function testFuzz_InitializeModuleProxy(uint256 nonce, address safeAddr, address multisendAddr) public {
        address channels = 0x0101010101010101010101010101010101010101;
        address token = 0x1010101010101010101010101010101010101010;
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));
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
        vm.mockCall(channels, abi.encodeWithSignature("token()"), abi.encode(token));

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
}
