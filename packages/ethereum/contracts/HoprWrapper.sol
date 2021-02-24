// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.6.0;


import "@openzeppelin/contracts/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./HoprToken.sol";


/**
 * HoprWrapper is a smart contract that may receive xHOPR (ERC-677)
 * and it will send back wxHOPR (ERC-777) and vice versa.
 */
contract HoprWrapper is IERC777Recipient, ERC1820Implementer, ReentrancyGuard {
    using SafeMath for uint256;
    using SafeERC20 for IERC20;

    // setup ERC1820
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");

    // xHOPR token
    IERC20 public xHOPR;
    // wrapped xHOPR address
    HoprToken public wxHOPR;

    // total amount of xHOPR collected in this contract
    uint256 total_xHOPR = 0;

    event Wrapped(address indexed account, uint256 amount);
    event Unwrapped(address indexed account, uint256 amount);

    constructor(IERC20 _xHOPR, HoprToken _wxHOPR) public {
        xHOPR = _xHOPR;
        wxHOPR = _wxHOPR;

        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    }

    /**
     * A hook triggered when xHOPR or wxHOPR tokens are send to this contract.
     *
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
        // solhint-disable-next-line no-unused-vars
        address to,
        uint256 amount,
        bytes calldata userData,
        // solhint-disable-next-line no-unused-vars
        bytes calldata operatorData
    ) external override nonReentrant {
        // must be xHOPR or wxHOPR
        require(msg.sender == address(xHOPR) || msg.sender == address(wxHOPR), "Invalid token");
        // must not be triggered by self
        if (operator == address(this)) return;

        if (msg.sender == address(xHOPR)) {
            total_xHOPR = total_xHOPR.add(amount);
            wxHOPR.mint(from, amount, "", "");

            emit Wrapped(from, amount);
        } else {
            total_xHOPR = total_xHOPR.sub(amount);
            wxHOPR.burn(amount, "");
            xHOPR.safeTransfer(from, amount);

            emit Unwrapped(from, amount);
        }
    }
}
