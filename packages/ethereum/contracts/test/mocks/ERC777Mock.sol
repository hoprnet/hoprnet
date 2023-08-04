// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <=0.9.0;

import "openzeppelin-contracts-4.4.2/utils/Context.sol";
import "openzeppelin-contracts-4.4.2/token/ERC777/ERC777.sol";

contract ERC777Mock is Context, ERC777 {
    constructor(
        address initialHolder,
        uint256 initialBalance,
        string memory name,
        string memory symbol,
        address[] memory defaultOperators
    ) ERC777(name, symbol, defaultOperators) {
        _mint(initialHolder, initialBalance, "", "");
    }

    function mintInternal(address to, uint256 amount, bytes memory userData, bytes memory operatorData) public {
        _mint(to, amount, userData, operatorData);
    }

    function approveInternal(address holder, address spender, uint256 value) public {
        _approve(holder, spender, value);
    }
}
