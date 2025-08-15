// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Test} from "forge-std/Test.sol";

import {Upgrades} from "openzeppelin-foundry-upgrades/LegacyUpgrades.sol";

import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import {UpgradeableBeacon} from "@openzeppelin/contracts/proxy/beacon/UpgradeableBeacon.sol";
import {BeaconProxy} from "@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol";
import {IBeacon} from "@openzeppelin/contracts/proxy/beacon/IBeacon.sol";

import {Greeter} from "./contracts/Greeter.sol";
import {GreeterProxiable} from "./contracts/GreeterProxiable.sol";
import {GreeterV2} from "./contracts/GreeterV2.sol";
import {GreeterV2Proxiable} from "./contracts/GreeterV2Proxiable.sol";

/**
 * @dev Tests for the Upgrades library in LegacyUpgrades.
 */
contract LegacyUpgradesTest is Test {
    function testUUPS() public {
        vm.startPrank(msg.sender);
        address proxy = address(new ERC1967Proxy(
            address(new GreeterProxiable()),
            abi.encodeWithSelector(GreeterProxiable.initialize.selector, ("hello"))
        ));
        vm.stopPrank();

        Greeter instance = Greeter(proxy);
        address implAddressV1 = Upgrades.getImplementationAddress(proxy);

        assertEq(instance.greeting(), "hello");

        Upgrades.upgradeProxy(
            proxy,
            "GreeterV2Proxiable.sol",
            abi.encodeWithSelector(GreeterV2Proxiable.resetGreeting.selector),
            msg.sender
        );
        address implAddressV2 = Upgrades.getImplementationAddress(proxy);

        assertEq(instance.greeting(), "resetted");
        assertFalse(implAddressV2 == implAddressV1);
    }

    function testTransparent() public {
        vm.startPrank(msg.sender);
        address proxyAdmin = address(new ProxyAdmin());
        address proxy = address(new TransparentUpgradeableProxy(
             address(new Greeter()),
            proxyAdmin,
            abi.encodeWithSelector(Greeter.initialize.selector, ("hello"))
        ));
        vm.stopPrank();

        Greeter instance = Greeter(proxy);
        address implAddressV1 = Upgrades.getImplementationAddress(proxy);
        address adminAddress = Upgrades.getAdminAddress(proxy);

        assertFalse(adminAddress == address(0));

        assertEq(instance.greeting(), "hello");

        Upgrades.upgradeProxy(proxy, "GreeterV2.sol", abi.encodeWithSelector(GreeterV2.resetGreeting.selector), msg.sender);
        address implAddressV2 = Upgrades.getImplementationAddress(proxy);

        assertEq(Upgrades.getAdminAddress(proxy), adminAddress);

        assertEq(instance.greeting(), "resetted");
        assertFalse(implAddressV2 == implAddressV1);
    }

    function testBeacon() public {
        address implAddressV1 = address(new Greeter());

        vm.startPrank(msg.sender);
        address beacon = address(new UpgradeableBeacon(implAddressV1));
        address proxy = address(new BeaconProxy(beacon, abi.encodeWithSelector(Greeter.initialize.selector, ("hello"))));
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
        address proxy = address(new ERC1967Proxy(
            address(new GreeterProxiable()),
            abi.encodeWithSelector(GreeterProxiable.initialize.selector, ("hello"))
        ));
        Upgrades.upgradeProxy(proxy, "GreeterV2Proxiable.sol", abi.encodeWithSelector(GreeterV2Proxiable.resetGreeting.selector));
        vm.stopPrank();
    }

    function testUpgradeBeaconWithoutCaller() public {
        vm.startPrank(msg.sender);
        address beacon = address(new UpgradeableBeacon(address(new Greeter())));
        Upgrades.upgradeBeacon(beacon, "GreeterV2.sol");
        vm.stopPrank();
    }
}
