// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {Upgrades, Options} from "openzeppelin-foundry-upgrades/Upgrades.sol";

import {MyContract} from "./contracts/MyContract.sol";

contract UpgradesTest is Test {
    function testValidateWithReferenceBuildInfo_referenceContract() public {
        Options memory opts;
        opts.referenceBuildInfoDir = "test_artifacts/build-info-v1";
        opts.referenceContract = "build-info-v1:MyContract";

        Upgrades.validateUpgrade(
            "MyContract.sol",
            opts
        );
    }
}
