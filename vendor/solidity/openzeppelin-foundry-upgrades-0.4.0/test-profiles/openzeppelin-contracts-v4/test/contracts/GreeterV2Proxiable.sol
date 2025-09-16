// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

// These contracts are for testing only, they are not safe for use in production.

/// @custom:oz-upgrades-from GreeterProxiable
contract GreeterV2Proxiable is Initializable, OwnableUpgradeable, UUPSUpgradeable {
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    string public greeting;

    function initialize(string memory _greeting) initializer public {
        __Ownable_init();
        __UUPSUpgradeable_init();
        greeting = _greeting;
    }

    function _authorizeUpgrade(address newImplementation)
        internal
        onlyOwner
        override
    {}

    function resetGreeting() public reinitializer(2) {
        greeting = "resetted";
    }
}
