pragma solidity ^0.6.0;

import "@openzeppelin/contracts/GSN/Context.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Snapshot.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20Burnable.sol";


contract HoprToken is Context, AccessControl, ERC20Snapshot, ERC20Burnable {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    constructor() public ERC20("HOPR Token", "HOPR") {
        _setupRole(DEFAULT_ADMIN_ROLE, _msgSender());
        _setupRole(MINTER_ROLE, _msgSender());
    }

    /**
     * @dev Creates `amount` new tokens for `to`.
     *
     * See {ERC20-_mint}.
     *
     * Requirements:
     *
     * - the caller must have the `MINTER_ROLE`.
     */
    function mint(address to, uint256 amount) public {
        require(hasRole(MINTER_ROLE, _msgSender()), "HoprToken: must have minter role to mint");
        _mint(to, amount);
    }

    function _transfer(
        address sender,
        address recipient,
        uint256 amount
    ) internal override(ERC20, ERC20Snapshot) {
        super._transfer(sender, recipient, amount);
    }

    function _mint(address account, uint256 amount) internal override(ERC20, ERC20Snapshot) {
        super._mint(account, amount);
    }

    function _burn(address account, uint256 amount) internal override(ERC20, ERC20Snapshot) {
        super._burn(account, amount);
    }
}
