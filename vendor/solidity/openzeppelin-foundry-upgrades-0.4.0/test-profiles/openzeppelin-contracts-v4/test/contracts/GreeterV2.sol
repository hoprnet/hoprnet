// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

/// @custom:oz-upgrades-from Greeter
contract GreeterV2 is Initializable, OwnableUpgradeable {
    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        _disableInitializers();
    }

    string public greeting;

    function initialize(string memory _greeting) initializer public {
        __Ownable_init();
        greeting = _greeting;
    }

    function resetGreeting() public reinitializer(2) {
        greeting = "resetted";
    }

    function setGreeting(string memory _greeting) public onlyOwner {
        greeting = _greeting;
    }
}
