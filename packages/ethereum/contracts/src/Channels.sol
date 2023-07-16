// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.19;

import 'openzeppelin-contracts-4.8.3/utils/Multicall.sol';
import 'openzeppelin-contracts-4.8.3/utils/introspection/IERC1820Registry.sol';
import 'openzeppelin-contracts-4.8.3/utils/introspection/ERC1820Implementer.sol';
import 'openzeppelin-contracts-4.8.3/token/ERC20/IERC20.sol';
import 'openzeppelin-contracts-4.8.3/token/ERC777/IERC777Recipient.sol';
import 'openzeppelin-contracts-4.8.3/utils/cryptography/ECDSA.sol';
import './interfaces/INodeSafeRegistry.sol';

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
error WrongSafeNodePair();
error UnusedSafeNodePair();

/**
 *    &&&&
 *    &&&&
 *    &&&&
 *    &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *    &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *     &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *     &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *     &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *     %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *    %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                          %%%%
 *                                          %%%%
 *                                          %%%%
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
  type TicketReserved is uint16;

  Balance public constant MAX_USED_BALANCE = Balance.wrap(10 ** 25); // 1% of total supply, staking more is not sound
  Balance public constant MIN_USED_BALANCE = Balance.wrap(1); // no empty token transactions

  // Field order created by secp256k1 curve
  bytes32 constant FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

  // x-component of base point of secp256k1 curve
  bytes32 constant BASE_POINT_X_COMPONENT = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;

  // encoded sign of y-component of base point of secp256k1 curve
  uint8 constant BASE_POINT_Y_COMPONENT_SIGN = 27;

  // ERC-777 tokensReceived hook, fundChannelMulti
  uint256 public immutable FUND_CHANNEL_MULTI_SIZE =
    abi.encode(address(0), Balance.wrap(0), address(0), Balance.wrap(0)).length;

  // ERC-777 tokensReceived hook, fundChannel
  uint256 public immutable FUND_CHANNEL_SIZE =  abi.encode(address(0)).length;

  string public constant version = '2.0.0';

  bytes32 public immutable domainSeparator; // depends on chainId
  // Non-standard usage of EIP712 due computed property and custom property encoding
  bytes32 public constant redeemTicketSeparator =
    keccak256(
      'Ticket(bytes32 channelId,uint96 balance,uint64 ticketIndex,uint24 epoch,uint56 winProb,address porChallenge)'
    );

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
   * Holds a compact ECDSA signature, following ERC-2098
   */
  struct CompactSignature {
    bytes32 r;
    bytes32 vs;
  }

  /**
   * Represents the state of a channel
   *
   * Aligned to 2 EVM words
   */
  struct Channel {
    // iterated commitment, most recent opening gets revealed when redeeming a ticket
    bytes32 commitment;
    // latest balance of the channel, changes whenever a ticket gets redeemed
    Balance balance;
    // prevents tickets from being replayed, increased with every redeemed ticket
    TicketIndex ticketIndex;
    // if set, timestamp once we can pull all funds from the channel
    Timestamp closureTime;
    // prevents tickets issued for older instantions to be replayed
    ChannelEpoch epoch;
    // current state of the channel
    ChannelStatus status;
  }

  /**
   * Represents a ticket that can be redeemed using `redeemTicket` function.
   *
   * Aligned to 2 EVM words
   */
  struct TicketData {
    bytes32 channelId;
    Balance amount;
    TicketIndex index;
    ChannelEpoch epoch;
    WinProb winProb;
    TicketReserved reserved; // for future use
  }

  struct RedeemableTicket {
    TicketData data;
    CompactSignature signature;
    bytes32 opening;
    bytes32 porSecret;
  }

  /**
   * Stores channels, indexed by their channelId
   */
  mapping(bytes32 => Channel) public channels;

  /**
   * Token that will be used for all interactions.
   */
  IERC20 public immutable token;

  /**
   * Contract that stores the node-safe registry
   */
  IHoprNodeSafeRegistry public immutable nodeSafeRegistry;

  /**
   * Notice period before fund from an outgoing channel can be pulled out.
   */
  Timestamp public immutable noticePeriodChannelClosure; // in seconds

  /**
   * Emitted once a channel is opened.
   *
   * Includes source and destination separately because mapping
   * (source, destination) -> channelId destroys information.
   */
  event ChannelOpened(address indexed source, address indexed destination, Balance amount);

  /**
   * Emitted once balance of a channel is increased, e.g. after opening a
   * channel or redeeming a ticket.
   */
  event ChannelBalanceIncreased(bytes32 indexed channelId, Balance newBalance);

  /**
   * Emitted once balance of a channel is decreased, e.g. when redeeming
   * a ticket or closing a channel.
   */
  event ChannelBalanceDecreased(bytes32 indexed channelId, Balance newBalance);

  /**
   * Emitted once a commitment has been set for a channel. Includes
   * the current epoch since this value is necessary for issuing tickets.
   */
  event CommitmentSet(bytes32 indexed channelId, ChannelEpoch epoch);

  /**
   * Emitted once a party initiates the closure of an outgoing
   * channel. Includes the timestamp when the notice period is due.
   */
  event OutgoingChannelClosureInitiated(bytes32 channelId, Timestamp closureInitiationTime);

  /**
   * Emitted once a channel closure is finalized.
   */
  event ChannelClosed(bytes32 channelId);

  /**
   * Emitted once a ticket is redeemed. Includes latest ticketIndex
   * since this value is necessary for issuing and validating tickets.
   */
  event TicketRedeemed(bytes32 channelId, TicketIndex newTicketIndex);

  /**
   * @param _token HoprToken address
   * @param _nodeSafeRegistry address of the global registry of node and safe
   * @param _noticePeriodChannelClosure seconds until a channel can be closed
   */
  constructor(
    address _token, 
    address _nodeSafeRegistry,
    Timestamp _noticePeriodChannelClosure
  ) {
    if (Timestamp.unwrap(_noticePeriodChannelClosure) == 0) {
      revert InvalidNoticePeriod();
    }

    require(_token != address(0), 'token must not be empty');

    token = IERC20(_token);
    nodeSafeRegistry = IHoprNodeSafeRegistry(_nodeSafeRegistry);
    noticePeriodChannelClosure = _noticePeriodChannelClosure;
    _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));

    domainSeparator = keccak256(
      abi.encode(
        keccak256('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)'),
        keccak256(bytes('HoprChannels')),
        keccak256(bytes(version)),
        block.chainid,
        address(this)
      )
    );
  }

  modifier onlySafe(address node) {
    // check if NodeSafeRegistry entry exists
    if (nodeSafeRegistry.nodeToSafe(node) != msg.sender) {
      revert WrongSafeNodePair();
    }
    _;
  }

  modifier noSafeSet(address node) {
    // check if NodeSafeRegistry entry **does not** exist
    if (nodeSafeRegistry.nodeToSafe(node) != address(0)) {
      revert UnusedSafeNodePair();
    }
    _;
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
    if (Balance.unwrap(balance) < Balance.unwrap(MIN_USED_BALANCE)) {
      revert InvalidBalance();
    }
    if (Balance.unwrap(balance) > Balance.unwrap(MAX_USED_BALANCE)) {
      revert BalanceExceedsGlobalPerChannelAllowance();
    }
    _;
  }

  modifier validatePoRSecret(bytes32 response) {
    if (response == bytes32(0)) {
      revert InvalidPoRSecret('Response is 0. Value must be within the field');
    }

    if (response >= FIELD_ORDER) {
      revert InvalidPoRSecret('Response greater than field order. Value must be within the field');
    }
    _;
  }

  /**
   * Fund an outgoing channel
   * Used in channel operation with Safe
   *
   * @param self address of the source
   * @param account address of the destination
   * @param amount amount to fund for channel
   */
  function fundChannelSafe(
    address self,
    address account,
    Balance amount
  ) external onlySafe(self) {
    _fundChannelInternal(self, account, amount);
  }

  /**
   * Fund an outgoing channel by a node
   * @param self address of the source
   * @param account address of the destination
   * @param amount amount to fund for channel
   */
  function fundChannel(
    address self,
    address account,
    Balance amount
  ) external noSafeSet(msg.sender) {
    _fundChannelInternal(msg.sender, account, amount);
  }

  /**
   * @dev Internal function to fund an outgoing channel from self to account with amount token
   * @notice only balance above zero can execute
   *
   * @param self source address
   * @param account destination address
   * @param amount token amount
   */
  function _fundChannelInternal(
    address self,
    address account,
    Balance amount
  ) internal {
    // pull tokens from funder and handle result
    if (token.transferFrom(msg.sender, address(this), Balance.unwrap(amount)) != true) {
      // sth. went wrong, we need to revert here
      revert TokenTransferFailed();
    }

    if (Balance.unwrap(amount) == 0) {
      revert InvalidBalance();
    }

    _fundChannel(self, account, amount);
  }

  /**
   * Funds and thereby opens a channel from
   * - `account1` -> `account2` with `amount1` tokens
   * - `account2` -> `account1` with `amount2` tokens
   *
   * Used for testing and with ERC777.tokensReceived() method.
   *
   * @param account1 address of account1
   * @param amount1 amount to fund for channel `account1` -> `account2`
   * @param account2 address of account2
   * @param amount2 amount to fund for channel `account2` -> `account1`
   */
  function fundChannelMulti(
    address account1,
    Balance amount1,
    address account2,
    Balance amount2
  ) external {
    // pull tokens from funder and handle result
    if (token.transferFrom(msg.sender, address(this), Balance.unwrap(amount1) + Balance.unwrap(amount2)) != true) {
      // sth. went wrong, we need to revert here
      revert TokenTransferFailed();
    }

    // fund channel in direction of: account1 -> account2
    if (Balance.unwrap(amount1) > 0) {
      _fundChannel(account1, account2, amount1);
    }
    // fund channel in direction of: account2 -> account1
    if (Balance.unwrap(amount2) > 0) {
      _fundChannel(account2, account1, amount2);
    }

    if (Balance.unwrap(amount1) == 0 && Balance.unwrap(amount2) == 0) {
      revert InvalidBalance();
    }
  }

  function redeemTicketSafe(address self, RedeemableTicket calldata redeemable) external onlySafe(self) {
    _redeemTicketInternal(self, redeemable);
  }

  function redeemTicket(RedeemableTicket calldata redeemable) external noSafeSet(msg.sender) {
    _redeemTicketInternal(msg.sender, redeemable);
  }

  /**
   * Claims the incentive for relaying a mixnet packet using probabilistic payments.
   *
   * The caller needs to present a signed ticket. This ticket states a challenge which
   * must be fulfilled. The caller must provide the opening of an on-chain commitment.
   * Last, but not least, the probabilistic ticket must be a win - which can be determined
   * by the caller before submitting the transaction.
   *
   * @param redeemable ticket, signature, opening, porSecret
   */
  function _redeemTicketInternal(
    address self,
    RedeemableTicket calldata redeemable
  ) internal validateBalance(redeemable.data.amount) validatePoRSecret(redeemable.porSecret) {
    Channel storage spendingChannel = channels[redeemable.data.channelId];

    if (
      redeemable.opening != bytes32(0) || spendingChannel.commitment != keccak256(abi.encodePacked(redeemable.opening))
    ) {
      revert InvalidCommitmentOpening();
    }

    if (spendingChannel.status != ChannelStatus.OPEN && spendingChannel.status != ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'spending channel must be open or pending to close'});
    }

    if (ChannelEpoch.unwrap(spendingChannel.epoch) != ChannelEpoch.unwrap(redeemable.data.epoch)) {
      revert WrongChannelState({reason: 'channel epoch must match'});
    }

    if (TicketIndex.unwrap(spendingChannel.ticketIndex) >= TicketIndex.unwrap(redeemable.data.index)) {
      revert WrongChannelState({reason: 'a ticket with higher ticket index has already been redeemed'});
    }

    if (Balance.unwrap(spendingChannel.balance) < Balance.unwrap(redeemable.data.amount)) {
      revert InsufficientChannelBalance();
    }

    // Deviates from EIP712 due to computed property and non-standard struct property encoding
    bytes32 ticketHash = _getTicketHash(redeemable);

    if (!_isWinningTicket(ticketHash, redeemable)) {
      revert TicketIsNotAWin();
    }

    address source = ECDSA.recover(ticketHash, redeemable.signature.r, redeemable.signature.vs);
    if (_getChannelId(source, self) != redeemable.data.channelId) {
      revert InvalidTicketSignature();
    }

    spendingChannel.ticketIndex = redeemable.data.index;
    spendingChannel.commitment = redeemable.opening;
    spendingChannel.balance = Balance.wrap(
      Balance.unwrap(spendingChannel.balance) - Balance.unwrap(redeemable.data.amount)
    );
    emit ChannelBalanceDecreased(redeemable.data.channelId, spendingChannel.balance);

    bytes32 outgoingChannelId = _getChannelId(self, source);
    Channel storage earningChannel = channels[outgoingChannelId];

    if (earningChannel.status == ChannelStatus.CLOSED) {
      // The other channel does not exist, so we need to transfer funds directly
      if (token.transfer(self, Balance.unwrap(redeemable.data.amount)) != true) {
        revert TokenTransferFailed();
      }
    } else {
      earningChannel.balance = Balance.wrap(
        Balance.unwrap(earningChannel.balance) + Balance.unwrap(redeemable.data.amount)
      );
      emit ChannelBalanceIncreased(outgoingChannelId, earningChannel.balance);
    }

    // Informs about new ticketIndex
    emit TicketRedeemed(redeemable.data.channelId, redeemable.data.index);
  }

  function initiateOutgoingChannelClosureSafe(address self, address destination) external onlySafe(self) {
    _initiateOutgoingChannelClosureInternal(self, destination);
  }

  function initiateOutgoingChannelClosure(address destination) external noSafeSet(msg.sender) {
    _initiateOutgoingChannelClosureInternal(msg.sender, destination);
  }

  /**
   * Prepares a channel to pull out funds from an outgoing channel.
   *
   * There is a notice period to give the other end, `destination`, the
   * opportunity to redeem their collected tickets.
   *
   * @param destination destination end of the channel to close
   */
  function _initiateOutgoingChannelClosureInternal(address self, address destination) internal {
    // We can only initiate closure to outgoing channels
    bytes32 channelId = _getChannelId(self, destination);
    Channel storage channel = channels[channelId];

    // calling initiateClosure on a PENDING_TO_CLOSE channel extends the noticePeriod
    if (channel.status != ChannelStatus.OPEN && channel.status != ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'channel must have state OPEN or PENDING_TO_CLOSE'});
    }

    channel.closureTime = Timestamp.wrap(
      Timestamp.unwrap(_currentBlockTimestamp()) + Timestamp.unwrap(noticePeriodChannelClosure)
    );
    channel.status = ChannelStatus.PENDING_TO_CLOSE;

    // Inform others at which time the notice period is due
    emit OutgoingChannelClosureInitiated(channelId, channel.closureTime);
  }

  function closeIncomingChannelSafe(address self, address source) external onlySafe(self) {
    _closeIncomingChannelInternal(self, source);
  }

  function closeIncomingChannel(address source) external noSafeSet(msg.sender) {
    _closeIncomingChannelInternal(msg.sender, source);
  }

  /**
   * Closes an incoming channel.
   *
   * This can happen immediately since it is up to the caller to
   * redeem their collected tickets.
   *
   * @param source source end of the channel to close
   */
  function _closeIncomingChannelInternal(address self, address source) internal {
    // We can only close incoming channels directly
    bytes32 channelId = _getChannelId(source, self);

    Channel storage channel = channels[channelId];

    if (channel.status != ChannelStatus.OPEN) {
      revert WrongChannelState({reason: 'channel must have state OPEN'});
    }

    // Wipes commitment and gives ~20k gas refund
    channel.commitment = bytes32(0);

    channel.status = ChannelStatus.CLOSED; // ChannelStatus.CLOSED == 0
    channel.closureTime = Timestamp.wrap(0);
    channel.ticketIndex = TicketIndex.wrap(0);

    // channel.epoch must be kept

    if (Balance.unwrap(channel.balance) > 0) {
      if (token.transfer(source, Balance.unwrap(channel.balance)) != true) {
        revert TokenTransferFailed();
      }
    }

    emit ChannelClosed(channelId);

    channel.balance = Balance.wrap(0);
  }

  function finalizeOutgoingChannelClosureSafe(address self, address destination) external onlySafe(self) {
    _finalizeOutgoingChannelClosureInternal(self, destination);
  }

  function finalizeOutgoingChannelClosure(address destination) external noSafeSet(msg.sender) {
    _finalizeOutgoingChannelClosureInternal(msg.sender, destination);
  }

  /**
   * Pulls out funds from an outgoing channel. Can be called once
   * notice period is due.
   *
   * @param destination the address of the counterparty
   */
  function _finalizeOutgoingChannelClosureInternal(address self, address destination) internal {
    // We can only finalize closure to outgoing channels
    bytes32 channelId = _getChannelId(self, destination);
    Channel storage channel = channels[channelId];

    if (channel.status != ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'channel must be pending to close'});
    }

    if (Timestamp.unwrap(channel.closureTime) < Timestamp.unwrap(_currentBlockTimestamp())) {
      revert NoticePeriodNotDue();
    }

    // Wipes commitment and gives ~20k gas refund
    channel.commitment = bytes32(0);

    channel.status = ChannelStatus.CLOSED; // ChannelStatus.CLOSED == 0
    channel.closureTime = Timestamp.wrap(0);
    channel.ticketIndex = TicketIndex.wrap(0);

    // channel.epoch must be kept

    if (Balance.unwrap(channel.balance) > 0) {
      if (token.transfer(self, Balance.unwrap(channel.balance)) != true) {
        revert TokenTransferFailed();
      }
    }

    emit ChannelClosed(channelId);

    channel.balance = Balance.wrap(0);
  }

  function setCommitmentSafe(address self, bytes32 newCommitment, address source) external onlySafe(self) {
    _setCommitmentInternal(self, newCommitment, source);
  }

  function setCommitment(bytes32 newCommitment, address source) external noSafeSet(msg.sender) {
    _setCommitmentInternal(msg.sender, newCommitment, source);
  }

  /**
   * Sets a new iterated commitment for the channel source -> self
   *
   * When issuing a probabilistic ticket, it must stay hidden whether that ticket
   * leads to a payout. To hide this information, nodes must deposit in advance some
   * entropy on-chain that is fetched when redeeming the ticket.
   *
   * @param newCommitment, a secret derived from this new commitment
   * @param source the address of the source of the channel
   */
  function _setCommitmentInternal(address self, bytes32 newCommitment, address source) internal {
    if (newCommitment == bytes32(0)) {
      // revert since setting empty commitments is a no-op and therefore unintended
      revert InvalidCommitment();
    }

    // We can only set commitment to incoming channels.
    bytes32 channelId = _getChannelId(source, self);
    Channel storage channel = channels[channelId];

    if (channel.status != ChannelStatus.OPEN) {
      revert WrongChannelState({reason: 'Cannot set commitments for channels that are not in state OPEN.'});
    }

    // Cannot set same commitment again
    if (channel.commitment == newCommitment) {
      revert InvalidCommitment();
    }

    if (channel.commitment != bytes32(0)) {
      // The party ran out of commitment openings, this is a reset
      channel.epoch = ChannelEpoch.wrap(ChannelEpoch.unwrap(channel.epoch) + 1);
    }

    channel.commitment = newCommitment;

    emit CommitmentSet(channelId, channel.epoch);
  }

  /**
   * ERC777.tokensReceived() hook, triggered when sending funds to this contract
   *
   * Parses the payload and opens encoded channels.
   *
   * @param to address recipient address
   * @param amount uint256 amount of tokens to transfer
   * @param userData bytes extra information provided by the token holder (if any)
   */
  function tokensReceived(
    address,
    address,
    address to,
    uint256 amount,
    bytes calldata userData,
    bytes calldata
  ) external override {
    // don't accept any other tokens ;-)
    require(msg.sender == address(token), 'caller must be HoprToken');
    require(to == address(this), 'must be sending tokens to HoprChannels');

    // Opens an outgoing channel
    if (userData.length == FUND_CHANNEL_SIZE) {
      address dest;

      (dest) = abi.decode(userData, (address));

      _fundChannel(msg.sender, dest, Balance.wrap(uint96(amount)));
    // Opens two channels, donating msg.sender's tokens
    } else if (userData.length == FUND_CHANNEL_MULTI_SIZE) {
      address account1;
      Balance amount1;

      address account2;
      Balance amount2;

      (account1, amount1, account2, amount2) = abi.decode(userData, (address, Balance, address, Balance));
      require(
        amount == Balance.unwrap(amount1) + Balance.unwrap(amount2),
        'amount sent must be equal to amount specified'
      );

      // fund channel in direction of: account1 -> account2
      if (Balance.unwrap(amount1) > 0) {
        _fundChannel(account1, account2, amount1);
      }
      // fund channel in direction of: account2 -> account1
      if (Balance.unwrap(amount2) > 0) {
        _fundChannel(account2, account1, amount2);
      }
    }
  }

  // internal code

  /**
   * Funds and thereby opens a channel `source` -> `dest` with `amount` tokens.
   *
   * @param source the address of the channel source
   * @param dest the address of the channel destination
   * @param amount amount to fund account1
   */
  function _fundChannel(address source, address dest, Balance amount) internal validateBalance(amount) validateChannelParties(source, dest) {
    bytes32 channelId = _getChannelId(source, dest);
    Channel storage channel = channels[channelId];

    if (channel.status == ChannelStatus.PENDING_TO_CLOSE) {
      revert WrongChannelState({reason: 'cannot fund a channel that will close soon'});
    }

    channel.balance = Balance.wrap(Balance.unwrap(channel.balance) + Balance.unwrap(amount));

    if (channel.status == ChannelStatus.CLOSED) {
      // We are opening or reoping a channel
      channel.epoch = ChannelEpoch.wrap(ChannelEpoch.unwrap(channel.epoch) + 1);
      channel.ticketIndex = TicketIndex.wrap(0);

      channel.status = ChannelStatus.OPEN;
      emit ChannelOpened(source, dest, channel.balance);
    }
  }

  // utility functions, no state changes involved

  /**
   * Computes the channel identifier
   *
   * @param source the address of source
   * @param destination the address of destination
   * @return the channel id
   */
  function _getChannelId(address source, address destination) public pure returns (bytes32) {
    return keccak256(abi.encodePacked(source, destination));
  }

  /**
   * Gets the current block timestamp correctly sliced to uint32
   */
  function _currentBlockTimestamp() public view returns (Timestamp) {
    // solhint-disable-next-line
    return Timestamp.wrap(uint32(block.timestamp));
  }

  /**
   * Ticket redemption uses an asymmetric challenge-response mechanism whose verification
   * requires scalar multiplication of a secp256k1 curve point.
   *
   * Due to the lack of a cheap secp256k1 ECMUL precompile, the construction misuses
   * the ECRECOVER precompile to compute the scalar multiplication over secp256k1.
   * Although this returns an Ethereum address, the result is usable to validate the response
   * against the stated challenge.
   *
   * For more information see
   * https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
   *
   * @param scalar to multiply with secp256k1 base point
   */
  function _scalarTimesBasepoint(bytes32 scalar) public pure returns (address) {
    return
      ecrecover(
        0,
        BASE_POINT_Y_COMPONENT_SIGN,
        bytes32(BASE_POINT_X_COMPONENT),
        bytes32(mulmod(uint256(scalar), uint256(BASE_POINT_X_COMPONENT), uint256(FIELD_ORDER)))
      );
  }

  /**
   * Gets the non-trivial ticket hash.
   *
   * Note that signature is over ticket data and a computed property which implements
   * a challenge-response mechanism.
   *
   * @param redeemable ticket data
   */
  function _getTicketHash(RedeemableTicket calldata redeemable) public view returns (bytes32) {
    // Deviates from EIP712 due to computed property and non-standard struct property encoding
    bytes32 hashStruct = keccak256(
      abi.encode(
        redeemTicketSeparator,
        keccak256(abi.encode(redeemable.data, _scalarTimesBasepoint(redeemable.porSecret)))
      )
    );

    return keccak256(abi.encode(bytes1(0x19), bytes1(0x01), domainSeparator, hashStruct));
  }

  /**
   * Determines whether a ticket is considered a win.
   *
   * This is done by hashing values that must be revealed when redeeming tickets with
   * a property stated in the signed ticket.
   *
   * @param ticketHash hash of the ticket to check
   * @param redeemable ticket, opening, porSecret, signature
   */
  function _isWinningTicket(
    bytes32 ticketHash,
    RedeemableTicket calldata redeemable
  ) public pure returns (bool) {
    // hash function produces 256 bits output but we require only first 56 bits (IEEE 754 double precision means 53 signifcant bits)
    uint56 ticketProb = (uint56(bytes7(keccak256(abi.encodePacked(
      // unique due to ticketIndex + ticketEpoch
      ticketHash, 
      // entropy + commitment opening ticket redeemer
      redeemable.opening, 
      // challenge-response packet sender + next downstream node
      redeemable.porSecret, 
      // entropy by ticket issuer
      redeemable.signature.r, redeemable.signature.vs)))));

    return ticketProb <= WinProb.unwrap(redeemable.data.winProb);
  }
}
