pragma solidity ^0.6.0;

// SPDX-License-Identifier: LGPL-3.0-only

import "../ERC777/ERC777Snapshot.sol";

contract ERC777SnapshotMock is ERC777Snapshot {
    constructor(
        string memory name,
        string memory symbol,
        address initialAccount,
        uint256 initialBalance
    ) public ERC777(name, symbol, new address[](0)) {
        _mint(initialAccount, initialBalance, "", "");
    }

    function updateValueAtNowAccount(
        address account,
        uint256 value
    ) external {
        updateValueAtNow(accountSnapshots[account], value);
    }

    function mint(
        address to,
        uint256 amount,
        bytes calldata userData,
        bytes calldata operatorData
    ) external {
        _mint(to, amount, userData, operatorData);
    }

    function burn(
        address account,
        uint256 amount,
        bytes calldata userData,
        bytes calldata operatorData
    ) external {
        _burn(account, amount, userData, operatorData);
    }
}
