// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.8.0 <0.9.0;

import "forge-std/Test.sol";

import { HoprNodeManagementModule } from "../../src/node-stake/permissioned-module/NodeManagementModule.sol";
import { HoprCapabilityPermissions } from "../../src/node-stake/permissioned-module/CapabilityPermissions.sol";
import { HoprNodeStakeFactory, HoprNodeStakeFactoryEvents } from "../../src/node-stake/NodeStakeFactory.sol";
import { Safe } from "safe-contracts-1.4.1/Safe.sol";
import { SafeSuiteLibV141 } from "../../src/utils/SafeSuiteLibV141.sol";
import { SafeSingletonFixtureTest } from "../utils/SafeSingleton.sol";
import { ClonesUpgradeable } from "openzeppelin-contracts-upgradeable-4.9.2/proxy/ClonesUpgradeable.sol";

contract NodeSafeMigrationTest is Test, SafeSingletonFixtureTest, HoprNodeStakeFactoryEvents {
    using ClonesUpgradeable for address;

    // HoprNodeManagementModule public moduleSingleton;
    // HoprNodeStakeFactory public factory;
    // address public caller;
    // address public admin;
    // address public module;
    // address payable public safe;

    // /**
    //  * Manually import events and errors
    //  */
    // event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    // event SetMultisendAddress(address indexed multisendAddress);

    function setUp() public override(SafeSingletonFixtureTest) {
        super.setUp();
        // deploy safe suites
        deployEntireSafeSuite();

        // caller = vm.addr(101); // make make address(101) a caller
        // admin = vm.addr(102); // make make address(102) an admin
        // moduleSingleton = new HoprNodeManagementModule();
        // factory = new HoprNodeStakeFactory();
    }
}