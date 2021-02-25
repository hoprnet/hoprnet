// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.6.0;


import "@openzeppelin/contracts/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./HoprToken.sol";


/**
 * HoprWrapper is a smart contract that may receive xHOPR (ERC-677)
 * and it will send back wxHOPR (ERC-777) and vice versa.
 */
contract HoprWrapper is IERC777Recipient, ERC1820Implementer, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // setup ERC1820
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");

    // xHOPR token
    IERC20 public xHOPR;
    // wrapped xHOPR address
    HoprToken public wxHOPR;

    event Wrapped(address indexed account, uint256 amount);
    event Unwrapped(address indexed account, uint256 amount);

    constructor(IERC20 _xHOPR, HoprToken _wxHOPR) public {
        xHOPR = _xHOPR;
        wxHOPR = _wxHOPR;

        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    }

    /**
     * @dev Hook for xHOPR `transfer` and `transferAndCall` function
     * @param from address token holder address
     * @param amount uint256 amount of tokens to transfer
     * @param data bytes extra information provided by the token holder (if any)
     */
    function onTokenTransfer(
        address from,
        uint256 amount,
        // solhint-disable-next-line no-unused-vars
        bytes calldata data
    ) external nonReentrant returns (bool success) {
        // must be xHOPR token
        require(msg.sender == address(xHOPR), "Sender must be xHOPR");

        // mint wxHOPR
        wxHOPR.mint(from, amount, "", "");

        emit Wrapped(from, amount);
        return true;
    }

    /**
     * @dev Hook triggered when wxHOPR tokens are send to this contract.
     * @param operator address operator requesting the transfer
     * @param from address token holder address
     * @param to address recipient address
     * @param amount uint256 amount of tokens to transfer
     * @param userData bytes extra information provided by the token holder (if any)
     * @param operatorData bytes extra information provided by the operator (if any)
     */
    function tokensReceived(
        address operator,
        address from,
        address to,
        uint256 amount,
        bytes calldata userData,
        // solhint-disable-next-line no-unused-vars
        bytes calldata operatorData
    ) external override nonReentrant {
        // must be wxHOPR
        require(msg.sender == address(wxHOPR), "Sender must be wxHOPR");
        require(to == address(this), "Must be sending tokens to HoprWrapper");

        wxHOPR.burn(amount, "");
        xHOPR.safeTransfer(from, amount);

        emit Unwrapped(from, amount);
    }
}
