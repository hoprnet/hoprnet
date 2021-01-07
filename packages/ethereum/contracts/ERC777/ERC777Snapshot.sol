pragma solidity ^0.6.0;

// SPDX-License-Identifier: LGPL-3.0-only

import "@openzeppelin/contracts/math/SafeMath.sol";
// import "@openzeppelin/contracts/utils/Arrays.sol";
import "@openzeppelin/contracts/token/ERC777/ERC777.sol";
import "../utils/console.sol";

/**
 * @dev TODO: add documentantion
 */
abstract contract ERC777Snapshot is ERC777 {
    // Inspired by Jordi Baylina's MiniMeToken to record historical balances:
    // https://github.com/Giveth/minime/blob/ea04d950eea153a04c51fa510b068b9dded390cb/contracts/MiniMeToken.sol

    using SafeMath for uint256;
    // using Arrays for uint256[];

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

    // `balances` is the map that tracks the balance of each address, in this
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
    function balanceOfAt(address _owner, uint _blockNumber) external view returns (uint) {
        if (
            (accountSnapshots[_owner].length == 0) ||
            (accountSnapshots[_owner][0].fromBlock > _blockNumber)
        ) {
            return 0;
        } else {
            return _valueAt(accountSnapshots[_owner], _blockNumber);
        }
    }

    /**
     * @notice Total amount of tokens at a specific `_blockNumber`.
     * @param _blockNumber The block number when the totalSupply is queried
     * @return The total amount of tokens at `_blockNumber`
     */
    function totalSupplyAt(uint _blockNumber) external view returns(uint) {
        if (
            (totalSupplySnapshots.length == 0) ||
            (totalSupplySnapshots[0].fromBlock > _blockNumber)
        ) {
            return 0;
        } else {
            return _valueAt(totalSupplySnapshots, _blockNumber);
        }
    }

    // Update balance and/or total supply snapshots before the values are modified. This is implemented
    // in the _beforeTokenTransfer hook, which is executed for _mint, _burn, and _transfer operations.
    function _beforeTokenTransfer(address operator, address from, address to, uint256 amount) internal virtual override {
        super._beforeTokenTransfer(operator, from, to, amount);

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
        uint _block
    ) view internal returns (uint) {
        if (snapshots.length == 0) return 0;

        // Shortcut for the actual value
        if (_block >= snapshots[snapshots.length-1].fromBlock) {
            return snapshots[snapshots.length-1].value;
        }
        if (_block < snapshots[0].fromBlock) {
            return 0;
        }

        // Binary search of the value in the array
        // TODO: maybe use OZs Array util
        uint min = 0;
        uint max = snapshots.length-1;
        while (max > min) {
            uint mid = (max + min + 1)/ 2;
            if (snapshots[mid].fromBlock<=_block) {
                min = mid;
            } else {
                max = mid-1;
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
        if (
            (snapshots.length == 0) ||
            (snapshots[snapshots.length -1].fromBlock < block.number)
        ) {
            // TODO: check gas costs / best practises
            snapshots.push(
                Snapshot(
                    uint128(block.number),
                    uint128(_value)
                )
            );
        } else {
            snapshots[snapshots.length-1].value = uint128(_value);
        }
    }
}
