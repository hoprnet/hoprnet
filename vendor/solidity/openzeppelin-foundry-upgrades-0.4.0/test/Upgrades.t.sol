// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {Upgrades, Options} from "openzeppelin-foundry-upgrades/Upgrades.sol";

import {StringFinder} from "openzeppelin-foundry-upgrades/internal/StringFinder.sol";

import {IBeacon} from "@openzeppelin/contracts/proxy/beacon/IBeacon.sol";
import {ProxyAdmin} from "@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol";
import {ITransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

import {Greeter} from "./contracts/Greeter.sol";
import {GreeterProxiable} from "./contracts/GreeterProxiable.sol";
import {GreeterV2} from "./contracts/GreeterV2.sol";
import {GreeterV2Proxiable} from "./contracts/GreeterV2Proxiable.sol";
import {WithConstructor, NoInitializer} from "./contracts/WithConstructor.sol";
import {HasOwner} from "./contracts/HasOwner.sol";

// Import additional contracts to include them for compilation
import "./contracts/Validations.sol";

/**
 * @dev Tests for the Upgrades library.
 */
contract UpgradesTest is Test {
    using StringFinder for string;

    function testUUPS() public {
        address proxy = Upgrades.deployUUPSProxy(
            "GreeterProxiable.sol",
            abi.encodeCall(Greeter.initialize, (msg.sender, "hello"))
        );
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

    function testUUPS_upgradeWithoutData() public {
        address proxy = Upgrades.deployUUPSProxy(
            "GreeterProxiable.sol",
            abi.encodeCall(Greeter.initialize, (msg.sender, "hello"))
        );
        address implAddressV1 = Upgrades.getImplementationAddress(proxy);

        Upgrades.upgradeProxy(proxy, "GreeterV2Proxiable.sol", "", msg.sender);
        address implAddressV2 = Upgrades.getImplementationAddress(proxy);

        assertFalse(implAddressV2 == implAddressV1);
    }

    function testTransparent() public {
        address proxy = Upgrades.deployTransparentProxy(
            "Greeter.sol",
            msg.sender,
            abi.encodeCall(Greeter.initialize, (msg.sender, "hello"))
        );
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

    function testTransparent_upgradeWithoutData() public {
        address proxy = Upgrades.deployTransparentProxy(
            "Greeter.sol",
            msg.sender,
            abi.encodeCall(Greeter.initialize, (msg.sender, "hello"))
        );
        address implAddressV1 = Upgrades.getImplementationAddress(proxy);
        address adminAddress = Upgrades.getAdminAddress(proxy);

        assertFalse(adminAddress == address(0));

        Upgrades.upgradeProxy(proxy, "GreeterV2.sol", "", msg.sender);
        address implAddressV2 = Upgrades.getImplementationAddress(proxy);

        assertEq(Upgrades.getAdminAddress(proxy), adminAddress);

        assertFalse(implAddressV2 == implAddressV1);
    }

    function testBeacon() public {
        address beacon = Upgrades.deployBeacon("Greeter.sol", msg.sender);
        address implAddressV1 = IBeacon(beacon).implementation();

        address proxy = Upgrades.deployBeaconProxy(beacon, abi.encodeCall(Greeter.initialize, (msg.sender, "hello")));
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
        address proxy = Upgrades.deployUUPSProxy(
            "GreeterProxiable.sol",
            abi.encodeCall(GreeterProxiable.initialize, (msg.sender, "hello"))
        );

        vm.startPrank(msg.sender);
        Upgrades.upgradeProxy(proxy, "GreeterV2Proxiable.sol", abi.encodeCall(GreeterV2Proxiable.resetGreeting, ()));
        vm.stopPrank();
    }

    function testUpgradeBeaconWithoutCaller() public {
        address beacon = Upgrades.deployBeacon("Greeter.sol", msg.sender);

        vm.startPrank(msg.sender);
        Upgrades.upgradeBeacon(beacon, "GreeterV2.sol");
        vm.stopPrank();
    }

    function testValidateImplementation() public {
        Options memory opts;
        Invoker i = new Invoker();
        try i.validateImplementation("Validations.sol:Unsafe", opts) {
            fail();
        } catch {
            // TODO: check error message
        }
    }

    function testValidateLayout() public {
        Options memory opts;
        opts.referenceContract = "Validations.sol:LayoutV1";
        Invoker i = new Invoker();
        try i.validateUpgrade("Validations.sol:LayoutV2_Bad", opts) {
            fail();
        } catch {
            // TODO: check error message
        }
    }

    function testValidateLayoutUpgradesFrom() public {
        Options memory opts;
        Invoker i = new Invoker();
        try i.validateUpgrade("Validations.sol:LayoutV2_UpgradesFrom_Bad", opts) {
            fail();
        } catch {
            // TODO: check error message
        }
    }

    function testValidateNamespaced() public {
        Options memory opts;
        opts.referenceContract = "Validations.sol:NamespacedV1";
        Invoker i = new Invoker();
        try i.validateUpgrade("Validations.sol:NamespacedV2_Bad", opts) {
            fail();
        } catch {
            // TODO: check error message
        }
    }

    function testValidateNamespacedUpgradesFrom() public {
        Options memory opts;
        Invoker i = new Invoker();
        try i.validateUpgrade("Validations.sol:NamespacedV2_UpgradesFrom_Bad", opts) {
            fail();
        } catch {
            // TODO: check error message
        }
    }

    function testValidateNamespacedOk() public {
        Options memory opts;
        opts.referenceContract = "Validations.sol:NamespacedV1";
        Upgrades.validateUpgrade("Validations.sol:NamespacedV2_Ok", opts);
    }

    function testValidateNamespacedUpgradesFromOk() public {
        Options memory opts;
        Upgrades.validateUpgrade("Validations.sol:NamespacedV2_UpgradesFrom_Ok", opts);
    }

    function testValidateNamespacedNoReference() public {
        Options memory opts;
        Invoker i = new Invoker();
        // validate upgrade without reference contract - an error is expected from upgrades-core CLI
        try i.validateUpgrade("Validations.sol:NamespacedV2_Ok", opts) {
            fail();
        } catch {
            // TODO: check error message
        }
    }

    function testUnsafeSkipAllChecks() public {
        Options memory opts;
        opts.unsafeSkipAllChecks = true;
        Upgrades.validateImplementation("Validations.sol:Unsafe", opts);
    }

    function testUnsafeSkipStorageCheck() public {
        Options memory opts;
        opts.unsafeSkipStorageCheck = true;
        Upgrades.validateUpgrade("Validations.sol:NamespacedV2_UpgradesFrom_Bad", opts);
    }

    function testUnsafeAllow() public {
        Options memory opts;
        opts.unsafeAllow = "delegatecall,selfdestruct";
        Upgrades.validateImplementation("Validations.sol:Unsafe", opts);
    }

    function testUnsafeAllowRenames() public {
        Options memory opts;
        opts.unsafeAllowRenames = true;
        Upgrades.validateImplementation("Validations.sol:LayoutV2_Renamed", opts);
    }

    function testSkipStorageCheckNoReference() public {
        Options memory opts;
        opts.unsafeSkipStorageCheck = true;
        Upgrades.validateUpgrade("Validations.sol:NamespacedV2_Ok", opts);
    }

    function testWithConstructor() public {
        Options memory opts;
        opts.constructorData = abi.encode(123);
        address proxy = Upgrades.deployTransparentProxy(
            "WithConstructor.sol:WithConstructor",
            msg.sender,
            abi.encodeCall(WithConstructor.initialize, (456)),
            opts
        );
        assertEq(WithConstructor(proxy).a(), 123);
        assertEq(WithConstructor(proxy).b(), 456);
    }

    function testNoInitializer() public {
        Options memory opts;
        opts.constructorData = abi.encode(123);
        address proxy = Upgrades.deployTransparentProxy("WithConstructor.sol:NoInitializer", msg.sender, "", opts);
        assertEq(WithConstructor(proxy).a(), 123);
    }

    function testProxyAdminCheck() public {
        ProxyAdmin admin = new ProxyAdmin(msg.sender);

        Invoker i = new Invoker();
        try
            i.deployTransparentProxy(
                "Greeter.sol",
                address(admin), // NOT SAFE
                abi.encodeCall(Greeter.initialize, (msg.sender, "hello"))
            )
        {
            fail();
        } catch Error(string memory reason) {
            assertTrue(reason.contains("`initialOwner` must not be a ProxyAdmin contract."));
            assertTrue(reason.contains(vm.toString(address(admin))));
        }
    }

    function testProxyAdminCheck_emptyOpts() public {
        HasOwner hasOwner = new HasOwner(msg.sender);
        Options memory opts;

        Invoker i = new Invoker();
        try
            i.deployTransparentProxy(
                "Greeter.sol",
                address(hasOwner), // false positive
                abi.encodeCall(Greeter.initialize, (msg.sender, "hello")),
                opts
            )
        {
            fail();
        } catch Error(string memory reason) {
            assertTrue(reason.contains("`initialOwner` must not be a ProxyAdmin contract."));
            assertTrue(reason.contains(vm.toString(address(hasOwner))));
        }
    }

    function testProxyAdminCheck_skip() public {
        HasOwner hasOwner = new HasOwner(msg.sender);
        Options memory opts;
        opts.unsafeSkipProxyAdminCheck = true;

        Upgrades.deployTransparentProxy(
            "Greeter.sol",
            address(hasOwner),
            abi.encodeCall(Greeter.initialize, (msg.sender, "hello")),
            opts
        );
    }

    function testProxyAdminCheck_skipAll() public {
        HasOwner hasOwner = new HasOwner(msg.sender);
        Options memory opts;
        opts.unsafeSkipAllChecks = true;

        Upgrades.deployTransparentProxy(
            "Greeter.sol",
            address(hasOwner),
            abi.encodeCall(Greeter.initialize, (msg.sender, "hello")),
            opts
        );
    }

    function testWarningAndError() public {
        Options memory opts;
        opts.unsafeAllow = "state-variable-immutable";

        Invoker i = new Invoker();
        try i.validateImplementation("Validations.sol:HasWarningAndError", opts) {
            fail();
        } catch Error(string memory reason) {
            assertTrue(vm.contains(reason, "Use of delegatecall is not allowed"));
        }
    }
}

contract Invoker {
    function deployTransparentProxy(
        string memory contractName,
        address admin,
        bytes memory data
    ) public returns (address) {
        return Upgrades.deployTransparentProxy(contractName, admin, data);
    }

    function deployTransparentProxy(
        string memory contractName,
        address admin,
        bytes memory data,
        Options memory opts
    ) public returns (address) {
        return Upgrades.deployTransparentProxy(contractName, admin, data, opts);
    }

    function validateImplementation(string memory contractName, Options memory opts) public {
        Upgrades.validateImplementation(contractName, opts);
    }

    function validateUpgrade(string memory contractName, Options memory opts) public {
        Upgrades.validateUpgrade(contractName, opts);
    }
}
