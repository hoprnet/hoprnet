// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// This contract is for testing only.

library StringHelper {
    /// Join an array of strings with a space between each element.
    function join(string[] memory arr) internal pure returns (string memory) {
        string memory result;
        for (uint i = 0; i < arr.length; i++) {
            result = string.concat(result, arr[i]);
            if (i < arr.length - 1) {
                result = string.concat(result, " ");
            }
        }
        return result;
    }
}
