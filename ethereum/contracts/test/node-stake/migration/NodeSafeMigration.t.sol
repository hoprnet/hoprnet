// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import { Test } from "forge-std/Test.sol";
import { HoprNodeManagementModule } from "../../../src/node-stake/permissioned-module/NodeManagementModule.sol";
import { HoprNodeStakeFactory, HoprNodeStakeFactoryEvents } from "../../../src/node-stake/NodeStakeFactory.sol";
import { HoprNodeSafeMigration, HoprNodeSafeMigrationEvents, IOwner } from "../../../src/node-stake/migration/NodeSafeMigration.sol";
import { Enum, IAvatar } from "../../../src/interfaces/IAvatar.sol";
import { IAvatar } from "../../../src/interfaces/IAvatar.sol";
import { ERC1820RegistryFixtureTest } from "../../utils/ERC1820Registry.sol";
import { SafeSuiteLibV141 } from "../../../src/utils/SafeSuiteLibV141.sol";
import { SafeSuiteLibV150 } from "../../../src/utils/SafeSuiteLibV150.sol";
import { SafeSingletonFixtureTest } from "../../utils/SafeSingleton.sol";
import { HoprToken } from "../../../src/static/HoprToken.sol";

contract NodeSafeMigrationTest is Test, ERC1820RegistryFixtureTest, SafeSingletonFixtureTest, HoprNodeStakeFactoryEvents, HoprNodeSafeMigrationEvents {
    HoprNodeManagementModule public oldModuleSingleton;
    HoprNodeManagementModule public newModuleSingleton;
    HoprNodeSafeMigration public migrationContract;
    HoprNodeStakeFactory public oldFactory;
    HoprNodeStakeFactory public newFactory;
    HoprToken public hoprToken;

    address public constant ADMIN = address(0xBEEF);
    address public constant OLD_ANNOUNCEMENT_ADDRESS = address(0xB0B);
    address public constant NEW_ANNOUNCEMENT_ADDRESS = address(0xB1B);
    address constant OLD_CHANNELS = 0x0101010101010101010101010101010101010101;
    address constant NEW_CHANNELS = 0x0202020202020202020202020202020202020202;
    bytes32 public constant OLD_ANNOUNCEMENT_TARGET =
        bytes32(hex"0000000000000000000000000000000000000B0B010101010101010101010000");
    bytes32 public constant NEW_ANNOUNCEMENT_TARGET =
        bytes32(hex"0000000000000000000000000000000000000B1B010101010101010101010000");
    bytes32 public constant OLD_DEFAULT_TARGET =
        bytes32(hex"0101010101010101010101010101010101010101010101010101010101010101");
    bytes32 public constant NEW_DEFAULT_TARGET =
        bytes32(hex"0202020202020202020202020202020202020202010101010101010101010101");
    bytes32 public constant _IMPLEMENTATION_SLOT = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;


    function setUp() public override(ERC1820RegistryFixtureTest, SafeSingletonFixtureTest) {
        ERC1820RegistryFixtureTest.setUp();
        SafeSingletonFixtureTest.setUp();
        // deploy safe suites
        deployEntireSafeSuite();

        hoprToken = new HoprToken();
        oldModuleSingleton = new HoprNodeManagementModule();
        newModuleSingleton = new HoprNodeManagementModule();
        oldFactory = new HoprNodeStakeFactory(address(oldModuleSingleton), OLD_ANNOUNCEMENT_ADDRESS, ADMIN);
        newFactory = new HoprNodeStakeFactory(address(newModuleSingleton), NEW_ANNOUNCEMENT_ADDRESS, ADMIN);
        migrationContract = new HoprNodeSafeMigration(address(newModuleSingleton), address(newFactory));
    }

    modifier mockTokenChannel() {
        vm.mockCall(OLD_CHANNELS, abi.encodeWithSignature("TOKEN()"), abi.encode(address(hoprToken)));
        vm.mockCall(NEW_CHANNELS, abi.encodeWithSignature("TOKEN()"), abi.encode(address(hoprToken)));
        vm.mockCall(address(hoprToken), abi.encodeWithSignature("approve(address,address)"), abi.encode(bool(true)));
    
        (, uint256 oldDefaultAllowance, bytes32 oldDefaultAnnouncement) = oldFactory.defaultHoprNetwork();
        vm.startPrank(ADMIN);
        oldFactory.updateHoprNetwork(HoprNodeStakeFactory.HoprNetwork({
            tokenAddress: address(hoprToken),
            defaultAnnouncementTarget: oldDefaultAnnouncement,
            defaultTokenAllowance: oldDefaultAllowance
        }));
        (, uint256 newDefaultAllowance, bytes32 newDefaultAnnouncement) = newFactory.defaultHoprNetwork();
        newFactory.updateHoprNetwork(HoprNodeStakeFactory.HoprNetwork({
            tokenAddress: address(hoprToken),
            defaultAnnouncementTarget: newDefaultAnnouncement,
            defaultTokenAllowance: newDefaultAllowance
        }));
        vm.stopPrank();
        _;
    }

    /**
     * @dev preflight check if all the safe contracts are well set
     */
    function test_SafeSuiteSetupForMigration() public view {
        // there's code in Singleton contract
        assertTrue(hasSingletonContract());
        // there's code in ERC1820 contract
        assertTrue(hasErc1820Registry());
        // there's code in Safe Singleton
        assertGt(SafeSuiteLibV141.SAFE_Safe_ADDRESS.code.length, 0);
        // there's code in SafeL2 Singleton v1.4.1
        assertGt(SafeSuiteLibV141.SAFE_SafeL2_ADDRESS.code.length, 0);
        // there's code in SafeL2 Singleton v1.5.0
        assertGt(SafeSuiteLibV150.SAFE_SafeL2_ADDRESS.code.length, 0);
        // there's code in Safe Proxy Factory
        assertGt(SafeSuiteLibV141.SAFE_SafeProxyFactory_ADDRESS.code.length, 0);
        // there's code in Safe MultiSendCallOnly
        assertGt(SafeSuiteLibV141.SAFE_MultiSendCallOnly_ADDRESS.code.length, 0);
        // there's code in Safe ExtensibleFallbackHandler, v1.5.0
        assertGt(SafeSuiteLibV150.SAFE_ExtensibleFallbackHandler_ADDRESS.code.length, 0);
        // safe version matches
        assertEq(oldFactory.safeVersion(), SafeSuiteLibV141.SAFE_VERSION);
        assertEq(newFactory.safeVersion(), SafeSuiteLibV141.SAFE_VERSION);
    }

    /**
     * @dev test migrate the implementaiton of a module proxy
     */
    function test_MigrateModuleSingleton() public mockTokenChannel {
        uint256 nonce = 0x01D;
        uint256 callerPrivateKey = 0xca11e2;
        address caller = vm.addr(callerPrivateKey);
        address[] memory admins = new address[](1);
        admins[0] = caller;
        // use the old factory to deploy a module with an outdated implementation
        vm.prank(caller);
        (address oldModuleProxy, address payable safeAddress) = oldFactory.clone(
            nonce,
            OLD_DEFAULT_TARGET,
            admins
        );
        assertEq(vm.load(address(oldModuleProxy), _IMPLEMENTATION_SLOT), bytes32(uint256(uint160(address(oldModuleSingleton)))));

        // migrate the module implementation to a new one, using delegate call from the safe
        bytes memory data = abi.encodeWithSelector(
            migrationContract.migrateModuleSingleton.selector,
            oldModuleProxy,
            hex""
        );

        vm.prank(caller);
        // vm.expectEmit(false, false, false, true, safeAddress);
        // emit ChangedModuleImplementation(
        //     oldModuleProxy
        // );
        _helperSafeTxnDelegateCall(address(migrationContract), IAvatar(safeAddress), callerPrivateKey, data);

        // verify the current implementation address is indeed the latest one
        assertEq(vm.load(address(oldModuleProxy), _IMPLEMENTATION_SLOT), bytes32(uint256(uint160(address(newModuleSingleton)))));
        vm.clearMockedCalls();
    }

    function test_MigrateNodeSafe() public mockTokenChannel {
        uint256 callerPrivateKey = 0xCAFE;
        address caller = vm.addr(callerPrivateKey);
        address[] memory admins = new address[](1);
        admins[0] = caller;
        address[] memory nodes = new address[](3);
        nodes[0] = address(0xCAB1);
        nodes[1] = address(0xCAB2);
        nodes[2] = address(0xCAB3);
        uint256 nonce = 999;
        uint256 newNonce = nonce + 1;

        // create a safe via the old factory
        vm.prank(caller);
        (address oldModuleProxy, address payable safeAddress) = oldFactory.clone(
            nonce,
            OLD_DEFAULT_TARGET,
            admins
        );
        // check the safe is created with the old module
        assertTrue(IAvatar(safeAddress).isModuleEnabled(address(oldModuleProxy)));
        // module proxy uses old implementation, at the implementation slot
        assertEq(vm.load(address(oldModuleProxy), _IMPLEMENTATION_SLOT), bytes32(uint256(uint160(address(oldModuleSingleton)))));
        // check the safe is owned by the old factory
        assertEq(IAvatar(safeAddress).getOwners().length, 1);
        assertEq(IAvatar(safeAddress).getOwners()[0], address(caller));
        // check the module is owned by the safe
        assertEq(IOwner(address(oldModuleProxy)).owner(), safeAddress);

        // prepare safe transaction data for migration. Caller should call 
        // migrateSafeV141ToL2AndMigrateToUpgradeableModule(address,bytes32,uint256,address,address[] memory)
        // on the migration contract via delegatecall from the safe
        bytes memory data = abi.encodeWithSelector(
            migrationContract.migrateSafeV141ToL2AndMigrateToUpgradeableModule.selector,
            oldModuleProxy,
            NEW_DEFAULT_TARGET,
            newNonce,
            nodes
        );
        // predict the new module deployment address
        address newModuleProxyPrediction = newFactory.predictModuleAddress(safeAddress, newNonce, safeAddress, NEW_DEFAULT_TARGET);

        // migrate the module singleton to the new version via delegatecall from the safe, using delegatecall
        vm.prank(caller);
        // vm.expectEmit(false, false, false, true, safeAddress);
        // emit SafeAndModuleMigrationCompleted(
        //     safeAddress,
        //     oldModuleProxy,
        //     newModuleProxyPrediction
        // );
        _helperSafeTxnDelegateCall(address(migrationContract), IAvatar(safeAddress), callerPrivateKey, data);

        // check the module is now upgraded to the new singleton
        assertEq(IOwner(address(newModuleProxyPrediction)).owner(), safeAddress); // new module is now owned by the safe
        assertTrue(IAvatar(safeAddress).isModuleEnabled(address(newModuleProxyPrediction))); // new module is enabled in the safe
        assertFalse(IAvatar(safeAddress).isModuleEnabled(address(oldModuleProxy))); // old module is no longer enabled in the safe
        assertEq(vm.load(address(newModuleProxyPrediction), _IMPLEMENTATION_SLOT), bytes32(uint256(uint160(address(newModuleSingleton)))));

        vm.clearMockedCalls();
    }

    function _helperSafeTxnDelegateCall(address to, IAvatar safeInstance, uint256 senderPrivateKey, bytes memory data) private {
        address sender = vm.addr(senderPrivateKey);
        uint256 safeNonce = safeInstance.nonce();
        bytes32 dataHash =
            safeInstance.getTransactionHash(to, 0, data, Enum.Operation.DelegateCall, 0, 0, 0, address(0), sender, safeNonce);

        // sign dataHash
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(senderPrivateKey, dataHash);
        safeInstance.execTransaction(
            to,                     // to
            0,                      // value
            data,                   // data 
            Enum.Operation.DelegateCall, // operation
            0,                      // safeTxGas
            0,                      // baseGas
            0,                      // gasPrice
            address(0),             // gasToken
            payable(sender),        // refundReceiver
            abi.encodePacked(r, s, v) // signatures
        );
    }
}