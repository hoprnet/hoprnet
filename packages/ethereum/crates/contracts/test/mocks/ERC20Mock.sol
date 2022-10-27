// SPDX-License-Identifier: GPL-3.0-only

pragma solidity >=0.6.0 <0.9.0;

import '@openzeppelin/contracts/token/ERC20/ERC20.sol';

/**
 * @title ERC20Mock
 * This mock just provides a public safeMint, mint, and burn functions for testing purposes
 */
contract ERC20Mock is ERC20 {
  constructor(address initialAccount, uint256 initialBalance) payable ERC20('ERC20 Mock', 'M20') {
    _mint(initialAccount, initialBalance);
  }

  function mint(address account, uint256 amount) public {
    _mint(account, amount);
  }

  function burn(address account, uint256 amount) public {
    _burn(account, amount);
  }

  function transferInternal(
    address from,
    address to,
    uint256 value
  ) public {
    _transfer(from, to, value);
  }

  function approveInternal(
    address owner,
    address spender,
    uint256 value
  ) public {
    _approve(owner, spender, value);
  }
}
