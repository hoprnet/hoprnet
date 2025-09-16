// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {Upgrades, Options} from "openzeppelin-foundry-upgrades/Upgrades.sol";

import {StringFinder} from "openzeppelin-foundry-upgrades/internal/StringFinder.sol";

import {MyContract} from "./contracts/MyContract.sol";

contract UpgradesTest is Test {
    using StringFinder for string;

    function testValidateWithReferenceBuildInfo_Bad() public {
        Options memory opts;
        opts.referenceBuildInfoDir = "test_artifacts/build-info-v1";

        Validator v = new Validator();
        try v.validateUpgrade("MyContract.sol", opts) {
            fail();
        } catch Error(string memory reason) {
            assertTrue(reason.contains("Deleted `x`"));
        }
    }
}

contract Validator {
    function validateUpgrade(string memory contractName, Options memory opts) public {
        Upgrades.validateUpgrade(contractName, opts);
    }
}