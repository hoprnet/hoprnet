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
    uint256 public secs_expire;     // seconds it takes to expire channel after channel's creation
    uint256 public secs_closure;    // seconds it takes to allow closing of channel after channel's -
                                    // 'sender' provided a signature

    // the payment channel has been created and opened
    event OpenedChannel(
        uint256 id,
        address indexed funder,
        address indexed sender,
        address indexed recipient,
        uint256 deposit,
        uint256 expiration_time
    );

    // the payment channel's 'sender' is closing the channel
    event InitiatedChannelClosure(uint256 indexed id, uint256 closure_amount, uint256 closure_time);

    // the payment channel has been settled and closed
    event ClosedChannel(uint256 indexed id, uint256 senderAmount, uint256 recipientAmount);

    enum ChannelStatus {
        NONE,               // channel does not exist
        OPEN,               // channel is open
        PENDING_CLOSURE,    // channel is pending for closure
        CLOSE               // channel is closed
    }

    struct Channel {
        address sender;             // the account sending payments
        address recipient;          // the account receiving the payments
        uint256 deposit;            // the token deposit
        uint256 expiration_time;    // the time when the channel expires
        uint256 closure_amount;     // the amount of tokens that will be send to the recipient -
                                    // if channel is closed by 'sender'
        uint256 closure_time;       // the time when the channel can be closed by 'sender'
        ChannelStatus status;       // channel's status
    }

    mapping(uint256 => Channel) public channels;    // store channels by channelId
    uint256 public numberOfChannels = 0;            // number of channels

    constructor(IERC20 _token, uint256 _secs_expire, uint256 _secs_closure) public {
        token = _token;
        secs_expire = _secs_expire;
        secs_closure = _secs_closure;
    }

    // channel's status must be equal to 'status'
    modifier statusMustBe(uint256 channelId, ChannelStatus status) {
        require(uint256(channels[channelId].status) == uint256(status), "channel's status is wrong");
        _;
    }

    // caller must be channel's sender
    modifier callerMustBeSender(uint256 channelId) {
        require(msg.sender == channels[channelId].sender, "caller is not channel's sender");
        _;
    }

    // caller must be channel's recipient
    modifier callerMustBeRecipient(uint256 channelId) {
        require(msg.sender == channels[channelId].recipient, "caller is not channel's recipient");
        _;
    }

    // create a channel, specified tokens must be approved beforehand
    function createChannel(
        address funder,
        address sender,
        address recipient,
        uint256 amount
    ) external statusMustBe(numberOfChannels.add(1), ChannelStatus.NONE) returns (uint256) {
        require(amount > 0, "amount must be larger than 0");

        uint256 channelId = numberOfChannels.add(1);
        token.safeTransferFrom(funder, address(this), amount);
        numberOfChannels = channelId;

        uint256 expiretionTime = now + secs_expire;

        channels[channelId] = Channel(
            sender,
            recipient,
            amount,
            expiretionTime,
            0,
            0,
            ChannelStatus.OPEN
        );

        emit OpenedChannel(channelId, funder, sender, recipient, amount, expiretionTime);

        return channelId;
    }

    // close a channel, the recipient can close the channel at any time
    // by presenting a signed amount from the sender. The recipient will
    // be sent that amount, and the remainder will go back to the sender
    function closeChannel(uint256 channelId, uint256 amount, bytes calldata signature)
    external callerMustBeRecipient(channelId) {
        Channel storage channel = channels[channelId];

        require(
            uint256(channel.status) == uint256(ChannelStatus.OPEN) ||
            uint256(channel.status) == uint256(ChannelStatus.PENDING_CLOSURE),
            "channel's status must be 'OPEN' or 'PENDING_CLOSURE'"
        );
        require(isValidSignature(channel.sender, amount, signature), "signature is not valid");
        require(amount > channel.closure_amount, "'amount' must be larger than 'closure_amount'");

        settle(channelId, amount);
    }

    function initiateChannelClosure(uint256 channelId, uint256 amount)
    external callerMustBeSender(channelId) statusMustBe(channelId, ChannelStatus.OPEN) {
        Channel storage channel = channels[channelId];

        uint256 closure_time = now + secs_closure;
        if (closure_time > channel.expiration_time) {
            closure_time = channel.expiration_time;
        }

        channel.closure_amount = amount;
        channel.closure_time = closure_time;
        channel.status = ChannelStatus.PENDING_CLOSURE;

        emit InitiatedChannelClosure(channelId, amount, closure_time);
    }

    // if the timeout is reached without the recipient providing a better signature, then
    // the tokens is released according to `closure_amount`
    function claimChannelClosure(uint256 channelId)
    external statusMustBe(channelId, ChannelStatus.PENDING_CLOSURE) {
        Channel storage channel = channels[channelId];
        require(now >= channel.closure_time, "'closure_time' has not passed");

        settle(channelId, channel.closure_amount);
    }

    // if the timeout is reached without the recipient closing the channel, then
    // the tokens is released back to the sender
    function claimChannelExpiration(uint256 channelId) external {
        Channel storage channel = channels[channelId];
        require(now >= channel.expiration_time, "channel has not expired");

        settle(channelId, channel.closure_amount);
    }

    // settle channel, send 'amount' to recipient and the rest to sender
    function settle(uint256 channelId, uint256 amount) internal {
        Channel storage channel = channels[channelId];

        if (amount > 0) {
            token.safeTransfer(channel.recipient, amount);
            channel.deposit = channel.deposit.sub(amount);
        }

        uint256 remaining = channel.deposit;
        if (remaining > 0) {
            token.safeTransfer(channel.sender, remaining);
        }

        channel.deposit = 0;
        channel.status = ChannelStatus.CLOSE;
        emit ClosedChannel(channelId, remaining, amount);
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

    // revert when somebody is sending ether
    function () external {
        revert("you should not send ethereum to this address");
    }
}