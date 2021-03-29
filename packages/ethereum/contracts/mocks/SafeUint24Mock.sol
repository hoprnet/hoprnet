// SPDX-License-Identifier: MIT
pragma solidity 0.7.5;

import "../utils/SafeUint24.sol";

contract SafeUint24Mock {
    function add(uint24 a, uint24 b) public pure returns (uint24) {
        return SafeUint24.add(a, b);
    }

    function div(uint24 a, uint24 b) public pure returns (uint24) {
        return SafeUint24.div(a, b);
    }

    function mod(uint24 a, uint24 b) public pure returns (uint24) {
        return SafeUint24.mod(a, b);
    }
}