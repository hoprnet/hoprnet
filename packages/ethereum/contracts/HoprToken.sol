// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC777/ERC777.sol";
import "./ERC777/ERC777Snapshot.sol";

contract HoprToken is AccessControl, ERC777Snapshot {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    constructor() ERC777("HOPR Token", "HOPR", new address[](0)) {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    /**
     * @dev Creates `amount` tokens and assigns them to `account`, increasing
     * the total supply.
     *
     * If a send hook is registered for `account`, the corresponding function
     * will be called with `operator`, `data` and `operatorData`.
     * Emits {Minted} and {IERC20-Transfer} events.
     *
     * Requirements
     *
     * - `account` cannot be the zero address.
     * - if `account` is a contract, it must implement the {IERC777Recipient}
     * interface.
     * - `account` must have minter role
     */
    function mint(
        address account,
        uint256 amount,
        bytes memory userData,
        bytes memory operatorData
    ) public {
        require(hasRole(MINTER_ROLE, msg.sender), "caller does not have minter role");
        _mint(account, amount, userData, operatorData);
    }
}
