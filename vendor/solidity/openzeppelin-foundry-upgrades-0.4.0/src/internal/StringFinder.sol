// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Vm} from "forge-std/Vm.sol";
import {Utils} from "./Utils.sol";

/**
 * String finder functions using Forge's string cheatcodes.
 * For internal use only.
 */
library StringFinder {
    /**
     * Returns whether the subject string contains the search string.
     */
    function contains(string memory subject, string memory search) internal returns (bool) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);
        return vm.contains(subject, search);
    }

    /**
     * Returns whether the subject string starts with the search string.
     */
    function startsWith(string memory subject, string memory search) internal pure returns (bool) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);
        uint256 index = vm.indexOf(subject, search);
        return index == 0;
    }

    /**
     * Returns whether the subject string ends with the search string.
     */
    function endsWith(string memory subject, string memory search) internal pure returns (bool) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);
        string[] memory tokens = vm.split(subject, search);
        return tokens.length > 1 && bytes(tokens[tokens.length - 1]).length == 0;
    }

    /**
     * Returns the number of non-overlapping occurrences of the search string in the subject string.
     */
    function count(string memory subject, string memory search) internal pure returns (uint256) {
        Vm vm = Vm(Utils.CHEATCODE_ADDRESS);
        string[] memory tokens = vm.split(subject, search);
        return tokens.length - 1;
    }
}
