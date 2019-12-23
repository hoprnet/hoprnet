pragma solidity ^0.5.3;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/utils/Address.sol";

contract PaymentChannel {
    using SafeERC20 for IERC20;
    using SafeMath for uint256;
    using Address for address;

    IERC20 public token; //  the token that will be used to settle payments
    uint256 public timeout_duration; // time for timeout after creation

    // inform that a payment channel has been created
    event OpenedChannel(
        uint256 id,
        address indexed funder,
        address indexed sender,
        address indexed recipient,
        uint256 deposit,
        uint256 expiration
    );

    // inform that a payment channel partners that the channel has been settled and closed
    event ClosedChannel(uint256 indexed id, uint256 senderAmount, uint256 recipientAmount);

    enum ChannelStatus {
        NONE,
        OPEN,
        CLOSE
    }

    struct Channel {
        address sender;         //  the account sending payments
        address recipient;      //  the account receiving the payments
        uint256 deposit;        //  the token deposit
        uint256 expiration;     //  timeout time in case the recipient never closes
        ChannelStatus status;   //  channel's status
    }

    mapping(uint256 => Channel) public channels;    //  store channels by channelId
    uint256 public numberOfChannels = 0;            //  number of channels

    constructor(IERC20 _token, uint256 _timeout_duration) public {
        token = _token;
        timeout_duration = _timeout_duration;
    }

    // channel must exist
    function channelMustExist(uint256 channelId) internal view {
        require(uint256(channels[channelId].status) != 0, "channel does not exist");
    }

    // channel must not exist
    function channelMustNotExist(uint256 channelId) internal view {
        require(uint256(channels[channelId].status) == 0, "channel already exists");
    }

    // caller must be channel's sender
    function callerMustBeSender(Channel storage channel) internal view {
        require(msg.sender == channel.sender, "caller is not channel's sender");
    }

    // caller must be channel's recipient
    function callerMustBeRecipient(Channel storage channel) internal view {
        require(msg.sender == channel.recipient, "caller is not channel's recipient");
    }

    // create a channel, specified tokens must be approved beforehand
    function createChannel(
        address funder,
        address sender,
        address recipient,
        uint256 amount
    ) external returns (uint256) {
        uint256 channelId = numberOfChannels.add(1);

        channelMustNotExist(channelId);
        require(amount > 0, "amount must be larger than 0");

        token.safeTransferFrom(funder, address(this), amount);
        numberOfChannels = channelId;

        uint256 expiretionTime = now + timeout_duration;

        channels[channelId] = Channel(
            sender,
            recipient,
            amount,
            expiretionTime,
            ChannelStatus.OPEN
        );

        emit OpenedChannel(channelId, funder, sender, recipient, amount, expiretionTime);

        return channelId;
    }

    // close a channel, the recipient can close the channel at any time
    // by presenting a signed amount from the sender. The recipient will
    // be sent that amount, and the remainder will go back to the sender
    function closeChannel(uint256 channelId, uint256 amount, bytes calldata signature) external {
        channelMustExist(channelId);

        Channel storage channel = channels[channelId];

        callerMustBeRecipient(channel);
        require(isValidSignature(channel.sender, amount, signature), "signature is not valid");

        settle(channelId, amount);
    }

    // if the timeout is reached without the recipient closing the channel, then
    // the ether is released back to the sender
    function claimChannelTimeout(uint256 channelId) external {
        channelMustExist(channelId);

        Channel storage channel = channels[channelId];

        require(now >= channel.expiration, "channel has not expired");

        settle(channelId, 0);
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
    function prefixed(bytes32 hash) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash));
    }

    function isValidSignature(address sender, uint256 amount, bytes memory signature) internal view returns (bool) {
        bytes32 message = prefixed(keccak256(abi.encodePacked(address(this), amount)));

        // Check that the signature is from the payment sender.
        return recoverSigner(message, signature) == sender;
    }

    // revert when somebody is sending ether
    function () external {
        revert("you should not send ethereum to this address");
    }
}