// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.19;

import { Multicall } from "openzeppelin-contracts/utils/Multicall.sol";
import { IERC1820Registry } from "openzeppelin-contracts/utils/introspection/IERC1820Registry.sol";
import { ERC1820Implementer } from "openzeppelin-contracts/utils/introspection/ERC1820Implementer.sol";
import { IERC20 } from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import { IERC777Recipient } from "openzeppelin-contracts/token/ERC777/IERC777Recipient.sol";
import { ECDSA } from "openzeppelin-contracts/utils/cryptography/ECDSA.sol";

import { HoprCrypto } from "./Crypto.sol";
import { HoprLedger } from "./Ledger.sol";
import { HoprMultiSig } from "./MultiSig.sol";
import { HoprNodeSafeRegistry } from "./node-stake/NodeSafeRegistry.sol";

uint256 constant TWENTY_FOUR_HOURS = 24 * 60 * 60 * 1000; // in milliseconds

uint256 constant INDEX_SNAPSHOT_INTERVAL = TWENTY_FOUR_HOURS;

abstract contract HoprChannelsEvents {
    /**
     * Emitted once a channel is opened.
     *
     * Includes source and destination separately because mapping
     * (source, destination) -> channelId destroys information.
     */
    event ChannelOpened(address indexed source, address indexed destination);

    /**
     * Emitted once balance of a channel is increased, e.g. after opening a
     * channel or redeeming a ticket.
     */
    event ChannelBalanceIncreased(bytes32 indexed channelId, HoprChannels.Balance newBalance);

    /**
     * Emitted once balance of a channel is decreased, e.g. when redeeming
     * a ticket or closing a channel.
     */
    event ChannelBalanceDecreased(bytes32 indexed channelId, HoprChannels.Balance newBalance);

    /**
     * Emitted once a party initiates the closure of an outgoing
     * channel. Includes the timestamp when the notice period is due.
     */
    event OutgoingChannelClosureInitiated(bytes32 indexed channelId, HoprChannels.Timestamp closureTime);

    /**
     * Emitted once a channel closure is finalized.
     */
    event ChannelClosed(bytes32 indexed channelId);

    /**
     * Emitted once a ticket is redeemed. Includes latest ticketIndex
     * since this value is necessary for issuing and validating tickets.
     */
    event TicketRedeemed(bytes32 indexed channelId, HoprChannels.TicketIndex newTicketIndex);

    /**
     * Emitted once the domain separator is updated.
     */
    event DomainSeparatorUpdated(bytes32 indexed domainSeparator);
}

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
 *
 */
