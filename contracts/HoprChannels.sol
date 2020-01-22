pragma solidity ^0.5.3;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";

contract HoprChannels {
    using SafeERC20 for IERC20;
    using SafeMath for uint256;

    IERC20 public token;        // the token that will be used to settle payments
    uint256 public secsClosure; // seconds it takes to allow closing of channel after channel's -
                                // 'sender' provided a signature

    // the payment channel has been created and opened
    event OpenedChannel(
        address indexed funder,
        address indexed sender,
        address indexed recipient,
        uint256 deposit
    );

    // TODO: check with Sebastian if we need this
    // recipient has reedemed a ticket
    // event TickedReedemed(
    //     address indexed sender,
    //     address indexed recipient,
    //     uint256 amount
    // );

    // recipient withdrawed unsettled channel balance
    event Withdrawed(
        address indexed sender,
        address indexed recipient,
        uint256 recipientAmount
    );

    // the payment channel's 'sender' is closing the channel
    event InitiatedChannelClosure(
        address indexed sender,
        address indexed recipient,
        uint256 closureTime
    );

    // the payment channel has been settled and closed
    event ClosedChannel(
        address indexed sender,
        address indexed recipient,
        uint256 senderAmount,
        uint256 recipientAmount
    );

    struct Channel {
        uint256 deposit;        // tokens deposited
        uint256 unsettled;      // tokens that are claimable but not yet settled
        uint256 closureTime;    // the time when the channel can be closed by 'sender'
        bool isOpen;            // channel is open
    }

    // store channels e.g: channels[sender][recipient]
    mapping(address => mapping(address => Channel)) public channels;
    mapping(address => bytes32) public hashedSecrets;  

    constructor(IERC20 _token, uint256 _secsClosure) public {
        token = _token;
        secsClosure = _secsClosure;
    }

    /**
     * @notice sets caller's hashedSecret
     * @param hashedSecret bytes32 hashedSecret to store
     */
    // TODO: check with Robert if this is ok
    function setHashedSecret(bytes32 hashedSecret) external {
        require(hashedSecrets[msg.sender] != hashedSecret, "new and old hashedSecret must not be the same");

        hashedSecrets[msg.sender] = hashedSecret;
    }

    /**
     * Create and open a channel between 'sender' and 'recipient',
     * specified tokens must be approved beforehand.
     *
     * @notice create and open a channel
     * @param funder address account which funds the channel
     * @param sender address account which owns the channel
     * @param recipient address account which receives payments
     * @param amount uint256 amount to fund the channel
     */
    function createChannel(address funder, address sender, address recipient, uint256 amount) external {
        require(funder != address(0), "'funder' address is empty");
        require(sender != address(0), "'sender' address is empty");
        require(recipient != address(0), "'recipient' address is empty");
        require(hashedSecrets[recipient] != bytes32(0), "'recipient' has not set a hashed secret");
        require(amount > 0, "'amount' must be greater than 0");

        Channel storage channel = channels[sender][recipient];
        require(channel.isOpen == false, "channel is not closed");

        token.safeTransferFrom(funder, address(this), amount);

        channel.deposit = amount;
        channel.isOpen = true;

        emit OpenedChannel(funder, sender, recipient, amount);
    }

    /**
     * @notice redeem ticket
     * @param sender address account that created the channel
     * @param pre_image bytes32 the value that once hashed produces recipients hashedSecret
     * @param s_a bytes32 secret
     * @param s_b bytes32 secret
     * @param amount uint256 amount recipient will receive
     * @param win_prob bytes32 win probability
     * @param signature bytes recipient's signature
     */
    function redeemTicket(
        address sender,
        bytes32 pre_image,
        bytes32 s_a,
        bytes32 s_b,
        uint256 amount,
        bytes32 win_prob,
        bytes memory signature
    ) public {
        address recipient = msg.sender;
        bytes32 hashedSecret = hashedSecrets[recipient];
        Channel storage channel = channels[sender][recipient];

        require(
            channel.isOpen ||
            isChannelPendingClosure(channel),
            "channel must be 'open' or 'pending for closure'"
        );
        require(amount > 0, "amount must be strictly greater than zero");
        require(
            hashedSecret == keccak256(abi.encodePacked(pre_image)),
            "given value is not a pre-image of the stored on-chain secret"
        );

        bytes32 hashed_s_a = keccak256(abi.encodePacked(s_a));
        bytes32 hashed_s_b = keccak256(abi.encodePacked(s_b));
        bytes32 challange = keccak256(abi.encodePacked(hashed_s_a, hashed_s_b));
        bytes32 hashedTicket = keccak256(abi.encodePacked(challange, hashedSecret, amount, win_prob));

        // TODO: implement xor
        require(uint256(hashedTicket) < uint256(win_prob), "ticket must be a win");
        require(recoverSigner(hashedTicket, signature) == recipient, "signature must be valid");

        hashedSecrets[recipient] = pre_image;
        channel.unsettled = channel.unsettled.add(amount);

        require(channel.unsettled <= channel.deposit, "unsettled balance must be strictly lesser than deposit balance");
    }

    /**
     * Close a channel between 'sender' and 'recipient',
     * the recipient can close the channel at any time.
     * 
     * The recipient will be sent the unsettled balance,
     * and the remainder will go back to the sender.
     *
     * @notice close channel and settle payment
     * @param sender address account that created the channel
    */
    function closeChannel(address sender) public {
        require(sender != address(0), "'sender' address is empty");

        settle(sender, msg.sender);
    }

    /**
     * @notice redeem ticket and close channel
     * @param sender address account that created the channel
     * @param pre_image bytes32 the value that once hashed produces recipients hashedSecret
     * @param s_a bytes32 secret
     * @param s_b bytes32 secret
     * @param amount uint256 amount recipient will receive
     * @param win_prob bytes32 win probability
     * @param signature bytes recipient's signature
     */
    function redeemTicketAndCloseChannel(
        address sender,
        bytes32 pre_image,
        bytes32 s_a,
        bytes32 s_b,
        uint256 amount,
        bytes32 win_prob,
        bytes calldata signature
    ) external {
        redeemTicket(
            sender,
            pre_image,
            s_a,
            s_b,
            amount,
            win_prob,
            signature
        );
        closeChannel(sender);
    }

    /**
     * @notice withdraw unsettled balance
     * @param sender address account which owns the channel
     */
    function withdraw(address sender) external {
        Channel storage channel = channels[sender][msg.sender];

        if (channel.unsettled > 0) {
            token.safeTransfer(msg.sender, channel.unsettled);
            channel.deposit = channel.deposit.sub(channel.unsettled);

            emit Withdrawed(sender, msg.sender, channel.unsettled);

            channel.unsettled = 0;
        }
    }

    /**
     * The 'sender' can initiate channel closure at any time,
     * it starts a timeout.
     *
     * @notice initiate channel's closure
     * @param recipient address account which will receive the payment
     */
    function initiateChannelClosure(address recipient) external {
        Channel storage channel = channels[msg.sender][recipient];
        require(channel.isOpen, "channel is not open");

        channel.closureTime = now + secsClosure;

        emit InitiatedChannelClosure(msg.sender, recipient, channel.closureTime);
    }

    /**
     * If the timeout is reached without the recipient providing a signature,
     * then the tokens can be claimed by 'sender'.
     *
     * @notice claim channel's closure
     * @param recipient address the recipient account
     */
    function claimChannelClosure(address recipient) external {
        Channel storage channel = channels[msg.sender][recipient];

        require(
            channel.isOpen &&
            isChannelPendingClosure(channel),
            "channel is not pending for closure"
        );
        require(now >= channel.closureTime, "'closureTime' has not passed");

        settle(msg.sender, recipient);
    }

    /**
     * Settle channel, send 'amount' to recipient and the rest to sender.
     *
     * @notice settle channel
     * @param sender address account which owns the channel
     * @param recipient address account which receives payments
     */
    function settle(address sender, address recipient) internal {
        Channel storage channel = channels[sender][recipient];

        if (channel.unsettled > 0) {
            token.safeTransfer(recipient, channel.unsettled);
            channel.deposit = channel.deposit.sub(channel.unsettled);
        }

        if (channel.deposit > 0) {
            token.safeTransfer(sender, channel.deposit);
        }

        emit ClosedChannel(sender, recipient, channel.deposit, channel.unsettled);

        channel.deposit = 0;
        channel.unsettled = 0;
        channel.closureTime = 0;
        channel.isOpen = false;
    }

    /// return 'true' if channel is pending for closure
    function isChannelPendingClosure(Channel memory channel) internal pure returns (bool) {
        return channel.closureTime > 0;
    }

    // TODO: check if this works
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
}