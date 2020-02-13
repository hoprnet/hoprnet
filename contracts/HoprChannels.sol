/*
    // TODO: move this to readme
    account_a, account_b    => addresses involved in a channel (unsorted)
    party_a, party_b        => addresses involved in a channel (sorted)

    // TODO: meeting notes
    implement: fund channel by using signature
    implement: close channel by using signature
    implement: state_counter
    implement: recovery
    implement: fundAndOpen channel
    implement: reedeemAndInitiateClosure channel
*/
pragma solidity ^0.5.3;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "@openzeppelin/contracts/math/SafeMath.sol";

contract HoprChannels {
    using SafeERC20 for IERC20;
    using SafeMath for uint256;

    // Used to protect against malleable signatures
    uint256 constant HALF_CURVE_ORDER = uint256(0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0);

    // an account has set a new secret hash
    event SecretHashSet(
        address indexed account,
        bytes32 secretHash
    );

    // the payment channel has been funded
    event FundedChannel(
        address indexed funder,
        address indexed recipient,
        address indexed counter_party,
        uint256 recipient_amount,
        uint256 counter_party_amount
    );

    // the payment channel has been opened
    event OpenedChannel(
        address indexed opener,
        address indexed counter_party
    );

    // a party has initiated channel closure
    event InitiatedChannelClosure(
        address indexed initiator,
        address indexed counter_party,
        uint256 closureTime
    );

    // the payment channel has been settled and closed
    event ClosedChannel(
        address indexed closer,
        address indexed counter_party,
        uint256 party_a_amount,
        uint256 party_b_amount
    );

    struct Account {
        bytes32 hashedSecret;   // account's hashedSecret
        uint256 counter;        // increases everytime 'setHashedSecret' is called by the account
    }

    enum ChannelStatus {
        UNINITIALISED,
        CLOSED,  // @TODO. Change this to UNINITIALISED
        FUNDED,
        OPEN,
        PENDING
    }

    // @TODO update this when adding / removing states.
    uint8 constant NUMBER_OF_STATES = 5

    struct Channel {
        uint256 deposit;            // tokens in the deposit
        uint256 party_a_balance;    // tokens that are claimable by party 'A'
        uint256 closureTime;        // the time when the channel can be closed by either party
        uint256 state_counter;      // 0: channel closed
                                    // 1: channel funding
                                    // 2: channel open
                                    // 3: channel pending
    }

    IERC20 public token;        // the token that will be used to settle payments
    uint256 public secsClosure; // seconds it takes to allow closing of channel after channel's -
                                // initiated channel closure

    // store accounts' state
    mapping(address => Account) public accounts;

    // store channels' state e.g: channels[hash(party_a, party_b)]
    mapping(bytes32 => Channel) public channels;

    constructor(IERC20 _token, uint256 _secsClosure) public {
        token = _token;
        secsClosure = _secsClosure;
    }

    /**
     * @notice sets caller's hashedSecret
     * @param hashedSecret bytes32 hashedSecret to store
    */
    function setHashedSecret(bytes32 hashedSecret) external {
        require(hashedSecret != bytes32(0), "hashedSecret must not be empty");

        Account storage account = accounts[msg.sender];
        require(account.hashedSecret != hashedSecret, "new and old hashedSecret must not be the same");

        account.hashedSecret = hashedSecret;
        account.counter = account.counter.add(1);

        emit SecretHashSet(msg.sender, hashedSecret);
    }

    /**
     * Fund a channel between 'account_a' and 'account_b',
     * specified tokens must be approved beforehand.
     *
     * @notice fund a channel
     * @param recipient address account which the funds are for
     * @param counter_party address the counter_party of 'recipient'
     * @param amount uint256 amount to fund the channel
    */
    function fundChannel(address recipient, address counter_party, uint256 amount) public {
        address funder = msg.sender; // account which funds the channel

        require(recipient != counter_party, "'recipient' and 'counter_party' must not be the same");
        require(recipient != address(0), "'recipient' address is empty");
        require(counter_party != address(0), "'counter_party' address is empty");
        require(amount > 0, "'amount' must be greater than 0");

        (
            address party_a,
            ,
            Channel storage channel,
            ChannelStatus status
        ) = getChannel(recipient, counter_party);

        require(
            status == ChannelStatus.CLOSED ||
            status == ChannelStatus.FUNDED,
            "channel is open"
        );

        token.safeTransferFrom(funder, address(this), amount);

        channel.deposit = channel.deposit.add(amount);

        if (recipient == party_a) {
            channel.party_a_balance = channel.party_a_balance.add(amount);
        }

        if (status == ChannelStatus.CLOSED) {
            channel.state_counter = channel.state_counter.add(1);
        }

        emit FundedChannel(funder, recipient, counter_party, amount, 0);
    }

    /**
     * Fund a channel between 'initiator' and 'counter_party',
     * specified tokens must be approved beforehand.
     *
     * @notice fund a channel
     * @param account_a_amount uint256
     * @param counter_party_amount uint256
     * @param r bytes32
     * @param s bytes32
     * @param v uint8
    */
    function fundChannelWithSig(
        uint256 state_counter,
        uint256 deposit,
        uint256 party_a_amount,
        uint256 not_after
        bytes32 r,
        bytes32 s,
        uint8 v
    ) external {
        require(0 < deposit, "Total balance must be strictly greater than zero.");
        require(party_a_amount <= deposit, "Balance of partyA must be strictly smaller than the total_balance.");
        require(uint256(s) <= HALF_CURVE_ORDER, "Found malleable signature. Please insert a low-s signature.");
        require(expiry < block.timestamp, "Signature must not be expired.")

        // struct SignedRequest {
        //     uint256 state_counter // current state
        //     address initiator // address of the initiator
        //     uint256 deposit // total balance
        //     uint256 party_a_balance // balance of partyA
        //     uint256 not_after // expiry timestamp
        // }

        bytes32 hashedMessage = keccak256(abi.encodePacked(state_counter, msg.sender, deposit, party_a_amount, not_after));
        address counterparty = ecrecover(hashedMessage, v, r, s);

        require(counterparty != msg.sender, "Initiator and counterparty must not be the same.");

        bytes32 channelId = getChannelId(msg.sender, counterparty);
        Channel storage channel = channels[channelId];

        require(channel.state_counter == state_counter, "Stored state_counter and given state_counter must be the same.");
        require(status == ChannelStatus.UNINITIALISED, "Channel must be UNINITIALISED.");

        // not final yet
        token.safeTransferFrom(initiator, address(this), initiator_amount);
        token.safeTransferFrom(counter_party, address(this), counter_party_amount);
        // -------------

        channel.deposit = channel.deposit.add(initiator_amount).add(counter_party_amount);

        if (isPartyA(msg.sender, counterparty)) {
            channel.party_a_balance = channel.party_a_balance.add(initiator_amount);
        } else {
            channel.party_a_balance = channel.party_a_balance.add(counter_party_amount);
        }

        channel.state_counter = channel.state_counter.add(1);

        emit FundedChannel(
            address(0),
            initiator,
            counter_party,
            initiator_amount,
            counter_party_amount
        );
    }

    /**
     * @notice open a channel
     * @param counter_party address the counter_party of 'msg.sender'
    */
    function openChannel(address counter_party) public {
        address opener = msg.sender;

        require(opener != counter_party, "'opener' and 'counter_party' must not be the same");
        require(opener != address(0), "'opener' address is empty");
        require(counter_party != address(0), "'counter_party' address is empty");

        (
            ,
            ,
            Channel storage channel,
            ChannelStatus status
        ) = getChannel(opener, counter_party);
 
        require(status == ChannelStatus.FUNDED, "channel was not funded");

        channel.state_counter = channel.state_counter.add(1);

        emit OpenedChannel(opener, counter_party);
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
        address recipient = msg.sender;
        Account storage recipientAccount = accounts[recipient];

        bytes32 challenge = 
            keccak256(abi.encodePacked(secret_a)) ^ keccak256(abi.encodePacked(secret_b));

        // @Steven additional (double-) hashing is not necessary here
        bytes32 hashedTicket = prefixed(abi.encodePacked(
            challenge,
            pre_image,
            recipientAccount.counter,
            amount,
            win_prob
        ));

        require(uint256(hashedTicket) < uint256(win_prob), "ticket must be a win");

        (
            address party_a,
            ,
            Channel storage channel,
            ChannelStatus status
        ) = getChannel(recipient, ecrecover(hashedTicket, v, r, s));

        require(
            status == ChannelStatus.OPEN ||
            status == ChannelStatus.PENDING,
            "channel must be 'open' or 'pending for closure'"
        );
        require(amount > 0, "amount must be strictly greater than zero");
        require(
            recipientAccount.hashedSecret == keccak256(abi.encodePacked(pre_image)),
            "given value is not a pre-image of the stored on-chain secret"
        );

        recipientAccount.hashedSecret = pre_image;

        if (recipient == party_a) {
            channel.party_a_balance = channel.party_a_balance.add(amount);
        } else {
            channel.party_a_balance = channel.party_a_balance.sub(amount);
        }

        require(
            channel.party_a_balance <= channel.deposit,
            "party_a_balance must be strictly lesser than deposit balance"
        );
    }

    /**
     * A channel's party can initiate channel closure at any time,
     * it starts a timeout.
     *
     * @notice initiate channel's closure
     * @param counter_party address counter party of 'msg.sender'
    */
    function initiateChannelClosure(address counter_party) external {
        address initiator = msg.sender;

        (
            ,
            ,
            Channel storage channel,
            ChannelStatus status
        ) = getChannel(initiator, counter_party);

        require(status == ChannelStatus.OPEN, "channel is not open");

        channel.closureTime = now + secsClosure;
        channel.state_counter = channel.state_counter.add(1);

        emit InitiatedChannelClosure(initiator, counter_party, channel.closureTime);
    }

    /**
     * If the timeout is reached without the 'counter_party' reedeming a ticket,
     * then the tokens can be claimed by 'msg.sender'.
     *
     * @notice claim channel's closure
     * @param counter_party address counter party of 'msg.sender'
    */
    function claimChannelClosure(address counter_party) external {
        address initiator = msg.sender;

        (
            address party_a,
            address party_b,
            Channel storage channel,
            ChannelStatus status
        ) = getChannel(initiator, counter_party);

        require(
            status == ChannelStatus.PENDING,
            "channel is not pending for closure"
        );
        require(now >= channel.closureTime, "'closureTime' has not passed");

        // settle balances
        if (channel.party_a_balance > 0) {
            token.safeTransfer(party_a, channel.party_a_balance);
            channel.deposit = channel.deposit.sub(channel.party_a_balance);
        }

        if (channel.deposit > 0) {
            token.safeTransfer(party_b, channel.deposit);
        }

        emit ClosedChannel(initiator, counter_party, channel.party_a_balance, channel.deposit);

        channel.deposit = 0;
        channel.party_a_balance = 0;
        channel.closureTime = 0;
        channel.state_counter = channel.state_counter.add(1);
    }

    // function closeChannelWithSig

    /**
     * @notice returns channel data
     * @param account_a address of account 'A'
     * @param account_b address of account 'B'
    */
    // TODO: maybe remove this
    function getChannel(address account_a, address account_b)
    internal view returns (address, address, Channel storage, ChannelStatus) {
        (address party_a, address party_b) = getParties(account_a, account_b);
        bytes32 channelId = getChannelId(party_a, party_b);
        Channel storage channel = channels[channelId];
        ChannelStatus status = getChannelStatus(channel);

        return (
            party_a,
            party_b,
            channel,
            status
        );
    }

    /**
     * @notice return true if account_a is party_a
     * @param account_a address of account 'A'
     * @param account_b address of account 'B'
    */
    function isPartyA(address account_a, address account_b) internal pure returns (bool) {
        return uint160(account_a) < uint160(account_b);
    }

    /**
     * @notice return party_a and party_b
     * @param account_a address of account 'A'
     * @param account_b address of account 'B'
    */
    function getParties(address account_a, address account_b) internal pure returns (address, address) {
        if (isPartyA(account_a, account_b)) {
            return (account_a, account_b);
        } else {
            return (account_b, account_a);
        }
    }

    /**
     * @notice return channel id
     * @param party_a address of party 'A'
     * @param party_b address of party 'B'
    */
    function getChannelId(address party_a, address party_b) internal pure returns (bytes32) {
        if (isPartyA(party_a)) {
            return keccak256(abi.encodePacked(party_a, party_b));
        } else {
            return keccak256(abi.encodePacked(party_b, party_a));
        }
    }

    /**
     * @notice returns 'ChannelStatus'
     * @param channel Channel
    */
    function getChannelStatus(Channel memory channel) internal pure returns (ChannelStatus) {
        return ChannelStatus(channel.state_counter.mod(4));
    }

    /**
     * @notice builds a prefixed hash to mimic the behavior of eth_sign
     * @param message bytes32 message to prefix
    */
    function prefixed(bytes32 message) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", message));
    }
}