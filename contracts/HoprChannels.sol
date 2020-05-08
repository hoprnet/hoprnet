pragma solidity ^0.6.0;

import "@openzeppelin/contracts/introspection/IERC1820Registry.sol";
import "@openzeppelin/contracts/introspection/ERC1820Implementer.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC777/IERC777Recipient.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";


contract HoprChannels is IERC777Recipient, ERC1820Implementer {
    using SafeMath for uint256;

    // an account has set a new secret hash
    event SecretHashSet(address indexed account, bytes32 secretHash);

    // the payment channel has been funded
    event FundedChannel(
        address indexed funder,
        address indexed recipient,
        address indexed counterParty,
        uint256 recipientAmount,
        uint256 counterPartyAmount
    );

    // the payment channel has been opened
    event OpenedChannel(address indexed opener, address indexed counterParty);

    // a party has initiated channel closure
    event InitiatedChannelClosure(address indexed initiator, address indexed counterParty, uint256 closureTime);

    // the payment channel has been settled and closed
    event ClosedChannel(
        address indexed closer,
        address indexed counterParty,
        uint256 partyAAmount,
        uint256 partyBAmount
    );

    struct Account {
        bytes32 hashedSecret; // account's hashedSecret
        uint256 counter; // increases everytime 'setHashedSecret' is called by the account
    }

    enum ChannelStatus {UNINITIALISED, FUNDED, OPEN, PENDING}

    struct Channel {
        uint256 deposit; // tokens in the deposit
        uint256 partyABalance; // tokens that are claimable by party 'A'
        uint256 closureTime; // the time when the channel can be closed by either party
        uint256 stateCounter;
        /*
            stateCounter:
            0: uninitialised
            1: funding
            2: open
            3: pending
        */
    }

    // setup ERC1820
    IERC1820Registry internal constant _ERC1820_REGISTRY = IERC1820Registry(0x1820a4B7618BdE71Dce8cdc73aAB6C95905faD24);
    bytes32 public constant TOKENS_RECIPIENT_INTERFACE_HASH = keccak256("ERC777TokensRecipient");

    // TODO: update this when adding / removing states.
    uint8 constant NUMBER_OF_STATES = 4;

    // used to protect against malleable signatures
    uint256 constant HALF_CURVE_ORDER = uint256(0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0);

    IERC20 public token; // the token that will be used to settle payments
    uint256 public secsClosure; // seconds it takes to allow closing of channel after channel's -
    // initiated channel closure, in case counter-party does not act -
    // within this time period

    // store accounts' state
    mapping(address => Account) public accounts;

    // store channels' state e.g: channels[hash(party_a, party_b)]
    mapping(bytes32 => Channel) public channels;

    constructor(IERC20 _token, uint256 _secsClosure) public {
        token = _token;
        secsClosure = _secsClosure;

        _ERC1820_REGISTRY.setInterfaceImplementer(address(this), TOKENS_RECIPIENT_INTERFACE_HASH, address(this));
    }

    /**
     * @notice sets caller's hashedSecret
     * @param hashedSecret bytes32 hashedSecret to store
     */
    function setHashedSecret(bytes32 hashedSecret) external {
        require(hashedSecret != bytes32(0), "HoprChannels: hashedSecret is empty");

        Account storage account = accounts[msg.sender];
        require(account.hashedSecret != hashedSecret, "HoprChannels: new and old hashedSecrets are the same");

        account.hashedSecret = hashedSecret;
        account.counter = account.counter += 1;

        emit SecretHashSet(msg.sender, hashedSecret);
    }

    /**
     * Fund a channel between 'accountA' and 'accountB',
     * specified tokens must be approved beforehand.
     * Called when HOPR tokens are send to this contract.
     *
     * @notice fund a channel
     * @param recipient address account which the funds are for
     * @param counterParty address the counterParty of 'recipient'
     * @param additionalDeposit uint256 amount to fund the channel
     */
    function fundChannel(
        address funder,
        address recipient,
        address counterParty,
        uint256 additionalDeposit
    ) internal {
        require(recipient != counterParty, "HoprChannels: 'recipient' and 'counterParty' must not be the same");
        require(recipient != address(0), "HoprChannels: 'recipient' address is empty");
        require(counterParty != address(0), "HoprChannels: 'counterParty' address is empty");
        require(additionalDeposit > 0, "HoprChannels: 'additionalDeposit' must be greater than 0");

        (address party_a, , Channel storage channel, ChannelStatus status) = getChannel(recipient, counterParty);

        require(
            status == ChannelStatus.UNINITIALISED || status == ChannelStatus.FUNDED,
            "HoprChannels: channel must be 'UNINITIALISED' or 'FUNDED'"
        );

        channel.deposit = channel.deposit.add(additionalDeposit);

        if (recipient == party_a) {
            channel.partyABalance = channel.partyABalance.add(additionalDeposit);
        }

        if (status == ChannelStatus.UNINITIALISED) {
            channel.stateCounter += 1;
        }

        emit FundedChannel(funder, recipient, counterParty, additionalDeposit, 0);
    }

    /**
     * Fund a channel between 'initiator' and 'counterParty' using a signature,
     * specified tokens must be approved beforehand.
     *
     * @notice fund a channel
     * @param stateCounter uint256
     * @param additionalDeposit uint256
     * @param partyAAmount uint256
     * @param not_after uint256
     * @param r bytes32
     * @param s bytes32
     * @param v uint8
     */
    function fundChannelWithSig(
        uint256 stateCounter,
        uint256 additionalDeposit,
        uint256 partyAAmount,
        uint256 not_after,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) external {
        address initiator = msg.sender;

        // verification
        require(additionalDeposit > 0, "HoprChannels: 'additionalDeposit' must be strictly greater than zero");
        require(
            partyAAmount <= additionalDeposit,
            "HoprChannels: 'partyAAmount' must be strictly smaller than 'additionalDeposit'"
        );
        require(
            uint256(s) <= HALF_CURVE_ORDER,
            "HoprChannels: found malleable signature, please insert a low-s signature"
        );
        require(not_after >= now, "HoprChannels: signature must not be expired");

        address counterparty = ecrecover(
            prefixed(keccak256(abi.encodePacked(stateCounter, initiator, additionalDeposit, partyAAmount, not_after))),
            v,
            r,
            s
        );

        require(initiator != counterparty, "HoprChannels: initiator and counterparty must not be the same");

        (address partyA, , Channel storage channel, ChannelStatus status) = getChannel(initiator, counterparty);

        require(
            channel.stateCounter == stateCounter,
            "HoprChannels: stored stateCounter and signed stateCounter must be the same"
        );
        require(
            status == ChannelStatus.UNINITIALISED || status == ChannelStatus.FUNDED,
            "HoprChannels: channel must be 'UNINITIALISED' or 'FUNDED'"
        );

        uint256 partyBAmount = additionalDeposit - partyAAmount;

        if (initiator == partyA) {
            token.transferFrom(initiator, address(this), partyAAmount);
            token.transferFrom(counterparty, address(this), partyBAmount);
        } else {
            token.transferFrom(initiator, address(this), partyBAmount);
            token.transferFrom(counterparty, address(this), partyAAmount);
        }

        channel.deposit = additionalDeposit;
        channel.partyABalance = partyAAmount;
        if (status == ChannelStatus.UNINITIALISED) {
            channel.stateCounter += 1;
        }

        if (initiator == partyA) {
            emit FundedChannel(address(0), initiator, counterparty, partyAAmount, partyBAmount);
        } else {
            emit FundedChannel(address(0), counterparty, initiator, partyAAmount, partyBAmount);
        }
    }

    /**
     * @notice open a channel
     * @param counterParty address the counterParty of 'msg.sender'
     */
    function openChannel(address counterParty) public {
        address opener = msg.sender;

        require(opener != counterParty, "HoprChannels: 'opener' and 'counterParty' must not be the same");
        require(counterParty != address(0), "HoprChannels: 'counterParty' address is empty");

        (, , Channel storage channel, ChannelStatus status) = getChannel(opener, counterParty);

        require(status == ChannelStatus.FUNDED, "HoprChannels: channel must be in 'FUNDED' state");

        channel.stateCounter += 1;

        emit OpenedChannel(opener, counterParty);
    }

    /**
     * @notice redeem ticket
     * @param pre_image bytes32 the value that once hashed produces recipients hashedSecret
     * @param secret_a bytes32 secret
     * @param secret_b bytes32 secret
     * @param amount uint256 amount 'msg.sender' will receive
     * @param win_prob bytes32 win probability
     * @param r bytes32
     * @param s bytes32
     * @param v uint8
     */
    function redeemTicket(
        bytes32 pre_image,
        bytes32 secret_a,
        bytes32 secret_b,
        uint256 amount,
        bytes32 win_prob,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) public {
        require(
            uint256(s) <= HALF_CURVE_ORDER,
            "HoprChannels: found malleable signature, please insert a low-s signature"
        );

        address recipient = msg.sender;
        Account storage recipientAccount = accounts[recipient];

        bytes32 challenge = keccak256(abi.encodePacked(secret_a)) ^ keccak256(abi.encodePacked(secret_b));

        bytes32 hashedTicket = prefixed(
            keccak256(abi.encodePacked(challenge, pre_image, recipientAccount.counter, amount, win_prob))
        );

        require(uint256(hashedTicket) < uint256(win_prob), "HoprChannels: ticket must be a win");

        (address party_a, , Channel storage channel, ChannelStatus status) = getChannel(
            recipient,
            ecrecover(hashedTicket, v, r, s)
        );

        require(
            status == ChannelStatus.OPEN || status == ChannelStatus.PENDING,
            "HoprChannels: channel must be 'OPEN' or 'PENDING'"
        );
        require(amount > 0, "HoprChannels: amount must be strictly greater than zero");
        require(
            recipientAccount.hashedSecret == keccak256(abi.encodePacked(pre_image)),
            "HoprChannels: given value is not a pre-image of the stored on-chain secret"
        );

        recipientAccount.hashedSecret = pre_image;

        if (recipient == party_a) {
            channel.partyABalance = channel.partyABalance.add(amount);
        } else {
            channel.partyABalance = channel.partyABalance.sub(amount);
        }

        require(
            channel.partyABalance <= channel.deposit,
            "HoprChannels: partyABalance must be strictly smaller than deposit balance"
        );
    }

    /**
     * A channel's party can initiate channel closure at any time,
     * it starts a timeout.
     *
     * @notice initiate channel's closure
     * @param counterParty address counter party of 'msg.sender'
     */
    function initiateChannelClosure(address counterParty) external {
        address initiator = msg.sender;

        (, , Channel storage channel, ChannelStatus status) = getChannel(initiator, counterParty);

        require(status == ChannelStatus.OPEN, "HoprChannels: channel must be 'OPEN'");

        channel.closureTime = now + secsClosure;
        channel.stateCounter += 1;

        emit InitiatedChannelClosure(initiator, counterParty, channel.closureTime);
    }

    /**
     * If the timeout is reached without the 'counterParty' reedeming a ticket,
     * then the tokens can be claimed by 'msg.sender'.
     *
     * @notice claim channel's closure
     * @param counterParty address counter party of 'msg.sender'
     */
    function claimChannelClosure(address counterParty) external {
        address initiator = msg.sender;

        (address party_a, address party_b, Channel storage channel, ChannelStatus status) = getChannel(
            initiator,
            counterParty
        );

        require(status == ChannelStatus.PENDING, "HoprChannels: channel must be 'PENDING'");
        require(now >= channel.closureTime, "HoprChannels: 'closureTime' has not passed");

        // settle balances
        if (channel.partyABalance > 0) {
            token.transfer(party_a, channel.partyABalance);
            channel.deposit = channel.deposit.sub(channel.partyABalance);
        }

        if (channel.deposit > 0) {
            token.transfer(party_b, channel.deposit);
        }

        emit ClosedChannel(initiator, counterParty, channel.partyABalance, channel.deposit);

        delete channel.deposit;
        delete channel.partyABalance;
        delete channel.closureTime;
        channel.stateCounter += 7;
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
        address to,
        uint256 amount,
        bytes calldata userData,
        bytes calldata operatorData
    ) external override {
        require(msg.sender == address(token), "HoprChannels: Invalid token");

        // if operator is self (fundWithSignature), don't call fundChannel
        if (operator != address(this)) {
            (address recipient, address counterParty) = abi.decode(userData, (address, address));

            fundChannel(from, recipient, counterParty, amount);
        }
    }

    /**
     * @notice returns channel data
     * @param accountA address of account 'A'
     * @param accountB address of account 'B'
     */
    // TODO: maybe remove this
    function getChannel(address accountA, address accountB)
        internal
        view
        returns (
            address,
            address,
            Channel storage,
            ChannelStatus
        )
    {
        (address party_a, address party_b) = getParties(accountA, accountB);
        bytes32 channelId = getChannelId(party_a, party_b);
        Channel storage channel = channels[channelId];
        ChannelStatus status = getChannelStatus(channel);

        return (party_a, party_b, channel, status);
    }

    /**
     * @notice return true if accountA is party_a
     * @param accountA address of account 'A'
     * @param accountB address of account 'B'
     */
    function isPartyA(address accountA, address accountB) internal pure returns (bool) {
        return uint160(accountA) < uint160(accountB);
    }

    /**
     * @notice return party_a and party_b
     * @param accountA address of account 'A'
     * @param accountB address of account 'B'
     */
    function getParties(address accountA, address accountB) internal pure returns (address, address) {
        if (isPartyA(accountA, accountB)) {
            return (accountA, accountB);
        } else {
            return (accountB, accountA);
        }
    }

    /**
     * @notice return channel id
     * @param party_a address of party 'A'
     * @param party_b address of party 'B'
     */
    function getChannelId(address party_a, address party_b) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(party_a, party_b));
    }

    /**
     * @notice returns 'ChannelStatus'
     * @param channel Channel
     */
    function getChannelStatus(Channel memory channel) internal pure returns (ChannelStatus) {
        return ChannelStatus(channel.stateCounter.mod(10));
    }

    /**
     * @notice builds a prefixed hash to mimic the behavior of eth_sign
     * @param message bytes32 message to prefix
     */
    function prefixed(bytes32 message) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
    }
}
