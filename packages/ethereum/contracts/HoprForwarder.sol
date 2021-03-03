// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.6.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";

contract HoprForwarder is IERC777Recipient {
    using SafeERC20 for IERC20;
    
    IERC1820Registry public constant ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");
    address public constant HOPR_TOKEN = 0xF5581dFeFD8Fb0e4aeC526bE659CFaB1f8c781dA;
    address payable public constant MULTISIG = 0x4F50Ab4e931289344a57f2fe4bBd10546a6fdC17;
    
    /**
     * @dev register this contract to ERC1820 registry
     */
    constructor() public {
         ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    }
    
    /**
     * @dev token recipient hook for ERC777
     * @param operator address operator requesting the transfer
     * @param from address token holder address. Should be zddress zero for token mintin
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
        bytes calldata operatorData
    ) external override {
        require(msg.sender == HOPR_TOKEN, "HoprMinterWrapper: Only accept HOPR token");
        require(from == address(0), "HoprMinterWrapper: Only receive mint");
        require(to == address(this), "HoprMinterWrapper: Must be sending tokens to the minter wrapper");
        IERC20(HOPR_TOKEN).safeTransfer(MULTISIG, amount);
    }

    /**
     * @dev Recover any residual ERC20 token/ETH to the multisig
     * @param token ERC20 token address that is sent to this contract. Address 0 for ETH
     */
    function recoverTokens(address token) external {
        if (token == address (0)) {
            MULTISIG.transfer(address(this).balance);           
        } else {
            IERC20(token).safeTransfer(MULTISIG, IERC20(token).balanceOf(address(this)));
        }
    }
}