// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.7.5;

import "../ERC777/ERC777Snapshot.sol";

contract ERC777SnapshotMock is ERC777Snapshot {
    constructor(
        string memory name,
        string memory symbol,
        address initialAccount,
        uint256 initialBalance
    ) ERC777(name, symbol, new address[](0)) {
        _mint(initialAccount, initialBalance, "", "");
    }

    function snapshot() public {
        _snapshot();
    }

    function mint(
        address to,
        uint256 amount,
        bytes memory userData,
        bytes memory operatorData
    ) public {
        _mint(to, amount, userData, operatorData);
    }

    function burn(
        address account,
        uint256 amount,
        bytes memory userData,
        bytes memory operatorData
    ) public {
        _burn(account, amount, userData, operatorData);
    }
}
