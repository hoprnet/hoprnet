// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {Upgrades, Options} from "openzeppelin-foundry-upgrades/Upgrades.sol";

import {IBeacon} from "@openzeppelin/contracts/proxy/beacon/IBeacon.sol";

import {Greeter} from "../../openzeppelin-contracts-v4/test/contracts/Greeter.sol";
import {GreeterProxiable} from "../../openzeppelin-contracts-v4/test/contracts/GreeterProxiable.sol";
import {GreeterV2} from "../../openzeppelin-contracts-v4/test/contracts/GreeterV2.sol";
import {GreeterV2Proxiable} from "../../openzeppelin-contracts-v4/test/contracts/GreeterV2Proxiable.sol";

/**
 * @dev Tests for the Upgrades library.
 */
contract UpgradesTest is Test {
    function testUUPS() public {
        vm.startPrank(msg.sender);
        address proxy = Upgrades.deployUUPSProxy(
            "GreeterProxiable.sol",
            abi.encodeCall(Greeter.initialize, ("hello"))
        );
        vm.stopPrank();

        Greeter instance = Greeter(proxy);
        address implAddressV1 = Upgrades.getImplementationAddress(proxy);

        assertEq(instance.greeting(), "hello");

        Upgrades.upgradeProxy(
            proxy,
            "GreeterV2Proxiable.sol",
            abi.encodeCall(GreeterV2Proxiable.resetGreeting, ()),
            msg.sender
        );
        address implAddressV2 = Upgrades.getImplementationAddress(proxy);

        assertEq(instance.greeting(), "resetted");
        assertFalse(implAddressV2 == implAddressV1);
    }

    function testTransparent() public {
        vm.startPrank(msg.sender);
        address proxy = Upgrades.deployTransparentProxy(
            "Greeter.sol",
            msg.sender,
            abi.encodeCall(Greeter.initialize, ("hello"))
        );
        vm.stopPrank();

        Greeter instance = Greeter(proxy);
        address implAddressV1 = Upgrades.getImplementationAddress(proxy);
        address adminAddress = Upgrades.getAdminAddress(proxy);

        assertFalse(adminAddress == address(0));

        assertEq(instance.greeting(), "hello");

        Upgrades.upgradeProxy(proxy, "GreeterV2.sol", abi.encodeCall(GreeterV2.resetGreeting, ()), msg.sender);
        address implAddressV2 = Upgrades.getImplementationAddress(proxy);

        assertEq(Upgrades.getAdminAddress(proxy), adminAddress);

        assertEq(instance.greeting(), "resetted");
        assertFalse(implAddressV2 == implAddressV1);
    }

    function testBeacon() public {
        address beacon = Upgrades.deployBeacon("Greeter.sol", msg.sender);
        address implAddressV1 = IBeacon(beacon).implementation();

        vm.startPrank(msg.sender);
        address proxy = Upgrades.deployBeaconProxy(beacon, abi.encodeCall(Greeter.initialize, ("hello")));
        vm.stopPrank();

        Greeter instance = Greeter(proxy);

        assertEq(Upgrades.getBeaconAddress(proxy), beacon);

        assertEq(instance.greeting(), "hello");

        Upgrades.upgradeBeacon(beacon, "GreeterV2.sol", msg.sender);
        address implAddressV2 = IBeacon(beacon).implementation();

        vm.prank(msg.sender);
        GreeterV2(address(instance)).setGreeting("modified");

        assertEq(instance.greeting(), "modified");
        assertFalse(implAddressV2 == implAddressV1);
    }

    function testUpgradeProxyWithoutCaller() public {
        vm.startPrank(msg.sender);
        address proxy = Upgrades.deployUUPSProxy(
            "GreeterProxiable.sol",
            abi.encodeCall(GreeterProxiable.initialize, ("hello"))
        );
        Upgrades.upgradeProxy(proxy, "GreeterV2Proxiable.sol", abi.encodeCall(GreeterV2Proxiable.resetGreeting, ()));
        vm.stopPrank();
    }

    function testUpgradeBeaconWithoutCaller() public {
        address beacon = Upgrades.deployBeacon("Greeter.sol", msg.sender);

        vm.startPrank(msg.sender);
        Upgrades.upgradeBeacon(beacon, "GreeterV2.sol");
        vm.stopPrank();
    }
}
