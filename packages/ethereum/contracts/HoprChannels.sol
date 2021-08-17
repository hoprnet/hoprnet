// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

import "@openzeppelin/contracts/utils/Multicall.sol";
import "@openzeppelin/contracts/utils/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/utils/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract HoprChannels is IERC777Recipient, ERC1820Implementer, Multicall {
    using SafeERC20 for IERC20;

    // required by ERC1820 spec
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    // required by ERC777 spec
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_MULTI_SIZE = abi.encode(address(0), address(0), uint256(0), uint256(0)).length;

    /**
     * @dev Possible channel statuses.

            Finalize Closure
            (After delay)+----------------+   Initiate Closure
                 +-------+Pending To Close| <---------+
                 |       +----------------+           |
                 |                                    |
              +--v---+                             +--+-+
              |Closed+---------------------------->+Open|
              +---+--+ Fund (If already committed) +--+-+
                  |                                   ^
             Fund |                                   | Bump Commitment
                  |       +----------------------+    |
                  +------>+Waiting For Commitment+----+
                          +----------------------+

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
        uint256 channelEpoch;
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

    event ChannelClosureInitiated(
        address indexed source,
        address indexed destination,
        uint32 closureInitiationTime
    );

    event ChannelClosureFinalized(
        address indexed source,
        address indexed destination,
        uint32 closureFinalizationTime,
        uint256 channelBalance
    );

    event ChannelBumped(
        address indexed source,
        address indexed destination,
        bytes32 newCommitment,
        uint256 ticketEpoch,
        uint256 channelBalance
    );

    event TicketRedeemed(
        address indexed source,
        address indexed destination,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        uint256 winProb,
        bytes signature
    );

    event TokensReceived(
        address indexed from,
        address indexed account1,
        address indexed account2,
        uint256 amount1,
        uint256 amount2
    );

    /**
     * @param _token HoprToken address
     * @param _secsClosure seconds until a channel can be closed
     */
    constructor(address _token, uint32 _secsClosure) {
        require(_token != address(0), "token must not be empty");

        token = IERC20(_token);
        secsClosure = _secsClosure;
        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    }

    /**
    * Assert that source and destination are good addresses, and distinct.
    */
    modifier validateSourceAndDest(address source, address destination) {
      require(source != destination, "source and destination must not be the same");
      require(source != address(0), "source must not be empty");
      require(destination != address(0), "destination must not be empty");
      _;
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
        require(amount1 + amount2 > 0, "amount must be greater than 0");
        token.safeTransferFrom(msg.sender, address(this), amount1 + amount2);
        if (amount1 > 0){
          _fundChannel(account1, account2, amount1);
        }
        if (amount2 > 0){
          _fundChannel(account2, account1, amount2);
        }
    }

    /**
    * @dev redeem a ticket.
    * If the sender has a channel to the source, the amount will be transferred
    * to that channel, otherwise it will be sent to their address directly.
    * @param source the source of the ticket
    * @param nextCommitment the commitment that hashes to the redeemers previous commitment
    * @param proofOfRelaySecret the proof of relay secret
    * @param winProb the winning probability of the ticket
    * @param amount the amount in the ticket
    * @param signature signature
    */
    function redeemTicket(
        address source,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        uint256 winProb,
        bytes memory signature
    ) external validateSourceAndDest(source, msg.sender) {
        require(nextCommitment != bytes32(0), "nextCommitment must not be empty");
        require(amount != uint256(0), "amount must not be empty");
        (, Channel storage spendingChannel) = _getChannel(
            source,
            msg.sender
        );
        require(spendingChannel.status == ChannelStatus.OPEN || spendingChannel.status == ChannelStatus.PENDING_TO_CLOSE, "spending channel must be open or pending to close");
        require(spendingChannel.commitment == keccak256(abi.encodePacked(nextCommitment)), "commitment must be hash of next commitment");
        require(spendingChannel.ticketEpoch == ticketEpoch, "ticket epoch must match");
        require(spendingChannel.ticketIndex < ticketIndex, "redemptions must be in order");

        bytes32 ticketHash = ECDSA.toEthSignedMessageHash(
            keccak256(
              _getEncodedTicket(
                  msg.sender,
                  spendingChannel.ticketEpoch,
                  proofOfRelaySecret,
                  spendingChannel.channelEpoch,
                  amount,
                  ticketIndex,
                  winProb
              )
            )
        );

        require(ECDSA.recover(ticketHash, signature) == source, "signer must match the counterparty");
        require(
            _getTicketLuck(
                ticketHash,
                nextCommitment,
                proofOfRelaySecret
            ) <= winProb,
            "ticket must be a win"
        );

          spendingChannel.ticketIndex = ticketIndex;
          spendingChannel.commitment = nextCommitment;
          spendingChannel.balance = spendingChannel.balance - amount;
          (, Channel storage earningChannel) = _getChannel(
              msg.sender,
              source
          );
          if (earningChannel.status == ChannelStatus.OPEN) {
            earningChannel.balance = earningChannel.balance + amount;
            emit ChannelUpdate(msg.sender, source, earningChannel);
          } else {
            token.safeTransfer(msg.sender, amount);
          }
          emit ChannelUpdate(source, msg.sender, spendingChannel);
          emit TicketRedeemed(source, msg.sender, nextCommitment, ticketEpoch, ticketIndex, proofOfRelaySecret, amount, winProb, signature);
    }


    /**
     * @dev Initialize channel closure.
     * When a channel owner (the 'source' of the channel) wants to 'cash out',
     * they must notify the counterparty (the 'destination') that they will do
     * so, and provide enough time for them to redeem any outstanding tickets
     * before-hand. This notice period is called the 'cool-off' period.
     * The channel 'destination' should be monitoring blockchain events, thus
     * they should be aware that the closure has been triggered, as this
     * method triggers a {ChannelUpdate} and an {ChannelClosureInitiated} event.
     * After the cool-off period expires, the 'source' can call
     * 'finalizeChannelClosure' which withdraws the stake.
     * @param destination the address of the destination
     */
    function initiateChannelClosure(
        address destination
    ) external validateSourceAndDest(msg.sender, destination) {
        (, Channel storage channel) = _getChannel(msg.sender, destination);
        require(channel.status == ChannelStatus.OPEN || channel.status == ChannelStatus.WAITING_FOR_COMMITMENT, "channel must be open or waiting for commitment");
        channel.closureTime = _currentBlockTimestamp() + secsClosure;
        channel.status = ChannelStatus.PENDING_TO_CLOSE;
        emit ChannelUpdate(msg.sender, destination, channel);
        emit ChannelClosureInitiated(msg.sender, destination, _currentBlockTimestamp());
    }

    /**
     * @dev Finalize the channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the sender. Then emits {ChannelUpdate} and the
     * {ChannelClosureFinalized} event.
     * @param destination the address of the counterparty
     */
    function finalizeChannelClosure(
        address destination
    ) external validateSourceAndDest(msg.sender, destination) {
        (, Channel storage channel) = _getChannel(msg.sender, destination);
        require(channel.status == ChannelStatus.PENDING_TO_CLOSE, "channel must be pending to close");
        require(channel.closureTime < _currentBlockTimestamp(), "closureTime must be before now");

        if (channel.balance > 0) {
          token.transfer(msg.sender, channel.balance);
        }

        emit ChannelClosureFinalized(msg.sender, destination, channel.closureTime, channel.balance);
        delete channel.balance;
        delete channel.closureTime;
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
    ) external validateSourceAndDest(source, msg.sender) {
        (, Channel storage channel) = _getChannel(
            source,
            msg.sender
        );

        require(newCommitment != bytes32(0), "Cannot set empty commitment");
        channel.commitment = newCommitment;
        channel.ticketEpoch = channel.ticketEpoch + 1;
        if (channel.status == ChannelStatus.WAITING_FOR_COMMITMENT){
          channel.status = ChannelStatus.OPEN;
        }
        emit ChannelUpdate(source, msg.sender, channel);
        emit ChannelBumped(source, msg.sender, newCommitment, channel.ticketEpoch, channel.balance);
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
        require(to == address(this), "must be sending tokens to HoprChannels");

        // must be one of our supported functions
        if (userData.length == FUND_CHANNEL_MULTI_SIZE) {
            address account1;
            address account2;
            uint256 amount1;
            uint256 amount2;

            (account1, account2, amount1, amount2) = abi.decode(userData, (address, address, uint256, uint256));
            require(amount == amount1 + amount2, "amount sent must be equal to amount specified");

            if (amount1 > 0){
                _fundChannel(account1, account2, amount1);
            }
            if (amount2 > 0){
                _fundChannel(account2, account1, amount2);
            }
            emit TokensReceived(from, account1, account2, amount1, amount2);
        }
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
    ) internal validateSourceAndDest(source, dest) {
        require(amount > 0, "amount must be greater than 0");

        (, Channel storage channel) = _getChannel(source, dest);
        require(channel.status != ChannelStatus.PENDING_TO_CLOSE, "Cannot fund a closing channel"); 
        if (channel.status == ChannelStatus.CLOSED) {
          // We are reopening the channel
          channel.channelEpoch = channel.channelEpoch + 1;
          channel.ticketEpoch = 0; // As we've incremented the channel epoch, we can restart the ticket counter
          channel.ticketIndex = 0;

          if (channel.commitment != bytes32(0)) {
            channel.status = ChannelStatus.OPEN;
          } else {
            channel.status = ChannelStatus.WAITING_FOR_COMMITMENT;
          }
        }

        channel.balance = channel.balance + amount;
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
        return uint32(block.timestamp);
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

        require(0 < uint256(response), "Invalid response. Value must be within the field");
        require(uint256(response) < FIELD_ORDER, "Invalid response. Value must be within the field");

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
