pragma solidity ^0.6.0;

// SPDX-License-Identifier: LGPL-3.0-only

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "./HoprToken.sol";

/**
 * @dev A HOPR Faucet that is used in public testnets.
 * Anybody can mint tokens, but only an address with 'PAUSER_ROLE'
 * can pause the creation of new tokens.
 */
contract HoprFaucet is AccessControl, Pausable {
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");
    HoprToken public hoprToken;

    constructor(address _hoprToken) public {
        hoprToken = HoprToken(_hoprToken);

        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _setupRole(PAUSER_ROLE, msg.sender);
    }

    /**
     * @dev Mints tokens
     */
    function mint(address account, uint256 amount) external whenNotPaused {
        hoprToken.mint(account, amount, "", "");
    }

    /**
     * @dev Triggers stopped state.
     */
    function pause() external whenNotPaused {
        require(hasRole(PAUSER_ROLE, msg.sender), "HoprFaucet: caller does not have pauser role");
        _pause();
    }

    /**
     * @dev Returns to normal state.
     */
    function unpause() external whenPaused {
        require(hasRole(PAUSER_ROLE, msg.sender), "HoprFaucet: caller does not have pauser role");
        _unpause();
    }
}
