// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.19;

import 'openzeppelin-contracts-4.8.3/utils/Multicall.sol';
import 'openzeppelin-contracts-4.8.3/utils/introspection/IERC1820Registry.sol';
import 'openzeppelin-contracts-4.8.3/utils/introspection/ERC1820Implementer.sol';
import 'openzeppelin-contracts-4.8.3/token/ERC20/IERC20.sol';
import 'openzeppelin-contracts-4.8.3/token/ERC777/IERC777Recipient.sol';
import 'openzeppelin-contracts-4.8.3/utils/cryptography/ECDSA.sol';

error InvalidBalance();
error BalanceExceedsGlobalPerChannelAllowance();

error SourceEqualsDestination();
error ZeroAddress(string reason);

error TokenTransferFailed();
error InvalidNoticePeriod();
error NoticePeriodNotDue();

error WrongChannelState(string reason);

error InvalidPoRSecret(string reason);
error InvalidTicketSignature();
error InvalidCommitmentOpening();
error InsufficientChannelBalance();
error InvalidCommitment();
error TicketIsNotAWin();

/**
 *      &&&&
 *      &&&&
 *      &&&&
 *      &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *      &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *       &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *       &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *       &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *       %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *      %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                            %%%%
 *                                            %%%%
 *                                            %%%%
 *
 * Manages mixnet incentives in the hopr network.
 **/
