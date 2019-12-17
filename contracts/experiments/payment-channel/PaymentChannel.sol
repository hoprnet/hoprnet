pragma solidity ^0.5.3;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/utils/Address.sol";

contract PaymentChannel {
    using SafeERC20 for IERC20;
    using SafeMath for uint256;
    using Address for address;

    event OpenedChannel(uint256 indexed channelId, uint256 depositedAmount);
    event ClosedChannel(uint256 indexed channelId, uint256 senderAmount, uint256 receiverAmount);

    enum ChannelStatus {
        OPEN,
        CLOSED
    }

    struct Channel {
        uint256 id;             //  Channel ID
        address sender;         //  The account sending payments
        address recipient;      //  The account receiving the payments
        IERC20 token;           //  The token that will be used
        uint256 deposit;        //  Token deposit amount
        uint256 expiration;     //  Timeout in case the recipient never closes
        ChannelStatus status;   //  Channel status
    }
    uint256 public numberOfChannels = 0;

    mapping(uint256 => Channel) public channels;

    // Ensure channel exists
    modifier channelMustExist(uint256 channelId) {
        require(channels[channelId].id == channelId, "channel does not exist");
        _;
    }

    // Create a channel, specified tokens must be approved beforehand
    function createChannel(address _recipient, address _token, uint256 amount, uint256 duration)
    external returns(uint256) {
        uint256 channelId = numberOfChannels.add(1);

        require(channels[channelId].id == 0, "channel already exists");
        require(_token != address(0), "address is empty");
        require(amount > 0, "amount must be larger than 0");
        require(duration > 0, "duration must be larger than 0");

        IERC20 token = IERC20(_token);
        token.safeTransferFrom(msg.sender, address(this), amount);
        numberOfChannels = channelId;

        channels[channelId] = Channel(
            channelId,
            msg.sender,
            _recipient,
            token,
            amount,
            now + duration,
            ChannelStatus.OPEN
        );

        emit OpenedChannel(channelId, amount);

        return channelId;
    }

    // The recipient can close the channel at any time by presenting a signed
    // amount from the sender. The recipient will be sent that amount, and the
    // remainder will go back to the sender.
    function closeChannel(uint256 channelId, uint256 amount, bytes calldata signature)
    external channelMustExist(channelId) {
        Channel storage channel = channels[channelId];

        require(msg.sender == channel.recipient, "caller is not channel's recipient");
        require(isValidSignature(channel.sender, amount, signature), "signature is not valid");

        settle(channel, amount);
    }

    // The sender can extend the expiration at any time.
    function extendChannelExpiration(uint256 channelId, uint256 newExpiration)
    external channelMustExist(channelId) {
        Channel storage channel = channels[channelId];

        require(msg.sender == channel.sender, "caller is not sender");
        require(newExpiration > channel.expiration, "new expiration is smaller than current expiration");

        channel.expiration = newExpiration;
    }

    // If the timeout is reached without the recipient closing the channel, then
    // the ether is released back to the sender.
    function claimChannelTimeout(uint256 channelId)
    external channelMustExist(channelId) {
        Channel storage channel = channels[channelId];

        require(now >= channel.expiration, "channel has not expired");

        settle(channel, 0);
    }

    // Settle channel, send 'amount' to recipient and the rest to sender
    function settle(Channel storage channel, uint256 amount) internal {
        if (amount > 0) {
            channel.token.safeTransfer(channel.recipient, amount);
            channel.deposit = channel.deposit.sub(amount);
        }

        uint256 remaining = channel.deposit;
        if (remaining > 0) {
            channel.token.safeTransfer(channel.sender, remaining);
        }

        channel.deposit = 0;
        channel.status = ChannelStatus.CLOSED;
        emit ClosedChannel(channel.id, remaining, amount);
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

    // Builds a prefixed hash to mimic the behavior of eth_sign.
    function prefixed(bytes32 hash) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash));
    }

    function isValidSignature(address sender, uint256 amount, bytes memory signature) internal view returns (bool) {
        bytes32 message = prefixed(keccak256(abi.encodePacked(address(this), amount)));

        // Check that the signature is from the payment sender.
        return recoverSigner(message, signature) == sender;
    }

    // Revert when somebody is sending ether
    function () external {
        revert();
    }
}