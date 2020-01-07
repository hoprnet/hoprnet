pragma solidity ^0.5.3;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/utils/Address.sol";

contract PaymentChannel {
    using SafeERC20 for IERC20;
    using SafeMath for uint256;
    using Address for address;

    IERC20 public token;            // the token that will be used to settle payments
    uint256 public secs_closure;    // seconds it takes to allow closing of channel after channel's -
                                    // 'sender' provided a signature

    // the payment channel has been created and opened
    event OpenedChannel(
        address indexed funder,
        address indexed sender,
        address indexed recipient,
        uint256 deposit
    );

    // the payment channel's 'sender' is closing the channel
    event InitiatedChannelClosure(
        address indexed sender,
        address indexed recipient,
        uint256 closure_amount,
        uint256 closure_time
    );

    // the payment channel has been settled and closed
    event ClosedChannel(
        address indexed sender,
        address indexed recipient,
        uint256 senderAmount,
        uint256 recipientAmount
    );

    enum ChannelStatus {
        CLOSED,         // channel is closed
        OPEN,           // channel is open
        PENDING_CLOSURE // channel is pending for closure
    }

    struct Channel {
        uint256 deposit;        // the token deposit
        uint256 closure_amount; // the amount of tokens that will be send to the recipient -
                                // if channel is closed by 'sender'
        uint256 closure_time;   // the time when the channel can be closed by 'sender'
        ChannelStatus status;   // channel's status
    }

    // store channels e.g: channels[sender][recipient]
    mapping(address => mapping(address => Channel)) public channels;

    constructor(IERC20 _token, uint256 _secs_closure) public {
        token = _token;
        secs_closure = _secs_closure;
    }

    // channel's status must be equal to 'status'
    modifier statusMustBe(address sender, address recipient, ChannelStatus status) {
        Channel storage channel = channels[sender][recipient];

        require(uint256(channel.status) == uint256(status), "channel's status is wrong");
        _;
    }

    // msg.sender must be equal to 'caller'
    modifier senderMustBe(address caller) {
        require(msg.sender == caller, "msg.sender is not required caller");
        _;
    }

    // create a channel, specified tokens must be approved beforehand
    function createChannel(address funder, address sender, address recipient, uint256 amount)
    external statusMustBe(sender, recipient, ChannelStatus.CLOSED) {
        require(funder != address(0), "'funder' address is empty");
        require(sender != address(0), "'sender' address is empty");
        require(recipient != address(0), "'recipient' address is empty");
        require(amount > 0, "'amount' must be larger than 0");

        token.safeTransferFrom(funder, address(this), amount);

        channels[sender][recipient] = Channel(
            amount,
            0,
            0,
            ChannelStatus.OPEN
        );

        emit OpenedChannel(funder, sender, recipient, amount);
    }

    // close a channel, the recipient can close the channel at any time
    // by presenting a signed amount from the sender. The recipient will
    // be sent that amount, and the remainder will go back to the sender
    function closeChannel(address sender, address recipient, uint256 amount, bytes calldata signature)
    external senderMustBe(recipient) {
        Channel storage channel = channels[sender][recipient];

        require(
            uint256(channel.status) == uint256(ChannelStatus.OPEN) ||
            uint256(channel.status) == uint256(ChannelStatus.PENDING_CLOSURE),
            "channel's status must be 'OPEN' or 'PENDING_CLOSURE'"
        );
        require(isValidSignature(sender, amount, signature), "signature is not valid");
        require(amount > channel.closure_amount, "'amount' must be larger than 'closure_amount'");

        settle(sender, recipient, amount);
    }

    function initiateChannelClosure(address sender, address recipient, uint256 amount) external
    senderMustBe(sender) statusMustBe(sender, recipient, ChannelStatus.OPEN) {
        Channel storage channel = channels[sender][recipient];

        uint256 closure_time = now + secs_closure;

        channel.closure_amount = amount;
        channel.closure_time = closure_time;
        channel.status = ChannelStatus.PENDING_CLOSURE;

        emit InitiatedChannelClosure(sender, recipient, amount, closure_time);
    }

    // if the timeout is reached without the recipient providing a better signature, then
    // the tokens is released according to `closure_amount`
    function claimChannelClosure(address sender, address recipient)
    external statusMustBe(sender, recipient, ChannelStatus.PENDING_CLOSURE) {
        Channel storage channel = channels[sender][recipient];
        require(now >= channel.closure_time, "'closure_time' has not passed");

        settle(sender, recipient, channel.closure_amount);
    }

    // settle channel, send 'amount' to recipient and the rest to sender
    function settle(address sender, address recipient, uint256 amount) internal {
        Channel storage channel = channels[sender][recipient];

        if (amount > 0) {
            token.safeTransfer(recipient, amount);
            channel.deposit = channel.deposit.sub(amount);
        }

        uint256 remaining = channel.deposit;
        if (remaining > 0) {
            token.safeTransfer(sender, remaining);
        }

        // channel.deposit = 0;
        // channel.closure_amount = 0;
        // channel.closure_time = 0;
        channel.status = ChannelStatus.CLOSED;
        emit ClosedChannel(sender, recipient, remaining, amount);
    }

    // return 'true' if signaure is signed by 'signer'
    function isValidSignature(address signer, uint256 amount, bytes memory signature) internal view returns (bool) {
        bytes32 message = prefixed(keccak256(abi.encodePacked(address(this), amount)));

        return recoverSigner(message, signature) == signer;
    }

    function splitSignature(bytes memory signature) internal pure returns (uint8, bytes32, bytes32) {
        require(signature.length == 65, "signature length is not 65");

        bytes32 r;
        bytes32 s;
        uint8 v;

        assembly {
            // first 32 bytes, after the length prefix
            r := mload(add(signature, 32))
            // second 32 bytes
            s := mload(add(signature, 64))
            // final byte (first byte of the next 32 bytes)
            v := byte(0, mload(add(signature, 96)))
        }

        return (v, r, s);
    }

    function recoverSigner(bytes32 message, bytes memory signature) internal pure returns (address) {
        uint8 v;
        bytes32 r;
        bytes32 s;

        (v, r, s) = splitSignature(signature);

        return ecrecover(message, v, r, s);
    }

    // builds a prefixed hash to mimic the behavior of eth_sign
    function prefixed(bytes32 message) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
    }
}