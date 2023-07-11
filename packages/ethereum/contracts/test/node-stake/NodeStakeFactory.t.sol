// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import '../../src/node-stake/permissioned-module/NodeManagementModule.sol';
import '../../src/node-stake/permissioned-module/CapabilityPermissions.sol';
import '../../src/node-stake/NodeStakeFactory.sol';
import '../../lib/safe-contracts/contracts/Safe.sol';
import "../../script/utils/SafeSuiteLib.sol";
import "../utils/SafeSingleton.sol";
import 'forge-std/Test.sol';
import "@openzeppelin/contracts-upgradeable/proxy/ClonesUpgradeable.sol";

contract HoprNodeManagementModuleTest is Test, SafeSingletonFixtureTest {
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
    event NewHoprNodeStakeModule(address instance);
    event NewHoprNodeStakeSafe(address instance);
    event AvatarSet(address indexed previousAvatar, address indexed newAvatar);

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
    }

    /**
    * @dev Clone a safe and a module and they are wired
    */
    function test_CloneSafeAndModule() public {
        uint256 nonce = 0;
        address expectedModuleAddress = factory.predictDeterministicAddress(address(moduleSingleton), keccak256(abi.encodePacked(caller, nonce)));
        
        vm.startPrank(caller);
        vm.expectEmit(true, false, false, false, address(factory));
        emit NewHoprNodeStakeModule(expectedModuleAddress);
        (module, safe) = factory.clone(address(moduleSingleton), admin, nonce);

        // Safe should have module enabled
        assertTrue(Safe(safe).isModuleEnabled(module));
        // Safe should have 1 threshold and admin as the only owner
        assertEq(Safe(safe).getThreshold(), 1, "Wrong threshold");
        address[] memory owners = Safe(safe).getOwners();
        assertEq(owners.length, 1, "Wrong number of owners");
        assertEq(owners[0], admin, "Wrong admin");
        // module should have safe as avatar
        assertEq(HoprNodeManagementModule(module).avatar(), safe, "Wrong avatar");
        // module owner should be safe
        assertEq(HoprNodeManagementModule(module).owner(), safe, "Wrong module owner");
        // module multisend should beSafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS
        assertEq(HoprNodeManagementModule(module).multisend(), SafeSuiteLib.SAFE_MultiSendCallOnly_ADDRESS, "Wrong module owner");

        vm.stopPrank();
    }

    function testFuzz_InitializeModuleProxy(uint256 nonce, address safeAddr, address multisendAddr) public {
        vm.assume(safeAddr != address(0));
        vm.assume(multisendAddr != address(0));
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);
        // add Safe and multisend to the module
        bytes memory moduleInitializer = abi.encodeWithSignature("initialize(bytes)", abi.encode(safeAddr, multisendAddr));

        vm.expectEmit(true, true, false, false, address(moduleProxy));
        emit OwnershipTransferred(address(0), safeAddr);
        vm.expectEmit(true, true, false, false, address(moduleProxy));
        emit AvatarSet(address(0), safeAddr);
        vm.expectEmit(true, false, false, false, address(moduleProxy));
        emit SetMultisendAddress(multisendAddr);
        moduleProxy.call(moduleInitializer);
    }

    function testRevert_CloneButFailToInitializeWithAddressZero(uint256 nonce, address safeAddr, address multisendAddr) public {
        vm.assume(safeAddr == address(0) || multisendAddr == address(0));
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);

        // initialize module proxy with invalid variables
        vm.expectRevert(HoprCapabilityPermissions.AddressIsZero.selector);
        // add Safe and multisend to the module
        bytes memory moduleInitializer = abi.encodeWithSignature("initialize(bytes)", abi.encode(safeAddr, multisendAddr));
        moduleProxy.call(moduleInitializer);
    }

    function testRevert_CloneButFailToInitializeTwice(uint256 nonce, address safeAddr, address multisendAddr) public {
        vm.assume(safeAddr != address(0) && multisendAddr != address(0));
        bytes32 salt = keccak256(abi.encodePacked(msg.sender, nonce));
        // 1. Deploy node management module
        address moduleProxy = address(moduleSingleton).cloneDeterministic(salt);

        // initialize module proxy with valid variables
        bytes memory moduleInitializer = abi.encodeWithSignature("initialize(bytes)", abi.encode(address(1), address(2)));
        moduleProxy.call(moduleInitializer);
        // re-initialize module proxy with variables
        bytes memory moduleReinitializer = abi.encodeWithSignature("initialize(bytes)", abi.encode(safeAddr, multisendAddr));
        vm.expectRevert(AlreadyInitialized.selector);
        moduleProxy.call(moduleReinitializer);
    }
}
