pragma solidity ^0.6.0;

// SPDX-License-Identifier: LGPL-3.0-only

import "@openzeppelin/contracts/GSN/Context.sol";
import "@openzeppelin/contracts/token/ERC777/ERC777.sol";

contract ERC777Mock is Context, ERC777 {
    constructor(
        address initialHolder,
        uint256 initialBalance,
        string memory name,
        string memory symbol,
        address[] memory defaultOperators
    ) public ERC777(name, symbol, defaultOperators) {
        _mint(initialHolder, initialBalance, "", "");
    }

    function mintInternal(
        address to,
        uint256 amount,
        bytes memory userData,
        bytes memory operatorData
    ) public {
        _mint(to, amount, userData, operatorData);
    }

    function approveInternal(
        address holder,
        address spender,
        uint256 value
    ) public {
        _approve(holder, spender, value);
    }
}