contract HoprChannels is
    IERC777Recipient,
    ERC1820Implementer,
    Multicall,
    HoprLedger(INDEX_SNAPSHOT_INTERVAL),
    HoprMultiSig,
    HoprCrypto,
    HoprChannelsEvents
{
    // required by ERC1820 spec
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    // required by ERC777 spec
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");

    type Balance is uint96;
    type TicketIndex is uint48;
    type TicketIndexOffset is uint32;
    type ChannelEpoch is uint24;
    type Timestamp is uint32; // overflows in year 2105
    // Using IEEE 754 double precision -> 53 significant bits
    type WinProb is uint56;

    error InvalidBalance();
    error BalanceExceedsGlobalPerChannelAllowance();
    error SourceEqualsDestination();
    error ZeroAddress(string reason);
    error TokenTransferFailed();
    error InvalidNoticePeriod();
    error NoticePeriodNotDue();
    error WrongChannelState(string reason);
    error InvalidTicketSignature();
    error InvalidVRFProof();
    error InsufficientChannelBalance();
    error TicketIsNotAWin();
    error InvalidAggregatedTicketInterval();
    error WrongToken();
    error InvalidTokenRecipient();
    error InvalidTokensReceivedUsage();

    Balance public constant MAX_USED_BALANCE = Balance.wrap(10 ** 25); // 1% of total supply, staking more is not sound
    Balance public constant MIN_USED_BALANCE = Balance.wrap(1); // no empty token transactions

    // ERC-777 tokensReceived hook, fundChannelMulti
    uint256 public immutable ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE =
        abi.encodePacked(address(0), Balance.wrap(0), address(0), Balance.wrap(0)).length;

    // ERC-777 tokensReceived hook, fundChannel
    uint256 public immutable ERC777_HOOK_FUND_CHANNEL_SIZE = abi.encodePacked(address(0), address(0)).length;

    string public constant VERSION = "2.0.0";

    bytes32 public domainSeparator; // depends on chainId

    /**
     * @dev Channel state machine
     *                                  redeemTicket()
     *                                     ┌──────┐
     * finalizeOutgoingChannelClosure()            v      │
     *  (after notice period), or  ┌──────────────────────┐
     *  closeIncomingChannel()     │                      │ initiateOutgoingChannelClosure()
     *            ┌────────────────│   Pending To Close   │<─────────────────┐
     *            │                │                      │                  │
     *            │                └──────────────────────┘                  │
     *            v                                                          │
     *     ┌────────────┐      tokensReceived() / fundChannel()         ┌──────────┐
     *     │            │──────────────────────────────────────────────>│          │
     *     │   Closed   │           closeIncomingChannel()              │   Open   │
     *     │            │<──────────────────────────────────────────────│          │
     *     └────────────┘                                               └──────────┘
     *                                                                    │      ^
     *                                                                    └──────┘
     *                                                                  redeemTicket()
     */
    enum ChannelStatus {
        CLOSED,
        OPEN,
        PENDING_TO_CLOSE
    }

    /**
     * Represents the state of a channel
     *
     * Aligned to 2 EVM words
     */
    struct Channel {
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
        // ticket is valid in this channel
        bytes32 channelId;
        // amount of tokens to transfer if ticket is a win
        Balance amount;
        // highest channel.ticketIndex to accept when redeeming
        // ticket, used to aggregate tickets off-chain
        TicketIndex ticketIndex;
        // delta by which channel.ticketIndex gets increased when redeeming
        // the ticket, should be set to 1 if ticket is not aggregated, and >1 if
        // it is aggregated. Must never be <1.
        TicketIndexOffset indexOffset;
        // replay protection, invalidates all tickets once payment channel
        // gets closed
        ChannelEpoch epoch;
        // encoded winning probability of the ticket
        WinProb winProb;
    }

    /**
     * Bundles data that is necessary to redeem a ticket
     */
    struct RedeemableTicket {
        // gives each ticket a unique identity and defines what this
        // ticket is worth
        TicketData data;
        // signature by the ticket issuer
        HoprCrypto.CompactSignature signature;
        // proof-of-relay secret computed by ticket redeemer, after
        // receiving keying material from next downstream node
        uint256 porSecret;
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
     * Notice period before fund from an outgoing channel can be pulled out.
     */
    Timestamp public immutable noticePeriodChannelClosure; // in seconds

    /**
     * @param _token HoprToken address
     * @param _noticePeriodChannelClosure seconds until a channel can be closed
     * @param _safeRegistry address of the contract that maps from accounts to deployed Gnosis Safe instances
     */
    constructor(address _token, Timestamp _noticePeriodChannelClosure, HoprNodeSafeRegistry _safeRegistry) {
        if (Timestamp.unwrap(_noticePeriodChannelClosure) == 0) {
            revert InvalidNoticePeriod();
        }

        require(_token != address(0), "token must not be empty");

        setNodeSafeRegistry(_safeRegistry);

        token = IERC20(_token);
        noticePeriodChannelClosure = _noticePeriodChannelClosure;
        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));

        updateDomainSeparator();
    }

    /**
     * Assert that source and destination are good addresses, and distinct.
     */
    modifier validateChannelParties(address source, address destination) {
        if (source == destination) {
            revert SourceEqualsDestination();
        }
        if (source == address(0)) {
            revert ZeroAddress({ reason: "source must not be empty" });
        }
        if (destination == address(0)) {
            revert ZeroAddress({ reason: "destination must not be empty" });
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

    /**
     * @dev recompute the domain seperator in case of a fork
     * This function should be called by anyone when required.
     * An event is emitted when the domain separator is updated
     */
    function updateDomainSeparator() public {
        // following encoding guidelines of EIP712
        bytes32 newDomainSeparator = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes("HoprChannels")),
                keccak256(bytes(VERSION)),
                block.chainid,
                address(this)
            )
        );

        if (newDomainSeparator != domainSeparator) {
            domainSeparator = newDomainSeparator;
            emit DomainSeparatorUpdated(domainSeparator);
        }
    }

    /**
     * See `_redeemTicketInternal`, entrypoint for MultiSig contract
     */
    function redeemTicketSafe(
        address self,
        RedeemableTicket calldata redeemable,
        HoprCrypto.VRFParameters calldata params
    )
        external
        HoprMultiSig.onlySafe(self)
    {
        _redeemTicketInternal(self, redeemable, params);
    }

    /**
     * See `_redeemTicketInternal`
     */
    function redeemTicket(
        RedeemableTicket calldata redeemable,
        HoprCrypto.VRFParameters calldata params
    )
        external
        HoprMultiSig.noSafeSet()
    {
        _redeemTicketInternal(msg.sender, redeemable, params);
    }

    /**
     * ~51k gas execution cost
     * Claims the incentive for relaying a mixnet packet using probabilistic payments.
     *
     * Verifies the outcome of a 3-to-4-party protocol: creator of the packet, ticket
     * issuer and ticket and next downstream node that acknowledges the reception and
     * the validity of the relayed mixnet packet. In many cases, the creator of the
     * packet and ticket redeemer is the same party.
     *
     * The packet creator states the challenge which gets fulfilled by presenting
     * `porSecret` (Proof-Of-Relay). The ticket issuer creates the ticket and signs
     * it. The provided signature acts as a source of entropy given by the ticket
     * issuer. The ticket redeemer ultimately receives a packet with a ticket next
     * to it. Once the ticket redeemer receives the acknowledgement from the next
     * downstream node, it can compute `porSecret`.
     *
     * When submitting the ticket, the ticket redeemer creates a deterministic
     * pseudo-random value that is verifiable by using its public key. This value is
     * unique for each ticket and adds entropy that can only be known by the ticket
     * redeemer.
     *
     * Tickets embed the incentive for relaying a single packet. To reduce on-chain
     * state changes, they can get aggregated before submitting to this function.
     *
     * Aggregated tickets define an validity interval such that the redemption of any
     * individual ticket invalidates the aggregated ticket and vice-versa.
     *
     * Used cryptographic primitives:
     * - ECDSA signature
     * - secp256k1 group homomorphism and DLP property
     * - hash_to_curve using simplified Shallue, van de Woestijne method
     * - Verifiable random function based on hash_to_curve
     * - pseudorandomness of keccak256 function
     *
     * @dev This method makes use of several methods to reduce stack height.
     *
     * @param self account address of the ticket redeemer
     * @param redeemable ticket, signature of ticket issuer, porSecret
     * @param params pseudo-random VRF value + proof that it was correctly using
     *               ticket redeemer's private key
     */
    function _redeemTicketInternal(
        address self,
        RedeemableTicket calldata redeemable,
        HoprCrypto.VRFParameters calldata params
    )
        internal
        validateBalance(redeemable.data.amount)
        HoprCrypto.isFieldElement(redeemable.porSecret)
    {
        Channel storage spendingChannel = channels[redeemable.data.channelId];

        if (spendingChannel.status != ChannelStatus.OPEN && spendingChannel.status != ChannelStatus.PENDING_TO_CLOSE) {
            revert WrongChannelState({ reason: "spending channel must be OPEN or PENDING_TO_CLOSE" });
        }

        if (ChannelEpoch.unwrap(spendingChannel.epoch) != ChannelEpoch.unwrap(redeemable.data.epoch)) {
            revert WrongChannelState({ reason: "channel epoch must match" });
        }

        // Aggregatable Tickets - validity interval:
        // A ticket has a base index and an offset. The offset must be > 0,
        // while the base index must be >= the currently set ticket index in the
        // channel.
        uint48 baseIndex = TicketIndex.unwrap(redeemable.data.ticketIndex);
        uint32 baseIndexOffset = TicketIndexOffset.unwrap(redeemable.data.indexOffset);
        uint48 currentIndex = TicketIndex.unwrap(spendingChannel.ticketIndex);
        if (baseIndexOffset < 1 || baseIndex < currentIndex) {
            revert InvalidAggregatedTicketInterval();
        }

        if (Balance.unwrap(spendingChannel.balance) < Balance.unwrap(redeemable.data.amount)) {
            revert InsufficientChannelBalance();
        }

        // Deviates from EIP712 due to computed property and non-standard struct property encoding
        bytes32 ticketHash = _getTicketHash(redeemable);

        if (!_isWinningTicket(ticketHash, redeemable, params)) {
            revert TicketIsNotAWin();
        }

        HoprCrypto.VRFPayload memory payload =
            HoprCrypto.VRFPayload(ticketHash, self, abi.encodePacked(domainSeparator));

        if (!vrfVerify(params, payload)) {
            revert InvalidVRFProof();
        }

        address source = ECDSA.recover(ticketHash, redeemable.signature.r, redeemable.signature.vs);
        if (_getChannelId(source, self) != redeemable.data.channelId) {
            revert InvalidTicketSignature();
        }

        spendingChannel.ticketIndex = TicketIndex.wrap(baseIndex + baseIndexOffset);
        spendingChannel.balance =
            Balance.wrap(Balance.unwrap(spendingChannel.balance) - Balance.unwrap(redeemable.data.amount));
        indexEvent(
            abi.encodePacked(ChannelBalanceDecreased.selector, redeemable.data.channelId, spendingChannel.balance)
        );
        emit ChannelBalanceDecreased(redeemable.data.channelId, spendingChannel.balance);

        bytes32 outgoingChannelId = _getChannelId(self, source);
        Channel storage earningChannel = channels[outgoingChannelId];

        // Informs about new ticketIndex
        indexEvent(abi.encodePacked(TicketRedeemed.selector, redeemable.data.channelId, spendingChannel.ticketIndex));
        emit TicketRedeemed(redeemable.data.channelId, spendingChannel.ticketIndex);

        if (earningChannel.status == ChannelStatus.CLOSED) {
            // The other channel does not exist, so we need to transfer funds directly
            if (token.transfer(msg.sender, Balance.unwrap(redeemable.data.amount)) != true) {
                revert TokenTransferFailed();
            }
        } else {
            // this CAN produce channels with more stake than MAX_USED_AMOUNT - which does not lead
            // to overflows since total supply < type(uin96).max
            earningChannel.balance =
                Balance.wrap(Balance.unwrap(earningChannel.balance) + Balance.unwrap(redeemable.data.amount));
            indexEvent(abi.encodePacked(ChannelBalanceIncreased.selector, outgoingChannelId, earningChannel.balance));
            emit ChannelBalanceIncreased(outgoingChannelId, earningChannel.balance);
        }
    }

    /**
     * See `_initiateOutgoingChannelClosureInternal`, entrypoint for MultiSig contract
     */
    function initiateOutgoingChannelClosureSafe(
        address self,
        address destination
    )
        external
        HoprMultiSig.onlySafe(self)
    {
        _initiateOutgoingChannelClosureInternal(self, destination);
    }

    /**
     * See `_initiateOutgoingChannelClosureInternal`
     */
    function initiateOutgoingChannelClosure(address destination) external HoprMultiSig.noSafeSet() {
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
        if (channel.status == ChannelStatus.CLOSED) {
            revert WrongChannelState({ reason: "channel must have state OPEN or PENDING_TO_CLOSE" });
        }

        channel.closureTime =
            Timestamp.wrap(Timestamp.unwrap(_currentBlockTimestamp()) + Timestamp.unwrap(noticePeriodChannelClosure));
        channel.status = ChannelStatus.PENDING_TO_CLOSE;

        // Inform others at which time the notice period is due
        indexEvent(abi.encodePacked(OutgoingChannelClosureInitiated.selector, channelId, channel.closureTime));
        emit OutgoingChannelClosureInitiated(channelId, channel.closureTime);
    }

    /**
     * See `_closeIncomingChannelInternal`, entrypoint for MultiSig contract
     */
    function closeIncomingChannelSafe(address self, address source) external HoprMultiSig.onlySafe(self) {
        _closeIncomingChannelInternal(self, source);
    }

    /**
     * See `_closeIncomingChannelInternal`
     */
    function closeIncomingChannel(address source) external HoprMultiSig.noSafeSet() {
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

        if (channel.status == ChannelStatus.CLOSED) {
            revert WrongChannelState({ reason: "channel must have state OPEN or PENDING_TO_CLOSE" });
        }

        uint256 balance = Balance.unwrap(channel.balance);

        channel.status = ChannelStatus.CLOSED; // ChannelStatus.CLOSED == 0
        channel.closureTime = Timestamp.wrap(0);
        channel.ticketIndex = TicketIndex.wrap(0);
        channel.balance = Balance.wrap(0);

        // channel.epoch must be kept

        indexEvent(abi.encodePacked(ChannelClosed.selector, channelId));
        emit ChannelClosed(channelId);

        if (balance > 0) {
            if (token.transfer(source, balance) != true) {
                revert TokenTransferFailed();
            }
        }
    }

    /**
     * See `_finalizeOutgoingChannelClosureInternal`, entrypoint for MultiSig contract
     */
    function finalizeOutgoingChannelClosureSafe(
        address self,
        address destination
    )
        external
        HoprMultiSig.onlySafe(self)
    {
        _finalizeOutgoingChannelClosureInternal(self, destination);
    }

    /**
     * See `_finalizeOutgoingChannelClosureInternal`
     */
    function finalizeOutgoingChannelClosure(address destination) external HoprMultiSig.noSafeSet() {
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
            revert WrongChannelState({ reason: "channel state must be PENDING_TO_CLOSE" });
        }

        if (Timestamp.unwrap(channel.closureTime) >= Timestamp.unwrap(_currentBlockTimestamp())) {
            revert NoticePeriodNotDue();
        }

        uint256 balance = Balance.unwrap(channel.balance);

        channel.status = ChannelStatus.CLOSED; // ChannelStatus.CLOSED == 0
        channel.closureTime = Timestamp.wrap(0);
        channel.ticketIndex = TicketIndex.wrap(0);
        channel.balance = Balance.wrap(0);

        // channel.epoch must be kept

        indexEvent(abi.encodePacked(ChannelClosed.selector, channelId));
        emit ChannelClosed(channelId);

        if (balance > 0) {
            if (token.transfer(msg.sender, balance) != true) {
                revert TokenTransferFailed();
            }
        }
    }

    /**
     * ERC777.tokensReceived() hook, triggered by ERC777 token contract after
     * transfering tokens.
     *
     * Depending on the userData payload, this method either funds one
     * channel, or a bidirectional channel, consisting of two unidirectional
     * channels.
     *
     * Channel source and destination are specified by the userData payload.
     *
     * @dev This function reverts if it results in a no-op, i.e., no state change occurs.
     * @notice The opening of bidirectional channels is currently implemented for internal
     * and community testing purposes only, and is not intended for production use.
     * @param from The account from which the tokens have been transferred
     * @param to The account to which the tokens have been transferred
     * @param amount The amount of tokens transferred
     * @param userData The payload that determines the intended action
     */
    function tokensReceived(
        address, // operator not needed
        address from,
        address to,
        uint256 amount,
        bytes calldata userData,
        bytes calldata // operatorData not needed
    )
        external
        override
    {
        // don't accept any other tokens ;-)
        if (msg.sender != address(token)) {
            revert WrongToken();
        }

        if (to != address(this)) {
            revert InvalidTokenRecipient();
        }

        if (userData.length == 0) {
            // ERC777.tokensReceived() hook got called by `ERC777.send()` or
            // `ERC777.transferFrom()` which we can ignore at this point
            return;
        }

        // Opens an outgoing channel
        if (userData.length == ERC777_HOOK_FUND_CHANNEL_SIZE) {
            if (amount > type(uint96).max) {
                revert BalanceExceedsGlobalPerChannelAllowance();
            }

            address src;
            address dest;

            assembly {
                src := shr(96, calldataload(userData.offset))
                dest := shr(96, calldataload(add(userData.offset, 20)))
            }

            address safeAddress = registry.nodeToSafe(src);

            // skip the check between `from` and `src` on node-safe registry
            if (from == src) {
                // node if opening an outgoing channel
                if (safeAddress != address(0)) {
                    revert ContractNotResponsible();
                }
            } else {
                if (safeAddress != from) {
                    revert ContractNotResponsible();
                }
            }

            _fundChannelInternal(src, dest, Balance.wrap(uint96(amount)));
            // Opens two channels, donating msg.sender's tokens
        } else if (userData.length == ERC777_HOOK_FUND_CHANNEL_MULTI_SIZE) {
            address account1;
            Balance amount1;
            address account2;
            Balance amount2;

            assembly {
                account1 := shr(96, calldataload(userData.offset))
                amount1 := shr(160, calldataload(add(0x14, userData.offset)))
                account2 := shr(96, calldataload(add(0x20, userData.offset)))
                amount2 := shr(160, calldataload(add(0x34, userData.offset)))
            }

            if (amount == 0 || amount != uint256(Balance.unwrap(amount1)) + uint256(Balance.unwrap(amount2))) {
                revert InvalidBalance();
            }

            // fund channel in direction of: account1 -> account2
            if (Balance.unwrap(amount1) > 0) {
                _fundChannelInternal(account1, account2, amount1);
            }
            // fund channel in direction of: account2 -> account1
            if (Balance.unwrap(amount2) > 0) {
                _fundChannelInternal(account2, account1, amount2);
            }
        } else {
            revert InvalidTokensReceivedUsage();
        }
    }

    /**
     * Fund an outgoing channel
     * Used in channel operation with Safe
     *
     * @param self address of the source
     * @param account address of the destination
     * @param amount amount to fund for channel
     */
    function fundChannelSafe(address self, address account, Balance amount) external HoprMultiSig.onlySafe(self) {
        _fundChannelInternal(self, account, amount);

        // pull tokens from Safe and handle result
        if (token.transferFrom(msg.sender, address(this), Balance.unwrap(amount)) != true) {
            // sth. went wrong, we need to revert here
            revert TokenTransferFailed();
        }
    }

    /**
     * Fund an outgoing channel by a node
     * @param account address of the destination
     * @param amount amount to fund for channel
     */
    function fundChannel(address account, Balance amount) external HoprMultiSig.noSafeSet() {
        _fundChannelInternal(msg.sender, account, amount);

        // pull tokens from funder and handle result
        if (token.transferFrom(msg.sender, address(this), Balance.unwrap(amount)) != true) {
            // sth. went wrong, we need to revert here
            revert TokenTransferFailed();
        }
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
    )
        internal
        validateBalance(amount)
        validateChannelParties(self, account)
    {
        bytes32 channelId = _getChannelId(self, account);
        Channel storage channel = channels[channelId];

        if (channel.status == ChannelStatus.PENDING_TO_CLOSE) {
            revert WrongChannelState({ reason: "cannot fund a channel that will close soon" });
        }

        channel.balance = Balance.wrap(Balance.unwrap(channel.balance) + Balance.unwrap(amount));

        if (channel.status == ChannelStatus.CLOSED) {
            // We are opening or reoping a channel
            channel.epoch = ChannelEpoch.wrap(ChannelEpoch.unwrap(channel.epoch) + 1);
            channel.ticketIndex = TicketIndex.wrap(0);

            channel.status = ChannelStatus.OPEN;

            indexEvent(abi.encodePacked(ChannelOpened.selector, self, account));
            emit ChannelOpened(self, account);
        }

        indexEvent(abi.encodePacked(ChannelBalanceIncreased.selector, channelId, channel.balance));
        emit ChannelBalanceIncreased(channelId, channel.balance);
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
     * Gets the hash of a ticket upon which the signature has been
     * created. Also used by the VRF.
     *
     * Tickets come with a signature from the ticket issuer and state a
     * challenge to be fulfilled when redeeming the ticket. As the validity
     * of the signature need to be checked before being able reconstruct
     * the response to the stated challenge, the signature includes the
     * challenge - rather the response, whereas the smart contract
     * requires the response.
     *
     * @param redeemable ticket data
     */
    function _getTicketHash(RedeemableTicket calldata redeemable) public view returns (bytes32) {
        address challenge = HoprCrypto.scalarTimesBasepoint(redeemable.porSecret);

        // TicketData is aligned to exactly 2 EVM words, from which channelId
        // takes one. Removing channelId can thus be encoded in 1 EVM word.
        //
        // Tickets get signed and transferred in packed encoding, consuming
        // 148 bytes, including signature and challenge. Using tight encoding
        // for ticket hash unifies on-chain and off-chain usage of tickets.
        uint256 secondPart = (uint256(Balance.unwrap(redeemable.data.amount)) << 160)
            | (uint256(TicketIndex.unwrap(redeemable.data.ticketIndex)) << 112)
            | (uint256(TicketIndexOffset.unwrap(redeemable.data.indexOffset)) << 80)
            | (uint256(ChannelEpoch.unwrap(redeemable.data.epoch)) << 56) | uint256(WinProb.unwrap(redeemable.data.winProb));

        // Deviates from EIP712 due to computed property and non-standard struct property encoding
        bytes32 hashStruct = keccak256(
            abi.encode(
                this.redeemTicket.selector,
                keccak256(abi.encodePacked(redeemable.data.channelId, secondPart, challenge))
            )
        );

        return keccak256(abi.encodePacked(bytes1(0x19), bytes1(0x01), domainSeparator, hashStruct));
    }

    /**
     * Determines whether a ticket is considered a win.
     *
     * This is done by hashing values that must be revealed when redeeming tickets with
     * a property stated in the signed ticket.
     *
     * @param ticketHash hash of the ticket to check
     * @param redeemable ticket, opening, porSecret, signature
     * @param params VRF values, entropy given by ticket redeemer
     */
    function _isWinningTicket(
        bytes32 ticketHash,
        RedeemableTicket calldata redeemable,
        HoprCrypto.VRFParameters calldata params
    )
        public
        pure
        returns (bool)
    {
        // hash function produces 256 bits output but we require only first 56 bits (IEEE 754 double precision means 53
        // signifcant bits)
        uint56 ticketProb = (
            uint56(
                bytes7(
                    keccak256(
                        abi.encodePacked(
                            // unique due to ticketIndex + ticketEpoch
                            ticketHash,
                            // use deterministic pseudo-random VRF output generated by redeemer
                            params.vx,
                            params.vy,
                            // challenge-response packet sender + next downstream node
                            redeemable.porSecret,
                            // entropy by ticket issuer, only ticket issuer can generate a valid signature
                            redeemable.signature.r,
                            redeemable.signature.vs
                        )
                    )
                )
            )
        );

        return ticketProb <= WinProb.unwrap(redeemable.data.winProb);
    }
}
