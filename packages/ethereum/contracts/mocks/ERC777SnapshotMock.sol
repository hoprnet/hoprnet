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
