// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Script} from "forge-std/Script.sol";

import {Greeter} from "./contracts/Greeter.sol";
import {GreeterProxiable} from "./contracts/GreeterProxiable.sol";
import {GreeterV2} from "./contracts/GreeterV2.sol";
import {GreeterV2Proxiable} from "./contracts/GreeterV2Proxiable.sol";

import {Upgrades, Options} from "openzeppelin-foundry-upgrades/LegacyUpgrades.sol";

import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";
import {UpgradeableBeacon} from "@openzeppelin/contracts/proxy/beacon/UpgradeableBeacon.sol";
import {BeaconProxy} from "@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol";

/**
 * @dev Sample script to upgrade OpenZeppelin Contracts v4 deployments using transparent, UUPS, and beacon proxies.
 */
contract LegacyUpgradesScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        // deploy initial implementations for testing only
        address greeter = address(new Greeter());
        address greeterProxiable = address(new GreeterProxiable());

        // deploy each type of proxy for testing only
        address proxyAdmin = address(new ProxyAdmin());
        address transparentProxy = address(new TransparentUpgradeableProxy(greeter, proxyAdmin, abi.encodeWithSelector(Greeter.initialize.selector, ("hello"))));

        address uupsProxy = address(new ERC1967Proxy(
            greeterProxiable,
            abi.encodeWithSelector(GreeterProxiable.initialize.selector, ("hello"))
        ));

        address beacon = address(new UpgradeableBeacon(greeter));
        new BeaconProxy(beacon, abi.encodeWithSelector(Greeter.initialize.selector, ("hello")));

        // example upgrade of an existing transparent proxy
        Upgrades.upgradeProxy(transparentProxy, "GreeterV2.sol", abi.encodeWithSelector(GreeterV2.resetGreeting.selector));

        // example upgrade of an existing UUPS proxy
        Upgrades.upgradeProxy(
            uupsProxy,
            "GreeterV2Proxiable.sol",
            abi.encodeWithSelector(GreeterV2Proxiable.resetGreeting.selector)
        );

        // example upgrade of an existing beacon
        Upgrades.upgradeBeacon(beacon, "GreeterV2.sol");

        vm.stopBroadcast();
    }
}
