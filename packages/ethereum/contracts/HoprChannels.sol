// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;
pragma abicoder v2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "./utils/ECDSA.sol";
import "./utils/SafeUint24.sol";

contract HoprChannels is IERC777Recipient, ERC1820Implementer {
    using SafeMath for uint256;
    using SafeUint24 for uint24;
    using SafeERC20 for IERC20;

    // required by ERC1820 spec
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    // required by ERC777 spec
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_SIZE = abi.encode(address(0), address(0)).length;
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_MULTI_SIZE = abi.encode(address(0), address(0), uint256(0), uint256(0)).length;

    /**
     * @dev Possible channel statuses.
     */
    enum ChannelStatus { CLOSED, OPEN, PENDING_TO_CLOSE }

    /**
     * @dev A channel struct, used to represent a channel's state
     */
    struct Channel {
        uint256 partyABalance;
        uint256 partyBBalance;

        bytes32 commitmentPartyA;
        bytes32 commitmentPartyB;
        uint256 partyATicketEpoch;
        uint256 partyBTicketEpoch;

        ChannelStatus status;
        uint channelEpoch; 

        // the time when the channel can be closed by either party
        // overloads at year >2105
        uint32 closureTime;

        // channel closure was initiated by party A
        bool closureByPartyA;
    }

    /**
     * @dev Stored channels keyed by their channel ids
     */
    mapping(bytes32 => Channel) public channels;

    /**
     * @dev Stored hashes of tickets keyed by their challenge,
     * true if ticket has been redeemed.
     */
    mapping(bytes32 => bool) public tickets;

    /**
     * @dev HoprToken, the token that will be used to settle payments
     */
    IERC20 public token;

    /**
     * @dev Seconds it takes until we can finalize channel closure once,
     * channel closure has been initialized.
     */
    uint32 public secsClosure;

    event ChannelUpdate(
        address partyA,
        address partyB,
        Channel newState
    );

    event TicketRedeemed(
        // @TODO: remove this and rely on `msg.sender`
        address indexed redeemer,
        address indexed counterparty,
        uint256 amount
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
     * @dev Funds a channel in one direction,
     * then emits {ChannelUpdate} event.
     * @param funder the address of the recipient
     * @param counterparty the address of the counterparty
     * @param amount amount to fund
     */
    function fundChannel(
        address funder,
        address counterparty,
        uint256 amount
    ) external {
        token.safeTransferFrom(msg.sender, address(this), amount);
        _fundChannel(
            funder,
            counterparty,
            amount,
            0
        );
    }

    /**
     * @dev Funds a channel, in both directions,
     * then emits {ChannelUpdate} event.
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
            amount1,
            amount2
        );
    }

    function redeemTicket(
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) external {
        _redeemTicket(
            msg.sender,
            counterparty,
            nextCommitment,
            ticketEpoch,
            proofOfRelaySecret,
            amount,
            winProb,
            r,
            s,
            v
        );
    }

    /**
     * @dev Initialize channel closure, updates channel's
     * closure time, when the cool-off period is over,
     * user may finalize closure, then emits
     * {ChannelUpdate} event.
     * @param counterparty the address of the counterparty
     */
    function initiateChannelClosure(
        address counterparty
    ) external {
        _initiateChannelClosure(msg.sender, counterparty);
    }

    /**
     * @dev Finalize channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the parties involved, then emits
     * {ChannelUpdate} event.
     * @param counterparty the address of the counterparty
     */
    function finalizeChannelClosure(
        address counterparty
    ) external {
        _finalizeChannelClosure(
            msg.sender,
            counterparty
        );
    }

    /**
     * A hook triggered when HOPR tokens are send to this contract.
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
            userData.length == FUND_CHANNEL_SIZE ||
            userData.length == FUND_CHANNEL_MULTI_SIZE,
            "userData must match one of our supported functions"
        );

        address account1;
        address account2;
        uint256 amount1;
        uint256 amount2;

        if (userData.length == FUND_CHANNEL_SIZE) {
            (account1, account2) = abi.decode(userData, (address, address));
            amount1 = amount;
        } else {
            (account1, account2, amount1, amount2) = abi.decode(userData, (address, address, uint256, uint256));
            require(amount == amount1.add(amount2), "amount sent must be equal to amount specified");
        }

        require(from == account1 || from == account2, "funder must be either account1 or account2");
        _fundChannel(account1, account2, amount1, amount2);
    }

    // internal code

    /**
     * @dev Funds a channel, then emits
     * {ChannelUpdate} event.
     * @param account1 the address of account1
     * @param account2 the address of account2
     * @param amount1 amount to fund account1
     * @param amount2 amount to fund account2
     */
    function _fundChannel(
        address account1,
        address account2,
        uint256 amount1,
        uint256 amount2
    ) internal {
        require(account1 != account2, "accountA and accountB must not be the same");
        require(account1 != address(0), "accountA must not be empty");
        require(account2 != address(0), "accountB must not be empty");
        require(amount1 > 0 || amount2 > 0, "amountA or amountB must be greater than 0");

        address partyA;
        address partyB;
        uint256 amountA;
        uint256 amountB;
        
        if (_isPartyA(account1, account2)){
          partyA = account1;
          partyB = account2;
          amountA = amount1;
          amountB = amount2;
        } else {
          partyA = account2;
          partyB = account1;
          amountA = amount2;
          amountB = amount1;
        }
        (,,, Channel storage channel) = _getChannel(partyA, partyB);

        require(channel.status != ChannelStatus.PENDING_TO_CLOSE, "Cannot fund a closing channel");
        
        if (channel.status == ChannelStatus.CLOSED) {
          // We are reopening the channel
          channel.channelEpoch = channel.channelEpoch.add(1);
          channel.status = ChannelStatus.OPEN;
        }

        channel.partyABalance = channel.partyABalance.add(amountA);
        channel.partyBBalance = channel.partyBBalance.add(amountB);
        emit ChannelUpdate(partyA, partyB, channel);
    }

    /**
     * @dev Initialize channel closure, updates channel's
     * closure time, when the cool-off period is over,
     * user may finalize closure, then emits
     * {ChannelUpdate} event.
     * @param initiator the address of the initiator
     * @param counterparty the address of the counterparty
     */
    function _initiateChannelClosure(
        address initiator,
        address counterparty
    ) internal {
        require(initiator != counterparty, "initiator and counterparty must not be the same");
        require(initiator != address(0), "initiator must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");

        (,,, Channel storage channel) = _getChannel(initiator, counterparty);
        require(channel.status == ChannelStatus.OPEN, "channel must be open");

        // @TODO: check with team, do we need SafeMath check here?
        channel.closureTime = _currentBlockTimestamp() + secsClosure;
        channel.status = ChannelStatus.PENDING_TO_CLOSE;

        bool isPartyA = _isPartyA(initiator, counterparty);
        if (isPartyA) {
            channel.closureByPartyA = true;
        }

        emit ChannelUpdate(initiator, counterparty, channel);
    }

    /**
     * @dev Finalize channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the parties involved, then emits
     * {ChannelUpdate} event.
     * @param initiator the address of the initiator
     * @param counterparty the address of the counterparty
     */
    function _finalizeChannelClosure(
        address initiator,
        address counterparty
    ) internal {
        require(address(token) != address(0), "token must not be empty");
        require(initiator != counterparty, "initiator and counterparty must not be the same");
        require(initiator != address(0), "initiator must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");

        (address partyA, address partyB,, Channel storage channel) = _getChannel(initiator, counterparty);
        require(channel.status == ChannelStatus.PENDING_TO_CLOSE, "channel must be pending to close");

        if (
            channel.closureByPartyA && (initiator == partyA) ||
            !channel.closureByPartyA && (initiator == partyB)
        ) {
            require(channel.closureTime < _currentBlockTimestamp(), "closureTime must be before now");
        }

        // settle balances
        if (channel.partyABalance > 0) {
            token.transfer(partyA, channel.partyABalance);
        }
        if (channel.partyBBalance > 0) {
            token.transfer(partyB, channel.partyBBalance);
        }

        delete channel.partyABalance; // channel.partyABalance = 0
        delete channel.partyBBalance; 
        delete channel.closureTime; // channel.closureTime = 0
        delete channel.closureByPartyA; // channel.closureByPartyA = false

        emit ChannelUpdate(initiator, counterparty, channel);
    }

    /**
     * @param account1 the address of accountA
     * @param account2 the address of accountB
     * @return a tuple of partyA, partyB, channelId, channel
     */
    function _getChannel(address account1, address account2)
        internal
        view
        returns (
            address,
            address,
            bytes32,
            Channel storage
        )
    {
        (address partyA, address partyB) = _sortAddresses(account1, account2);
        bytes32 channelId = _getChannelId(partyA, partyB);
        Channel storage channel = channels[channelId];

        return (partyA, partyB, channelId, channel);
    }

    /**
     * @param partyA the address of partyA
     * @param partyB the address of partyB
     * @return the channel id by hashing partyA and partyB
     */
    function _getChannelId(address partyA, address partyB) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(partyA, partyB));
    }

    /**
     * Parties are ordered - find the lower one.
     * @param query the address of which we are asking 'is this party A'
     * @param other the other address 
     * @return query is partyA 
     */
    function _isPartyA(address query, address other) internal pure returns (bool) {
        return uint160(query) < uint160(other);
    }

    /**
     * @param accountA the address of accountA
     * @param accountB the address of accountB
     * @return a tuple representing partyA and partyB
     */
    function _sortAddresses(address accountA, address accountB) internal pure returns (address, address) {
        if (_isPartyA(accountA, accountB)) {
            return (accountA, accountB);
        } else {
            return (accountB, accountA);
        }
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
     * @param secretPreImage the secretPreImage that results to the redeemers channel commitment
     * @param proofOfRelaySecret the proof of relay secret
     * @param winProb the winning probability of the ticket
     * @param amount the amount in the ticket
     * @param r part of the signature
     * @param s part of the signature
     * @param v part of the signature
     */
    function _redeemTicket(
        address redeemer,
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) internal {
        require(redeemer != address(0), "redeemer must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");
        require(nextCommitment != bytes32(0), "nextCommitment must not be empty");
        require(proofOfRelaySecret != bytes32(0), "proofOfRelaySecret must not be empty");
        require(amount != uint256(0), "amount must not be empty");
        // require(winProb != bytes32(0), "winProb must not be empty");
        require(r != bytes32(0), "r must not be empty");
        require(s != bytes32(0), "s must not be empty");
        require(v != uint8(0), "v must not be empty");
        (,,, Channel storage channel) = _getChannel(
            redeemer,
            counterparty
        );

        if (_isPartyA(redeemer, counterparty) {
          require(channel.partyACommitment == keccak256(abi.encodePacked(nextCommitment)), "commitment must be hash of next commitment");
          require(channel.partyATicketEpoch == ticketEpoch, "Ticket epoch must match");
        } else {
          require(channel.partyBCommitment == keccak256(abi.encodePacked(nextCommitment)), "commitment must be hash of next commitment");
          require(channel.partyBTicketEpoch == ticketEpoch, "Ticket epoch must match");
        }
        require(channel.status != ChannelStatus.CLOSED, "channel must be open or pending to close");

        uint256 ticketEpoch;
        if (_isPartyA(redeemer, counterparty)) {
          ticketEpoch = channel.partyATicketEpoch;
        } else {
          ticketEpoch = channel.partyBTicketEpoch;
        }

        bytes32 ticketHash = _getTicketHash(
            _getEncodedTicket(
                redeemer,
                ticketEpoch,
                proofOfRelaySecret,
                channel.channelEpoch,
                amount,
                winProb
            )
        );
        require(!tickets[ticketHash], "ticket must not be used twice");
        require(ECDSA.recover(ticketHash, r, s, v) == counterparty, "signer must match the counterparty");
        require(
            uint256(_getTicketLuck(
                ticketHash,
                secretPreImage,
                proofOfRelaySecret,
                winProb
            )) <= uint256(winProb),
            "ticket must be a win"
        );

        tickets[ticketHash] = true;

        if (_isPartyA(redeemer, counterparty)) {
            channel.partyACommitment = nextCommitment;
            channel.partyABalance = channel.partyABalance.add(amount);
            channel.partyATicketEpoch = channel.partyATicketEpoch.add(1);
        } else {
            channel.partyABalance = channel.partyABalance.sub(amount);
            channel.partyBCommitment = nextCommitment;
            channel.partyBTicketEpoch = channel.partyATicketEpoch.add(1);
        }

        emit TicketRedeemed(redeemer, counterparty, amount);
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
        bytes32 winProb
    ) internal pure returns (bytes memory) {
        bytes32 challenge = keccak256(abi.encodePacked(proofOfRelaySecret));

        return abi.encodePacked(
            recipient,
            challenge,
            recipientCounter,
            amount,
            winProb,
            channelIteration
        );
    }

    /**
     * @dev Prefix the ticket message and return
     * the actual hash that was used to sign
     * the ticket with.
     * @return prefixed ticket hash
     */
    function _getTicketHash(
        bytes memory packedTicket
    ) internal pure returns (bytes32) {
        return ECDSA.toEthSignedMessageHash(
            "187",
            packedTicket
        );
    }

    /**
     * @dev Get the ticket's "luck" by
     * hashing provided values.
     * @return luck
     */
    function _getTicketLuck(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(ticketHash, secretPreImage, proofOfRelaySecret, winProb));
    }
}
