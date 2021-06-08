// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

import "@openzeppelin/contracts/utils/math/SafeMath.sol";
import "@openzeppelin/contracts/utils/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/utils/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract HoprChannels is IERC777Recipient, ERC1820Implementer {
    using SafeMath for uint256;
    using SafeERC20 for IERC20;

    // required by ERC1820 spec
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    // required by ERC777 spec
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_MULTI_SIZE = abi.encode(address(0), address(0), uint256(0), uint256(0)).length;

    /**
     * @dev Possible channel statuses.
     */
    enum ChannelStatus { CLOSED, WAITING_FOR_COMMITMENT, OPEN, PENDING_TO_CLOSE }

    /**
     * @dev A channel struct, used to represent a channel's state
     */
    struct Channel {
        uint256 balance;
        bytes32 commitment;
        uint256 ticketEpoch;
        uint256 ticketIndex;
        ChannelStatus status;
        uint channelEpoch;
        // the time when the channel can be closed - NB: overloads at year >2105
        uint32 closureTime;
    }

    /**
     * @dev Stored channels keyed by their channel ids
     */
    mapping(bytes32 => Channel) public channels;

    /**
     * @dev HoprToken, the token that will be used to settle payments
     */
    IERC20 public token;

    /**
     * @dev Seconds it takes until we can finalize channel closure once,
     * channel closure has been initialized.
     */
    uint32 public secsClosure;

    event Announcement(
        address indexed account,
        bytes multiaddr
    );

    event ChannelUpdate(
        address indexed source,
        address indexed destination,
        Channel newState
    );

    /**
     * @param _token HoprToken address
     * @param _secsClosure seconds until a channel can be closed
     */
    constructor(address _token, uint32 _secsClosure) {
        token = IERC20(_token);
        secsClosure = _secsClosure;
        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    }

    /**
     * @dev Announces msg.sender's multiaddress.
     * Confirmation should be done off-chain.
     * @param multiaddr the multiaddress
     */
    function announce(bytes calldata multiaddr) external {
        emit Announcement(msg.sender, multiaddr);
    }

    /**
     * @dev Funds channels, in both directions, between 2 parties.
     * then emits {ChannelUpdate} event, for each channel.
     * @param account1 the address of account1
     * @param account2 the address of account2
     * @param amount1 amount to fund account1
     * @param amount2 amount to fund account2
     */
    function fundChannelMulti(
        address account1,
        address account2,
        uint256 amount1,
        uint256 amount2
    ) external {
        token.safeTransferFrom(msg.sender, address(this), amount1.add(amount2));
        _fundChannel(
            account1,
            account2,
            amount1
        );
        _fundChannel(
            account2,
            account1,
            amount2
        );
    }

    function redeemTicket(
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        uint256 winProb,
        bytes memory signature
    ) external {
        _redeemTicket(
            msg.sender,
            counterparty,
            nextCommitment,
            ticketEpoch,
            ticketIndex,
            proofOfRelaySecret,
            amount,
            winProb,
            signature
        );
    }

    /**
     * @dev Initialize channel closure.
     * When a channel owner (the 'source' of the channel) wants to 'cash out',
     * they must notify the counterparty (the 'destination') that they will do
     * so, and provide enough time for them to redeem any outstanding tickets
     * before-hand. This notice period is called the 'cool-off' period.
     * The channel 'destination' should be monitoring blockchain events, thus
     * they should be aware that the closure has been triggered, as this
     * method triggers a {ChannelUpdate} event.
     * After the cool-off period expires, the 'source' can call
     * 'finalizeChannelClosure' which withdraws the stake.
     * @param destination the address of the destination
     */
    function initiateChannelClosure(
        address destination
    ) external {
        _validateSourceAndDest(msg.sender, destination);
        (, Channel storage channel) = _getChannel(msg.sender, destination);
        require(channel.status == ChannelStatus.OPEN || channel.status == ChannelStatus.WAITING_FOR_COMMITMENT, "channel must be open or waiting for commitment");
        // @TODO: check with team, do we need SafeMath check here?
        channel.closureTime = _currentBlockTimestamp() + secsClosure;
        channel.status = ChannelStatus.PENDING_TO_CLOSE;
        emit ChannelUpdate(msg.sender, destination, channel);
    }

    /**
     * @dev Finalize the channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the sender. Then emits {ChannelUpdate} event.
     * @param destination the address of the counterparty
     */
    function finalizeChannelClosure(
        address destination
    ) external {
        _validateSourceAndDest(msg.sender, destination);
        require(address(token) != address(0), "token must not be empty");
        (, Channel storage channel) = _getChannel(msg.sender, destination);
        require(channel.status == ChannelStatus.PENDING_TO_CLOSE, "channel must be pending to close");
        require(channel.closureTime < _currentBlockTimestamp(), "closureTime must be before now");

        if (channel.balance > 0) {
          token.transfer(msg.sender, channel.balance);
        }

        delete channel.balance;
        delete channel.closureTime; // channel.closureTime = 0
        channel.status = ChannelStatus.CLOSED;
        emit ChannelUpdate(msg.sender, destination, channel);
    }

    /**
    * @dev Request a channelIteration bump, so we can make a new set of
    * commitments
    * Implies that msg.sender is the destination of the channel.
    * @param source the address of the channel source
    * @param newCommitment, a secret derived from this new commitment
    */
    function bumpChannel(
      address source,
      bytes32 newCommitment
    ) external {
        _validateSourceAndDest(source, msg.sender);

        (, Channel storage channel) = _getChannel(
            source,
            msg.sender
        );

        channel.commitment = newCommitment;
        channel.ticketEpoch = channel.ticketEpoch.add(1);
        emit ChannelUpdate(source, msg.sender, channel);
    }

    /**
     * A hook triggered when HOPR tokens are sent to this contract.
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
            userData.length == FUND_CHANNEL_MULTI_SIZE,
            "userData must match one of our supported functions"
        );

        address account1;
        address account2;
        uint256 amount1;
        uint256 amount2;

        (account1, account2, amount1, amount2) = abi.decode(userData, (address, address, uint256, uint256));
        require(amount == amount1.add(amount2), "amount sent must be equal to amount specified");

        //require(from == account1 || from == account2, "funder must be either account1 or account2");
        _fundChannel(account1, account2, amount1);
        _fundChannel(account2, account1, amount2);
    }

    // internal code

    /**
     * @dev Funds a channel, then emits
     * {ChannelUpdate} event.
     * @param source the address of the channel source
     * @param dest the address of the channel destination
     * @param amount amount to fund account1
     */
    function _fundChannel(
        address source,
        address dest,
        uint256 amount
    ) internal {
        _validateSourceAndDest(source, dest);
        require(amount > 0, "amount must be greater than 0");

        (, Channel storage channel) = _getChannel(source, dest);
        require(channel.status != ChannelStatus.PENDING_TO_CLOSE, "Cannot fund a closing channel"); 
        if (channel.status == ChannelStatus.CLOSED) {
          // We are reopening the channel
          channel.channelEpoch = channel.channelEpoch.add(1);
          channel.status = ChannelStatus.WAITING_FOR_COMMITMENT;
          channel.ticketIndex = 0;
        }

        channel.balance = channel.balance.add(amount);
        emit ChannelUpdate(source, dest, channel);
    }


    /**
     * @param source source
     * @param destination destination
     * @return a tuple of channelId, channel
     */
    function _getChannel(address source, address destination)
        internal
        view
        returns (
            bytes32,
            Channel storage
        )
    {
        bytes32 channelId = _getChannelId(source, destination);
        Channel storage channel = channels[channelId];
        return (channelId, channel);
    }

    /**
     * @param source the address of source
     * @param destination the address of destination
     * @return the channel id 
     */
    function _getChannelId(address source, address destination) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(source, destination));
    }

    /**
     * @return the current timestamp
     */
    function _currentBlockTimestamp() internal view returns (uint32) {
        // solhint-disable-next-line
        return uint32(block.timestamp % 2 ** 32);
    }

    /**
     * @dev Redeem a ticket
     * @param redeemer the redeemer address
     * @param counterparty the counterparty address
     * @param nextCommitment the commitment that hashes to the redeemers previous commitment
     * @param proofOfRelaySecret the proof of relay secret
     * @param winProb the winning probability of the ticket
     * @param amount the amount in the ticket
     * @param signature signature
     */
    function _redeemTicket(
        address redeemer,
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        uint256 winProb,
        bytes memory signature
    ) internal {
        require(redeemer != address(0), "redeemer must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");
        require(nextCommitment != bytes32(0), "nextCommitment must not be empty");
        require(amount != uint256(0), "amount must not be empty");
        (, Channel storage earningChannel) = _getChannel(
            redeemer,
            counterparty
        );
        (, Channel storage spendingChannel) = _getChannel(
            counterparty,
            redeemer
        );
        require(earningChannel.status != ChannelStatus.CLOSED, "earning channel must be open or pending to close");
        uint256 prevTicketEpoch;
        require(earningChannel.commitment == keccak256(abi.encodePacked(nextCommitment)), "commitment must be hash of next commitment");
        require(earningChannel.ticketEpoch == ticketEpoch, "ticket epoch must match");
        require(earningChannel.ticketIndex < ticketIndex, "redemptions must be in order");
        prevTicketEpoch = earningChannel.ticketEpoch;

        bytes32 ticketHash = ECDSA.toEthSignedMessageHash(
            keccak256(
              _getEncodedTicket(
                  redeemer,
                  prevTicketEpoch,
                  proofOfRelaySecret,
                  earningChannel.channelEpoch,
                  amount,
                  ticketIndex,
                  winProb
              )
            )
        );

        require(ECDSA.recover(ticketHash, signature) == counterparty, "signer must match the counterparty");
        require(
            _getTicketLuck(
                ticketHash,
                nextCommitment,
                proofOfRelaySecret
            ) <= winProb,
            "ticket must be a win"
        );

          earningChannel.commitment = nextCommitment;
          spendingChannel.balance = spendingChannel.balance.sub(amount);
          earningChannel.balance = earningChannel.balance.add(amount);
          earningChannel.ticketIndex = ticketIndex;
          emit ChannelUpdate(redeemer, counterparty, earningChannel);
    }


    /**
    * Assert that source and dest are good addresses, and distinct.
    */
    function _validateSourceAndDest (address source, address dest) internal {
      require(source != dest, "source and dest must not be the same");
      require(source != address(0), "source must not be empty");
      require(dest != address(0), "dest must not be empty");
    }

    /**
     * Uses the response to recompute the challenge. This is done
     * by multiplying the base point of the curve with the given response.
     * Due to the lack of embedded ECMUL functionality in the current
     * version of the EVM, this is done by misusing the `ecrecover` 
     * functionality. `ecrecover` performs the point multiplication and 
     * converts the output to an Ethereum address (sliced hash of the product
     * of base point and scalar).
     * See https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
     * @param response response that is used to recompute the challenge
     */
    function _computeChallenge(bytes32 response) internal pure returns (address)  {
        // Field order of the base field
        uint256 FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

        // x-coordinate of the base point
        uint256 gx = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;
        // y-coordinate of base-point is even, so v is 27
        uint8 gv = 27;

        address signer = ecrecover(0, gv, bytes32(gx), bytes32(mulmod(uint256(response), gx, FIELD_ORDER)));

        return signer;
    }
    /**
     * @dev Encode ticket data
     * @return bytes
     */
    function _getEncodedTicket(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        uint256 ticketIndex,
        uint256 winProb
    ) internal pure returns (bytes memory) {
        address challenge = _computeChallenge(proofOfRelaySecret);

        return abi.encodePacked(
            recipient,
            challenge,
            recipientCounter,
            amount,
            winProb,
            ticketIndex,
            channelIteration
        );
    }
    
    /**
     * @dev Get the ticket's "luck" by
     * hashing provided values.
     * @return luck
     */
    function _getTicketLuck(
        bytes32 ticketHash,
        bytes32 nextCommitment,
        bytes32 proofOfRelaySecret
    ) internal pure returns (uint256) {
        return uint256(keccak256(abi.encodePacked(ticketHash, nextCommitment, proofOfRelaySecret)));
    }
}
