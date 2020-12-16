// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.7.5;

import "@openzeppelin/contracts/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "./Accounts.sol";
import "./Channels.sol";
import "./Tickets.sol";

contract HoprChannels is IERC777Recipient, ERC1820Implementer, Accounts, Channels, Tickets {
    using SafeERC20 for IERC20;

    // required by ERC1820 spec
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    // required by ERC777 spec
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_SIZE = abi.encode(false, address(0), address(0)).length;
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_MULTI_SIZE = abi.encode(false, address(0), address(0), uint256(0), uint256(0)).length;

    /**
     * @dev HoprToken, the token that will be used to settle payments
     */
    IERC20 public token;

    /**
     * @param _token HoprToken address
     * @param _secsClosure seconds until a channel can be closed
     */
    constructor(address _token, uint256 _secsClosure) {
        token = IERC20(_token);
        secsClosure = _secsClosure;
        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    }

    /**
     * @dev Initializes an account,
     * stores it's public key, secret and counter,
     * then emits {AccountInitialized} and {AccountSecretUpdated} events.
     * @param secret account's secret
     * @param pubKeyFirstHalf first half of the public key
     * @param pubKeySecondHalf second half of the public key
     */
    function initializeAccount(
        bytes32 secret,
        uint256 pubKeyFirstHalf,
        uint256 pubKeySecondHalf
    ) external {
        _initializeAccount(
            msg.sender,
            pubKeyFirstHalf,
            pubKeySecondHalf,
            secret
        );
    }

    /**
     * @dev Updates account's secret and counter,
     * then emits {AccountSecretUpdated} event.
     * @param secret account's secret
     */
    function updateAccount(
        bytes32 secret
    ) external {
        _updateAccount(msg.sender, secret);
    }

    /**
     * @dev Funds a channel in one direction,
     * then emits {ChannelFunded} event.
     * @param account the address of the recipient
     * @param counterparty the address of the counterparty
     * @param amount amount to fund
     */
    function fundChannel(
        address account,
        address counterparty,
        uint256 amount
    ) external {
        token.safeTransferFrom(msg.sender, address(this), amount);

        _fundChannel(
            msg.sender,
            account,
            counterparty,
            amount,
            0
        );
    }

    /**
     * @dev Funds a channel, in both directions,
     * then emits {ChannelFunded} event.
     * @param accountA the address of accountA
     * @param accountB the address of accountB
     * @param amountA amount to fund accountA
     * @param amountB amount to fund accountB
     */
    function fundChannelMulti(
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        // @TODO: use SafeMath
        token.safeTransferFrom(msg.sender, address(this), amountA + amountB);

        _fundChannel(
            msg.sender,
            accountA,
            accountB,
            amountA,
            amountB
        );
    }

    /**
     * @dev Opens a channel, then emits
     * {ChannelOpened} event.
     * @param counterparty the address of the counterparty
     */
    function openChannel(address counterparty) external {
        _openChannel(msg.sender, counterparty);
    }

    /**
     * @dev Fund channel and then open it, then emits
     * {ChannelFunded} and {ChannelOpened} events.
     * @param accountA the address of accountA
     * @param accountB the address of accountB
     * @param amountA amount to fund accountA
     * @param amountB amount to fund accountB
     */
    function fundAndOpenChannel(
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        address opener = msg.sender;
        require(
            opener == accountA || opener == accountB,
            "opener must be accountA or accountB"
        );

        // @TODO: use SafeMath
        token.safeTransferFrom(msg.sender, address(this), amountA + amountB);

        address counterparty;
        if (opener == accountA) {
            counterparty = accountB;
        } else {
            counterparty = accountA;
        }

        _fundChannel(opener, accountA, accountB, amountA, amountB);
        _openChannel(opener, counterparty);
    }

    function redeemTicket(
        address counterparty,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) external {
        _redeemTicket(
            msg.sender,
            counterparty,
            secretPreImage,
            proofOfRelaySecret,
            amount,
            winProb,
            r,
            s,
            v
        );
    }

    /**
     * @dev Initialize channel closure, updates channel's
     * closure time, when the cool-off period is over,
     * user may finalize closure, then emits
     * {ChannelPendingToClose} event.
     * @param counterparty the address of the counterparty
     */
    function initiateChannelClosure(
        address counterparty
    ) external {
        _initiateChannelClosure(msg.sender, counterparty);
    }

    /**
     * @dev Finalize channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the parties involved, then emits
     * {ChannelClosed} event.
     * @param counterparty the address of the counterparty
     */
    function finalizeChannelClosure(
        address counterparty
    ) external {
        _finalizeChannelClosure(
            token,
            msg.sender,
            counterparty
        );
    }

    /**
     * A hook triggered when HOPR tokens are send to this contract.
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
    ) external override {
        require(msg.sender == address(token), "caller must be HoprToken");

        if (
            operator == address(this) || // must not be triggered by HoprChannels
            from == address(0) // ignore 'mint'
        ) {
            return;
        }

        // must be one of our supported functions
        require(
            userData.length == FUND_CHANNEL_SIZE ||
            userData.length == FUND_CHANNEL_MULTI_SIZE,
            "userData must match one of our supported functions"
        );

        bool shouldOpen;
        address accountA;
        address accountB;
        uint256 amountA;
        uint256 amountB;

        if (userData.length == FUND_CHANNEL_SIZE) {
            (shouldOpen, accountA, accountB) = abi.decode(userData, (bool, address, address));
            amountA = amount;
        } else {
            (shouldOpen, accountA, accountB, amountA, amountB) = abi.decode(userData, (bool, address, address, uint256, uint256));
            require(amount == amountA + amountB, "amount sent must be equal to amount specified");
        }

        _fundChannel(from, accountA, accountB, amountA, amountB);

        if (shouldOpen) {
            require(from == accountA || from == accountB, "funder must be either accountA or accountB");
            _openChannel(accountA, accountB);
        }
    }
}