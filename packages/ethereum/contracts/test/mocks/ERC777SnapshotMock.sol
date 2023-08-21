// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { ERC777Snapshot, ERC777 } from "../../src/static/ERC777/ERC777Snapshot.sol";

contract ERC777SnapshotMock is ERC777Snapshot {
    constructor(
        string memory name,
        string memory symbol,
        address initialAccount,
        uint256 initialBalance
    )
        ERC777(name, symbol, new address[](0))
    {
        _mint(initialAccount, initialBalance, "", "");
    }

    function getAccountValueAt(address _owner, uint128 _blockNumber) external view returns (uint128) {
        return uint128(_valueAt(accountSnapshots[_owner], _blockNumber));
    }

    function getTotalSupplyValueAt(uint128 _blockNumber) external view returns (uint128) {
        return uint128(_valueAt(totalSupplySnapshots, _blockNumber));
    }

    function updateValueAtNowAccount(address account, uint256 value) external {
        updateValueAtNow(accountSnapshots[account], value);
    }

    function mint(address to, uint256 amount, bytes calldata userData, bytes calldata operatorData) external {
        _mint(to, amount, userData, operatorData);
    }

    function burn(address account, uint256 amount, bytes calldata userData, bytes calldata operatorData) external {
        _burn(account, amount, userData, operatorData);
    }
}
