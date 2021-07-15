// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.6.0;

import "@openzeppelin/contracts-v3-0-1/math/SafeMath.sol";
import "../openzeppelin-contracts/ERC777.sol";

/**
 * @dev This contract extends an ERC777 token with a snapshot mechanism. When a snapshot is created, the balances and
 * total supply at the time are recorded for later access.
 *
 * This can be used to safely create mechanisms based on token balances such as trustless dividends or weighted voting.
 * In naive implementations it's possible to perform a "double spend" attack by reusing the same balance from different
 * accounts. By using snapshots to calculate dividends or voting power, those attacks no longer apply. It can also be
 * used to create an efficient ERC20 forking mechanism.
 *
 * Snapshots are created by the internal {updateValueAtNow} function.
 * To get the total supply at the time of a snapshot, call the function {totalSupplyAt} with a block number.
 * To get the balance of an account at the time of a snapshot, call the {balanceOfAt} function with a block number
 * and the account address.
 */
abstract contract ERC777Snapshot is ERC777 {
    // Inspired by Jordi Baylina's MiniMeToken to record historical balances:
    // https://github.com/Giveth/minime/blob/ea04d950eea153a04c51fa510b068b9dded390cb/contracts/MiniMeToken.sol

    using SafeMath for uint256;

    /**
     * @dev `Snapshot` is the structure that attaches a block number to a
     * given value, the block number attached is the one that last changed the
     * value
     */
    struct Snapshot {
        // `fromBlock` is the block number that the value was generated from
        uint128 fromBlock;
        // `value` is the amount of tokens at a specific block number
        uint128 value;
    }

    // `accountSnapshots` is the map that tracks the balance of each address, in this
    //  contract when the balance changes the block number that the change
    //  occurred is also included in the map
    mapping (address => Snapshot[]) public accountSnapshots;

    // Tracks the history of the `totalSupply` of the token
    Snapshot[] public totalSupplySnapshots;

    /**
     * @dev Queries the balance of `_owner` at a specific `_blockNumber`
     * @param _owner The address from which the balance will be retrieved
     * @param _blockNumber The block number when the balance is queried
     * @return The balance at `_blockNumber`
     */
    function balanceOfAt(address _owner, uint128 _blockNumber) external view returns (uint256) {
        return _valueAt(accountSnapshots[_owner], _blockNumber);
    }

    /**
     * @notice Total amount of tokens at a specific `_blockNumber`.
     * @param _blockNumber The block number when the totalSupply is queried
     * @return The total amount of tokens at `_blockNumber`
     */
    function totalSupplyAt(uint128 _blockNumber) external view returns(uint256) {
        return _valueAt(totalSupplySnapshots, _blockNumber);
    }

    // Update balance and/or total supply snapshots before the values are modified. This is implemented
    // in the _beforeTokenTransfer hook, which is executed for _mint, _burn, and _transfer operations.
    function _beforeTokenTransfer(address operator, address from, address to, uint256 amount) internal virtual override {
        if (from == address(0)) {
            // mint
            updateValueAtNow(accountSnapshots[to], balanceOf(to).add(amount));
            updateValueAtNow(totalSupplySnapshots, totalSupply().add(amount));
        } else if (to == address(0)) {
            // burn
            updateValueAtNow(accountSnapshots[from], balanceOf(from).sub(amount));
            updateValueAtNow(totalSupplySnapshots, totalSupply().sub(amount));
        } else if (from != to) {
            // transfer
            updateValueAtNow(accountSnapshots[from], balanceOf(from).sub(amount));
            updateValueAtNow(accountSnapshots[to], balanceOf(to).add(amount));
        }
    }

    /**
     * @dev `_valueAt` retrieves the number of tokens at a given block number
     * @param snapshots The history of values being queried
     * @param _block The block number to retrieve the value at
     * @return The number of tokens being queried
     */
    function _valueAt(
        Snapshot[] storage snapshots,
        uint128 _block
    ) view internal returns (uint256) {
        uint256 lenSnapshots = snapshots.length;
        if (lenSnapshots == 0) return 0;

        // Shortcut for the actual value
        if (_block >= snapshots[lenSnapshots - 1].fromBlock) {
            return snapshots[lenSnapshots - 1].value;
        }
        if (_block < snapshots[0].fromBlock) {
            return 0;
        }

        // Binary search of the value in the array
        uint256 min = 0;
        uint256 max = lenSnapshots - 1;
        while (max > min) {
            uint256 mid = (max + min + 1) / 2;

            uint256 midSnapshotFrom = snapshots[mid].fromBlock;
            if (midSnapshotFrom == _block) {
                return snapshots[mid].value;
            } else if (midSnapshotFrom < _block) {
                min = mid;
            } else {
                max = mid - 1;
            }
        }

        return snapshots[min].value;
    }

    /**
     * @dev `updateValueAtNow` used to update the `balances` map and the
     *  `totalSupplySnapshots`
     * @param snapshots The history of data being updated
     * @param _value The new number of tokens
     */
    function updateValueAtNow(Snapshot[] storage snapshots, uint256 _value) internal {
        require(_value <= uint128(-1), "casting overflow");
        uint256 lenSnapshots = snapshots.length;

        if (
            (lenSnapshots == 0) ||
            (snapshots[lenSnapshots - 1].fromBlock < block.number)
        ) {
            snapshots.push(
                Snapshot(
                    uint128(block.number),
                    uint128(_value)
                )
            );
        } else {
            snapshots[lenSnapshots - 1].value = uint128(_value);
        }
    }
}
