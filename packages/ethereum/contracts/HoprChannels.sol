// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

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
    uint256 public FUND_CHANNEL_SIZE = abi.encode(false, address(0), address(0)).length;
    // used by {tokensReceived} to distinguish which function to call after tokens are sent
    uint256 public FUND_CHANNEL_MULTI_SIZE = abi.encode(false, address(0), address(0), uint256(0), uint256(0)).length;

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

    event ChannelCommitmentUpdated(
        address indexed accountA,
        address indexed accountB,
        bytes32 commitmentPartyA,
        bytes32 commitmentPartyB,
        uint256 partyATicketEpoch,
        uint256 partyBTicketEpoch
    );

    event ChannelFunded(
        address indexed accountA,
        address indexed accountB,
        // @TODO: remove this and rely on `msg.sender`
        address funder,
        uint256 partyABalance,
        uint256 partyBBalance
    );

    event ChannelOpened(
        // @TODO: remove this and rely on `msg.sender`
        address indexed opener,
        address indexed counterparty
    );

    event ChannelPendingToClose(
        // @TODO: remove this and rely on `msg.sender`
        address indexed initiator,
        address indexed counterparty,
        uint256 closureTime
    );

    event ChannelClosed(
        // @TODO: remove this and rely on `msg.sender`
        address indexed initiator,
        address indexed counterparty
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
     * then emits {ChannelFunded} event.
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
            msg.sender,
            funder,
            counterparty,
            amount,
            0
        );
    }

    /**
     * @dev Funds a channel, in both directions,
     * then emits {ChannelFunded} event.
     * @param accountA the address of accountA
     * @param accountB the address of accountB
     * @param amountA amount to fund accountA
     * @param amountB amount to fund accountB
     */
    function fundChannelMulti(
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        token.safeTransferFrom(msg.sender, address(this), amountA.add(amountB));

        _fundChannel(
            msg.sender,
            accountA,
            accountB,
            amountA,
            amountB
        );
    }

    /**
     * @dev Opens a channel, then emits
     * {ChannelOpened} event.
     * @param counterparty the address of the counterparty
     */
    function openChannel(address counterparty) external {
        _openChannel(msg.sender, counterparty);
    }

    /**
     * @dev Fund channel and then open it, then emits
     * {ChannelFunded} and {ChannelOpened} events.
     * @param accountA the address of accountA
     * @param accountB the address of accountB
     * @param amountA amount to fund accountA
     * @param amountB amount to fund accountB
     */
    function fundAndOpenChannel(
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        address opener = msg.sender;
        require(
            opener == accountA || opener == accountB,
            "opener must be accountA or accountB"
        );

        token.safeTransferFrom(msg.sender, address(this), amountA.add(amountB));

        address counterparty;
        if (opener == accountA) {
            counterparty = accountB;
        } else {
            counterparty = accountA;
        }

        _fundChannel(opener, accountA, accountB, amountA, amountB);
        _openChannel(opener, counterparty);
    }

    function redeemTicket(
        address counterparty,
        bytes32 secretPreImage,
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
            secretPreImage,
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
     * {ChannelPendingToClose} event.
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
     * {ChannelClosed} event.
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

    // @TODO: check with team, is this function too complex?
    // @TODO: should we support account init?
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

        bool shouldOpen;
        address accountA;
        address accountB;
        uint256 amountA;
        uint256 amountB;

        if (userData.length == FUND_CHANNEL_SIZE) {
            (shouldOpen, accountA, accountB) = abi.decode(userData, (bool, address, address));
            amountA = amount;
        } else {
            (shouldOpen, accountA, accountB, amountA, amountB) = abi.decode(userData, (bool, address, address, uint256, uint256));
            require(amount == amountA.add(amountB), "amount sent must be equal to amount specified");
        }

        _fundChannel(from, accountA, accountB, amountA, amountB);

        if (shouldOpen) {
            require(from == accountA || from == accountB, "funder must be either accountA or accountB");
            _openChannel(accountA, accountB);
        }
    }

    // internal code

    /**
     * @dev Funds a channel, then emits
     * {ChannelFunded} event.
     * @param funder the address of the funder
     * @param accountA the address of accountA
     * @param accountB the address of accountB
     * @param amountA amount to fund accountA
     * @param amountB amount to fund accountB
     */
    function _fundChannel(
        address funder,
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) internal {
        require(funder != address(0), "funder must not be empty");
        require(accountA != accountB, "accountA and accountB must not be the same");
        require(accountA != address(0), "accountA must not be empty");
        require(accountB != address(0), "accountB must not be empty");
        require(amountA > 0 || amountB > 0, "amountA or amountB must be greater than 0");
        require(funder == accountA || funder == accountB, "funder must be A or B");

        (,,, Channel storage channel) = _getChannel(accountA, accountB);

        if (funder == accountA) {
            channel.partyABalance = channel.partyABalance.add(amountA);
        } else {
            channel.partyBBalance = channel.partyBBalance.add(amountB);
        }

        emit ChannelFunded(
            accountA,
            accountB,
            funder,
            channel.partyABalance,
            channel.partyBBalance
        );
    }

    /**
     * @dev Opens a channel, then emits
     * {ChannelOpened} event.
     * @param opener the address of the opener
     * @param counterparty the address of the counterparty
     */
    function _openChannel(
        address opener,
        address counterparty
    ) internal {
        require(opener != counterparty, "opener and counterparty must not be the same");
        require(opener != address(0), "opener must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");

        (,,, Channel storage channel) = _getChannel(opener, counterparty);
        require(channel.partyABalance.add(channel.partyBBalance) > 0, "channel must be funded");

        require(channel.status == ChannelStatus.CLOSED, "channel must be closed in order to open");

        // The channelEpoch indicates the recycling generation and ensures that both parties are using the correct generation.
        channel.channelEpoch = channel.channelEpoch.add(1);
        channel.status = ChannelStatus.OPEN;
        emit ChannelOpened(opener, counterparty);
    }

    /**
     * @dev Initialize channel closure, updates channel's
     * closure time, when the cool-off period is over,
     * user may finalize closure, then emits
     * {ChannelPendingToClose} event.
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

        emit ChannelPendingToClose(initiator, counterparty, channel.closureTime);
    }

    /**
     * @dev Finalize channel closure, if cool-off period
     * is over it will close the channel and transfer funds
     * to the parties involved, then emits
     * {ChannelClosed} event.
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

        emit ChannelClosed(initiator, counterparty);
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
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) internal {
        require(redeemer != address(0), "redeemer must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");
        require(secretPreImage != bytes32(0), "secretPreImage must not be empty");
        require(proofOfRelaySecret != bytes32(0), "proofOfRelaySecret must not be empty");
        require(amount != uint256(0), "amount must not be empty");
        // require(winProb != bytes32(0), "winProb must not be empty");
        require(r != bytes32(0), "r must not be empty");
        require(s != bytes32(0), "s must not be empty");
        require(v != uint8(0), "v must not be empty");

        /* TODO 
        Account storage account = accounts[redeemer];
        require(
            account.secret == keccak256(abi.encodePacked(secretPreImage)),
            // @TODO: add salt
            // accounts[msg.sender].hashedSecret == bytes27(keccak256(abi.encodePacked("HOPRnet", msg.sender, bytes27(preImage)))),
            "secretPreImage must be the hash of redeemer's secret"
        );
        */

        (,,, Channel storage channel) = _getChannel(
            redeemer,
            counterparty
        );
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

        // TODO account.secret = secretPreImage;
        tickets[ticketHash] = true;

        if (_isPartyA(redeemer, counterparty)) {
            channel.partyABalance = channel.partyABalance.add(amount);
        } else {
            channel.partyABalance = channel.partyABalance.sub(amount);
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