contract HoprChannels is IERC777Recipient, ERC1820Implementer, Multicall {
  // required by ERC1820 spec
  IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
  // required by ERC777 spec
  bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256('ERC777TokensRecipient');

  type Balance is uint96;
  type TicketEpoch is uint32;
  type TicketIndex is uint64;
  type ChannelEpoch is uint24;
  type Timestamp is uint32; // overflows in year 2105
  // Using IEEE 754 double precision -> 53 significant bits
  type WinProb is uint56;

  Balance public constant MAX_BALANCE_PER_CHANNEL = 10 ** 25; // 1% of total supply
  Balance public constant MIN_BALANCE_PER_CHANNEL = 1; // no empty token transactions

  // Field order created by secp256k1 curve
  uint256 constant FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

  // x-component of base point of secp256k1 curve
  uint256 constant BASE_POINT_X_COMPONENT = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;

  // encoded sign of y-component of base point of secp256k1 curve
  uint8 constant BASE_POINT_Y_COMPONENT_SIGN = 27;

  // used by {tokensReceived} to distinguish which function to call after tokens are sent
  uint256 public immutable FUND_CHANNEL_MULTI_SIZE = abi.encode(address(0), Balance(0), address(0), Balance(0)).length;

  string public immutable version = '2.0.0';

  bytes32 public immutable domainSeparator;
  bytes32 public immutable redeemTicketSeparator;

  /**
   * @dev Possible channel states.
   *
   * finalizeChannelClosure()    ┌──────────────────────┐
   *  (after notice period)      │                      │ initiateChannelClosure()
   *            ┌────────────────│   Pending To Close   │<─────────────────┐
   *            │                │                      │                  │
   *            │                └──────────────────────┘                  │
   *            v                                                          │
   *     ┌────────────┐                                               ┌──────────┐
   *     │            │              tokensReceived()                 │          │
   *     │   Closed   │──────────────────────────────────────────────>│   Open   │
   *     │            │                                               │          │
   *     └────────────┘                                               └──────────┘
   */
  enum ChannelStatus {
    CLOSED,
    OPEN,
    PENDING_TO_CLOSE
  }

  /**
   * Holding a compact ECDSA signature, following ERC-2098
   */
  struct CompactSignature {
    bytes32 r;
    bytes32 vs;
  }

  /**
   * Represents the state of channel
   */
  struct Channel {
    // iterated commitment, most recent opening gets revealed when
    // redeeming a ticket
    bytes32 commitment;
    // latest balance of the channel, changes whenever a ticket gets redeemed
    Balance balance;
    TicketIndex ticketIndex;
    ChannelStatus status;
    ChannelEpoch channelEpoch;
    // the time when the channel can be closed - NB: overloads at year >2105
    Timestamp closureTime;
  }

  /**
   * Represents a ticket that can be redeemed using `redeemTicket` function.
   *
   * Aligned to 2 EVM words
   */
  struct Ticket {
    bytes32 channelId;
    Balance amount;
    TicketIndex ticketIndex;
    ChannelEpoch channelEpoch;
    WinProb winProb;
    uint16 resered; // for future use
  }

  /**
   * @dev Stored channels keyed by their channel ids
   */
  mapping(bytes32 => Channel) public channels;

  /**
   * @dev HoprToken, the token that will be used to settle payments
   */
  IERC20 public immutable token;

  /**
   * @dev Seconds it takes until we can finalize channel closure once,
   * channel closure has been initialized.
   */
  Timestamp public immutable noticePeriodChannelClosure;

  /**
   * Emitted once a channel if funded.
   */
  event ChannelFunded(address indexed funder, address indexed source, address indexed destination, Balance amount);

  /**
   * Emitted once balance of a channel is increased.
   */
  event ChannelBalanceIncreased(address indexed source, address indexed destination, Balance newBalance);

  /**
   * Emitted once balance of a channel is decreased.
   */
  event ChannelBalanceDecreased(address indexed source, address indexed destination, Balance newBalance);

  /**
   * Emitted once bumpChannel is called.
   */
  event ChannelBumped(
    address indexed source,
    address indexed destination,
    bytes32 newCommitment,
    TicketEpoch ticketEpoch,
    Balance channelBalance
  );

  /**
   * Emitted once a channel closure is initialized.
   */
  event ChannelClosureInitiated(address indexed source, address indexed destination, Timestamp closureInitiationTime);

  /**
   * Emitted once a channel closure is finalized.
   */
  event ChannelClosureFinalized(
    address indexed source,
    address indexed destination,
    Timestamp closureFinalizationTime,
    Balance channelBalance
  );

  /**
   * Emitted once a ticket is redeemed.
   */
  event TicketRedeemed(
    address indexed source,
    address indexed destination,
    bytes32 nextCommitment,
    TicketEpoch ticketEpoch,
    TicketIndex ticketIndex,
    bytes32 proofOfRelaySecret,
    Balance amount,
    uint256 winProb
  );

  /**
   * @param _token HoprToken address
   * @param _secsClosure seconds until a channel can be closed
   */
  constructor(address _token, Timestamp _secsClosure) {
    if (_secsClosure == Timestamp(0)) {
      revert InvalidNoticePeriod();
    }

    require(_token != address(0), 'token must not be empty');

    token = IERC20(_token);
    noticePeriodChannelClosure = _secsClosure;
    _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));

    domainSeparator = keccak256(
      abi.encode(
        keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)'),
        keccak256(bytes('NodeStakeRegistry')),
        keccak256(bytes(version)),
        block.chainid,
        address(this)
      )
    );

    // Non-standard usage of EIP712 due computed property and custom property encoding
    redeemTicketSeparator = keccak256(
      'Ticket(bytes32 channelId,uint96 balance,uint64 ticketIndex,uint24 channelEpoch,uint56 winProb,address porChallenge)'
    );
  }

  /**
   * Assert that source and destination are good addresses, and distinct.
   */
  modifier validateChannelParties(address source, address destination) {
    if (source == destination) {
      revert SourceEqualsDestination();
    }
    if (source == address(0)) {
      revert ZeroAddress({reason: 'source must not be empty'});
    }
    if (destination == address(0)) {
      revert ZeroAddress({reason: 'destination must not be empty'});
    }
    _;
  }

  modifier validateBalance(Balance balance) {
    if (balance == Balance(0)) {
      revert InvalidBalance();
    }
    if (balance > Balance(10 ** 25)) {
      revert BalanceExceedsGlobalPerChannelAllowance();
    }
    _;
  }

  modifier validatePoRSecret(bytes32 response) {
    if (response == uint256(0)) {
      revert InvalidPoRSecret('Response is 0. Value must be within the field');
    }

    if (response >= FIELD_ORDER) {
      revert InvalidPoRSecret('Response greater than field order. Value must be within the field');
    }
    _;
  }

  /**
   * @dev Funds channels, in both directions, between 2 parties.
   * then emits {ChannelUpdated} event, for each channel.
   * @param account1 the address of account1
   * @param amount1 amount to fund account1
   * @param account2 the address of account2
   * @param amount2 amount to fund account2
   */
  function fundChannelMulti(
    address account1,
    Balance amount1,
    address account2,
    Balance amount2
  ) external validateBalance(amount1) validateBalance(amount2) validateChannelParties(account1, account2) {
    // pull tokens from funder and handle result
    if (token.transferFrom(msg.sender, address(this), amount1 + amount2) != true) {
      // sth. went wrong
      revert TokenTransferFailed();
    }

    // fund channel in direction of: account1 -> account2
    if (amount1 > 0) {
      _fundChannel(account1, account2, amount1);
    }
    // fund channel in direction of: account2 -> account1
    if (amount2 > 0) {
      _fundChannel(account2, account1, amount2);
    }
  }

  /**
   * @dev redeem a ticket.
   * If the sender has a channel to the source, the amount will be transferred
   * to that channel, otherwise it will be sent to their address directly.
   * @param nextCommitment the commitment that hashes to the redeemers previous commitment
   * @param signature compact ERC-2098 signature (https://eips.ethereum.org/EIPS/eip-2098)
   * @param ticket ticket to redeem
   */
  function redeemTicket(
    bytes32 nextCommitment,
    bytes32 por_secret,
    CompactSignature signature,
    Ticket ticket
  ) external validateBalance(ticket.amount) validatePoRSecret(ticket.proofOfRelaySecret) {
    Channel storage spendingChannel = this.channels(ticket.channelId);

    if (nextCommitment != bytes32(0) || spendingChannel.commitment != keccak256(abi.encodePacked(nextCommitment))) {
      revert InvalidCommitmentOpening();
    }

    if (spendingChannel.status != ChannelStatus.OPEN && spendingChannel.status != ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'spending channel must be open or pending to close'});
    }

    if (spendingChannel.channelEpoch != ticket.channel_epoch) {
      revert WrongChannelState({reason: 'channel epoch must match'});
    }

    if (spendingChannel.ticketIndex >= ticket.ticketIndex) {
      revert WrongChannelState({reason: 'a ticket with higher ticket index has already been redeemed'});
    }

    if (spendingChannel.amount < ticket.amount) {
      revert InsufficientChannelBalance();
    }

    // Deviates from EIP712 due to computed property and non-standard struct property encoding
    bytes32 ticketHash = keccak256(
      abi.encode(
        domainSeparator,
        redeemTicketSeparator,
        keccak256(abi.encode(ticket, this._computeChallenge(por_secret)))
      )
    );

    require(_getTicketLuck(ticketHash, nextCommitment, ticket.por_secret) <= ticket.win_prob, 'ticket must be a win');

    address source = ECDSA.recover(ticketHash, signature.r, ticket.vs);
    if (this._getChannelId(source, msg.sender) != ticket.channel_id) {
      revert InvalidTicketSignature();
    }

    spendingChannel.ticketIndex = ticket.ticketIndex;
    spendingChannel.commitment = nextCommitment;
    spendingChannel.balance = spendingChannel.balance - ticket.amount;
    emit ChannelBalanceIncreased(ticket.channelId, spendingChannel.balance);

    bytes32 outgoingChannelId = this._getChannelId(msg.sender, source);
    Channel storage earningChannel = this.channels(outgoingChannelId);

    if (earningChannel.status == ChannelStatus.CLOSED) {
      // The other channel does not exist, so we need to transfer funds directly
      if (token.transfer(msg.sender, ticket.amount) != true) {
        revert TokenTransferFailed();
      }
    } else {
      earningChannel.balance = earningChannel.balance + ticket.amount;
      emit ChannelBalanceIncreased(outgoingChannelId, earningChannel.balance);
    }

    // Informs about new ticketIndex
    emit TicketRedeemed(ticket.channelId, ticket.ticketIndex);
  }

  /**
   * @dev Initialize channel closure.
   * When a channel owner (the 'source' of the channel) wants to 'cash out',
   * they must notify the counterparty (the 'destination') that they will do
   * so, and provide enough time for them to redeem any outstanding tickets
   * before-hand. This notice period is called the 'cool-off' period.
   * The channel 'destination' should be monitoring blockchain events, thus
   * they should be aware that the closure has been triggered, as this
   * method triggers a {ChannelUpdated} and an {ChannelClosureInitiated} event.
   * After the cool-off period expires, the 'source' can call
   * 'finalizeChannelClosure' which withdraws the stake.
   * @param destination the destination
   */
  function initiateOutgoingChannelClosure(address destination) external {
    // We can only initiate closure to outgoing channels
    bytes32 channelId = this._getChannelId(msg.sender, destination);
    Channel storage channel = this.channels(channelId);

    // calling initiateClosure on a PENDING_TO_CLOSE channel extends the noticePeriod
    if (channel.status != ChannelStatus.OPEN && channel.status != ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'channel must have state OPEN or PENDING_TO_CLOSE'});
    }

    channel.closureTime = _currentBlockTimestamp() + noticePeriodChannelClosure;
    channel.status = ChannelStatus.PENDING_TO_CLOSE;

    // Inform others at which time the notice period is due
    emit ChannelClosureInitiated(channelId, channel.closureTime);
  }

  function closeIncomingChannel(address source) external {
    // We can only close incoming channels directly
    bytes32 channelId = this._getChannelId(source, msg.sender);

    Channel storage channel = this.channels(channelId);

    if (channel.status != ChannelStatus.OPEN) {
      revert WrongChannelState({reason: 'channel must have state OPEN'});
    }

    // Wipes commitment and gives ~20k gas refund
    channel.commitment = bytes32(0);

    channel.status = ChannelStatus.CLOSED; // ChannelStatus.CLOSED == 0
    channel.closureTime = Timestamp(0);
    channel.ticketIndex = TicketIndex(0);

    // channel.epoch must be kept

    if (channel.balance > 0) {
      if (token.transfer(msg.sender, channel.balance) != true) {
        revert TokenTransferFailed();
      }
    }

    channel.balance = Balance(0);
  }

  /**
   * @dev Finalize the channel closure, if cool-off period
   * is over it will close the channel and transfer funds
   * to the sender. Then emits {ChannelUpdated} and the
   * {ChannelClosureFinalized} event.
   * @param destination the address of the counterparty
   */
  function finalizeOutgoingChannelClosure(address destination) external {
    // We can only finalize closure to outgoing channels
    bytes32 channelId = this._getChannelId(msg.sender, destination);
    Channel storage channel = this.channels(channelId);

    if (channel.status != ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'channel must be pending to close'});
    }

    if (channel.closureTime < _currentBlockTimestamp()) {
      revert NoticePeriodNotDue();
    }

    // Wipes commitment and gives ~20k gas refund
    channel.commitment = bytes32(0);

    channel.status = ChannelStatus.CLOSED; // ChannelStatus.CLOSED == 0
    channel.closureTime = Timestamp(0);
    channel.ticketIndex = TicketIndex(0);

    // channel.epoch must be kept

    if (channel.balance > 0) {
      if (token.transfer(msg.sender, channel.balance) != true) {
        revert TokenTransferFailed();
      }
    }

    channel.balance = Balance(0);
  }

  /**
   * Sets a new iterated commitment for incoming channels.
   *
   * @dev this alters the channel state to prevent from
   * @param newCommitment, a secret derived from this new commitment
   * @param source the address of the source of the channel
   */
  function setCommitment(bytes32 newCommitment, address source) external {
    if (newCommitment == bytes32(0)) {
      // revert since setting empty commitments is a no-op and therefore unintended
      revert InvalidCommitment();
    }

    // We can only set commitment to incoming channels.
    bytes32 channelId = this._getChannelId(source, msg.sender);
    Channel storage channel = this.channels(channelId);

    if (channel.status != ChannelStatus.OPEN) {
      revert WrongChannelState({reason: 'Cannot set commitments for channels that are not in state OPEN.'});
    }

    if (channel.commitment != bytes32(0)) {
      // The party ran out of commitment openings, this is a reset
      channel.channelEpoch = channel.channelEpoch + 1;
    }

    channel.commitment = newCommitment;

    emit ChannelBumped(channelId, channel.channelEpoch);
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
    address to,
    uint256 amount,
    bytes calldata userData,
    bytes calldata operatorData
  ) external override {
    require(msg.sender == address(token), 'caller must be HoprToken');
    require(to == address(this), 'must be sending tokens to HoprChannels');

    // must be one of our supported functions
    if (userData.length == FUND_CHANNEL_MULTI_SIZE) {
      address account1;
      uint96 amount1;

      address account2;
      uint96 amount2;

      (account1, amount1, account2, amount2) = abi.decode(userData, (address, Balance, address, Balance));
      require(amount == amount1 + amount2, 'amount sent must be equal to amount specified');

      // fund channel in direction of: account1 -> account2
      if (amount1 > 0) {
        _fundChannel(from, account1, account2, amount1);
      }
      // fund channel in direction of: account2 -> account1
      if (amount2 > 0) {
        _fundChannel(from, account2, account1, amount2);
      }
    }
  }

  // internal code

  /**
   * @dev Funds a channel, then emits
   * {ChannelUpdated} event.
   * @param source the address of the channel source
   * @param dest the address of the channel destination
   * @param amount amount to fund account1
   */
  function _fundChannel(address source, address dest, Balance amount) private validateBalance(amount) {
    bytes32 channelId = this._getChannelId(source, dest);
    Channel storage channel = this.channels(channelId);

    if (channel.status == ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'cannot fund a channel that will close soon'});
    }

    if (channel.status == ChannelStatus.CLOSED) {
      // We are opening or reoping a channel
      channel.channelEpoch = channel.channelEpoch + 1;
      channel.ticketEpoch = 0; // As we've incremented the channel epoch, we can restart the ticket counter
      channel.ticketIndex = 0;

      channel.status = ChannelStatus.OPEN;
      emit ChannelOpened(source, dest, channel.channelEpoch);
    }

    channel.balance = channel.balance + amount;
    emit ChannelBalanceIncreased(channelId, channel.balance);
  }

  /**
   * @param source the address of source
   * @param destination the address of destination
   * @return the channel id
   */
  function _getChannelId(address source, address destination) private pure returns (bytes32) {
    return keccak256(abi.encodePacked(source, destination));
  }

  /**
   * @return the current timestamp
   */
  function _currentBlockTimestamp() private view returns (Timestamp) {
    // solhint-disable-next-line
    return Timestamp(block.timestamp);
  }

  /**
   * Uses the response to recompute the challenge. This is done
   * by multiplying the base point of the curve with the given response.
   * Due to the lack of embedded ECMUL functionality for the secp256k1 curve in the current
   * version of the EVM, this is done by misusing the `ecrecover`
   * functionality. `ecrecover` performs the point multiplication and
   * converts the output to an Ethereum address (sliced hash of the product
   * of base point and scalar).
   * See https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
   * @param response response that is used to recompute the challenge
   */
  function _computeChallenge(bytes32 response) private pure returns (address) {
    address signer = ecrecover(
      0,
      BASE_POINT_Y_COMPONENT_SIGN,
      bytes32(BASE_POINT_X_COMPONENT),
      bytes32(mulmod(uint256(response), BASE_POINT_X_COMPONENT, FIELD_ORDER))
    );

    return signer;
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
  ) private pure returns (WinProb) {
    // hash function produces 256 bits output but we require only first 56 bits (IEEE 754 double precision means 53 signifcant bits)
    return WinProb(keccak256(abi.encodePacked(ticketHash, nextCommitment, proofOfRelaySecret)));
  }
}
