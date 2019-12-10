pragma solidity ^0.5.3;

import "@openzeppelin/contracts/math/SafeMath.sol";

contract HoprChannel {
    using SafeMath for uint256;
    
    uint256 constant private TIME_CONFIRMATION = 1 minutes; // testnet value TODO: adjust for mainnet use
    
    // To inform the payment channel partners that the channel has been settled and closed.
    event ClosedChannel(bytes32 indexed channelId, bytes16 index, uint256 amountA);

    // To inform the payment channel partners that the channel has been opened.
    event OpenedChannel(bytes32 indexed channelId, uint256 amountA, uint256 amount);

    // To inform the payment channel partners that another party has opened a payment channel for them.
    event OpenedChannelFor(address indexed partyA, address indexed partyB, uint256 amountA, uint256 amount);

    // To inform a party that it should send its acknowledgement.
    event Challenge(bytes32 indexed challenge) anonymous;

    // Track the state of the channels
    enum ChannelState {
        UNINITIALIZED, // 0
        PARTYA_FUNDED, // 1
        PARTYB_FUNDED, // 2
        ACTIVE, // 3
        PENDING_SETTLEMENT // 4
    }

    struct Channel {
        ChannelState state;
        uint256 balance;
        uint256 balanceA;
        bytes16 index;
        uint256 settleTimestamp;
    }

    // Open channels
    mapping(bytes32 => Channel) public channels;

    struct State {
        bool isSet;
        // number of open channels
        // Note: the smart contract doesn't know the actual
        //       channels but it knows how many open ones
        //       there are.
        uint256 openChannels;
        uint256 stakedEther;
    }

    // Keeps track of the states of the
    // participating parties.
    mapping(address => State) public states;

    // Keeps track of the nonces that have
    // been used to open payment channels
    mapping(bytes16 => bool) public nonces;

    modifier enoughFunds(uint256 amount) {
        require(amount <= states[msg.sender].stakedEther, "Insufficient funds.");
        _;
    }

    /**
     * @notice desposit ether to stake
     */
    function() external payable {
        stakeFor(msg.sender);
    }

    /**
     * Stake funds for another party.
     *
     * @param beneficiary address the benificiary who receives the stake
     */
    function stakeFor(address beneficiary) payable public {
        if (msg.value > 0) {
            if (!states[beneficiary].isSet) {
                states[beneficiary] = State(true, 0, msg.value);
            } else {
                states[beneficiary].stakedEther = states[beneficiary].stakedEther.add(uint256(msg.value));
            }
        }
    }   

    /**
     * Creates a payment channel record in behalf for two channel parties.
     *
     * @notice The initiator does not need to be among the two channel parties.
     *
     * @param partyA address first party
     * @param partyB address second party
     */
    function createFor(address partyA, address partyB) payable public {
        bytes32 channelId = getId(partyA, partyB);

        uint256 funds = msg.value;

        require(funds > 1, "Cannot spread one wei uniformly over two parties.");

        if (funds % 2 == 1) {
            funds = funds - 1;
        }

        require(channels[channelId].state == ChannelState.UNINITIALIZED, "Channel already exists.");

        if (!states[partyA].isSet) {
            states[partyA] = State(true, 1, 0);
        } else {
            states[partyA].openChannels = states[partyA].openChannels + 1;
        }

        if (!states[partyB].isSet) {
            states[partyB] = State(true, 1, 0);
        } else {
            states[partyB].openChannels = states[partyB].openChannels + 1;
        }

        uint256 amount = funds;
        uint256 amountA = funds / 2;

        channels[channelId] = Channel(ChannelState.ACTIVE, amount, amountA, 0, 0);

        emit OpenedChannelFor(partyA, partyB, amountA, amount);
    }
    
    /**
     * @notice withdrawal staked ether
     * @param amount uint256
     */
    function unstakeEther(uint256 amount) public enoughFunds(amount) {
        require(states[msg.sender].openChannels == 0, "Waiting for remaining channels to close.");

        if (amount == states[msg.sender].stakedEther) {
            delete states[msg.sender];
        } else {
            states[msg.sender].stakedEther = states[msg.sender].stakedEther.sub(amount);
        }

        msg.sender.transfer(amount);
    } 

    /**
     * @notice pre-fund channel by with staked Ether of both parties
     * @param nonce nonce to prevent against replay attacks
     * @param funds uint256 how much money both parties put into the channel
     * @param r bytes32 signature first part
     * @param s bytes32 signature second part
     * @param v uint8 version
     */
    function createFunded(bytes16 nonce, uint256 funds, bytes32 r, bytes32 s, uint8 v) external enoughFunds(funds) {
        require(!nonces[nonce], "Nonce was already used.");
        nonces[nonce] = true;

        bytes32 hashedMessage = keccak256(abi.encodePacked(nonce, uint128(1), funds, bytes32(0), bytes1(0)));
        address counterparty = ecrecover(hashedMessage, v, r, s);
        
        require(states[counterparty].stakedEther >= funds, "Insufficient funds.");

        bytes32 channelId = getId(msg.sender, counterparty);

        require(channels[channelId].state == ChannelState.UNINITIALIZED, "Channel already exists.");

        states[msg.sender].stakedEther = states[msg.sender].stakedEther.sub(funds);
        states[counterparty].stakedEther = states[counterparty].stakedEther.sub(funds);

        states[msg.sender].openChannels = states[msg.sender].openChannels.add(1);
        states[counterparty].openChannels = states[counterparty].openChannels.add(1);
        
        uint256 totalBalance = uint256(2).mul(funds);
        channels[channelId] = Channel(ChannelState.ACTIVE, totalBalance, funds, 0, 0);

        emit OpenedChannel(channelId, funds, totalBalance);        
    }

    /**
     * Initiates the closing procedure of a payment channel. The sender of the transaction
     * proposes a possible settlement transaction and opens a time interval that the
     * counterparty can use to propose a more recent update transaction which leads to a reset
     * of the time interval.
     * Once the time interval is over, any of the parties can call the `withdraw` function
     * which finally payout the money.
     *
     * @notice settle & close payment channel
     * @param index bytes16
     * @param nonce bytes16
     * @param balanceA uint256
     * @param r bytes32
     * @param s bytes32
     * @param v bytes1
     */
    function closeChannel(
        bytes16 index,
        bytes16 nonce,
        uint256 balanceA,
        bytes32 curvePointFirst,
        bytes1 curvePointSecond,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) public {
        bytes32 hashedMessage = keccak256(abi.encodePacked(nonce, index, balanceA, curvePointFirst, curvePointSecond));
        address counterparty = ecrecover(hashedMessage, v, r, s);

        bytes32 channelId = getId(msg.sender, counterparty);
        Channel storage channel = channels[channelId];
        
        require(
            channel.index < index &&
            channel.state == ChannelState.ACTIVE || channel.state == ChannelState.PENDING_SETTLEMENT,
            "Unable to close payment channel due to probably invalid signature.");
        
        channel.balanceA = balanceA;
        channel.index = index;
        channel.state = ChannelState.PENDING_SETTLEMENT;
        channel.settleTimestamp = block.timestamp.add(TIME_CONFIRMATION);
        
        emit ClosedChannel(channelId, index, balanceA);
    }

    /**
     * The purpose of this method is to give the relayer the opportunity to claim the funds
     * in case the next downstream node responds with an invalid acknowledgement.
     *
     * @param rChallenge bytes32 first part of challenge signature
     * @param sChallenge bytes32 second part of challenge signature
     * @param rResponse bytes32 first part of response signature
     * @param sResponse bytes32 second part of response signature
     * @param keyHalf bytes32 
     * @param vChallenge uint8 version (either 27 or 28)
     * @param vResponse uint8 version (either 27 or 28)
     */
    function wrongAcknowledgement(
        bytes32 rChallenge,
        bytes32 sChallenge,
        bytes32 rResponse, 
        bytes32 sResponse,
        bytes32 keyHalf,
        // uint256 amount,
        uint8 vChallenge,
        uint8 vResponse
    ) view external {
        // solhint-disable-next-line max-line-length
        bytes32 hashedAcknowledgement = keccak256(abi.encodePacked(rChallenge, sChallenge, vChallenge, keyHalf));

        address counterparty = ecrecover(hashedAcknowledgement, vResponse, rResponse, sResponse);
        Channel storage channel = channels[getId(msg.sender, counterparty)];

        require(channel.state != ChannelState.UNINITIALIZED, "Invalid channel 123.");

        bytes32 hashedKeyHalf = keccak256(abi.encodePacked(keyHalf));

        // solhint-disable-next-line max-line-length
        require(msg.sender != ecrecover(hashedKeyHalf, vChallenge, rChallenge, sChallenge), "Trying to claim funds with a valid acknowledgement.");

        // states[counterparty].stakedEther = states[counterparty].stakedEther.sub(amount);
        // states[msg.sender].stakedEther = states[msg.sender].stakedEther.add(amount);
    }
    
    /**
     * @notice withdrawal pending balance from payment channel
     * @param counterParty address of the counter party
     */
    function withdraw(address counterParty) external {
        bytes32 channelId = getId(msg.sender, counterParty);
        Channel storage channel = channels[channelId];
        
        require(channel.state == ChannelState.PENDING_SETTLEMENT, "Channel is not withdrawable.");

        require(channel.settleTimestamp <= block.timestamp, "Channel not withdrawable yet.");
                        
        require(
            states[msg.sender].openChannels > 0 &&
            states[counterParty].openChannels > 0, 
            "Something went wrong. Double spend?");

        states[msg.sender].openChannels = states[msg.sender].openChannels.sub(1);
        states[counterParty].openChannels = states[counterParty].openChannels.sub(1);
        
        if (isPartyA(msg.sender, counterParty)) {
            // msg.sender == partyB
            // solhint-disable-next-line max-line-length
            states[msg.sender].stakedEther = states[msg.sender].stakedEther.add((channel.balance.sub(channel.balanceA)));
            states[counterParty].stakedEther = states[counterParty].stakedEther.add(channel.balanceA);
        } else {
            // msg.sender == partyA
            // solhint-disable-next-line max-line-length
            states[counterParty].stakedEther = states[counterParty].stakedEther.add((channel.balance.sub(channel.balanceA)));
            states[msg.sender].stakedEther = states[msg.sender].stakedEther.add(channel.balanceA); 
        }

        delete channels[channelId];
    }

    function isPartyA(address a, address b) private pure returns (bool) {
        require(a != b, "Party 'a' and party 'b' must not be equal.");

        return bytes20(a) < bytes20(b);
    }
    function getId(address a, address b) private pure returns (bytes32) {
        require(a != b, "Party 'a' and party 'b' must not be equal.");

        if (isPartyA(a, b)) {
            return keccak256(abi.encodePacked(a, b));
        } else {
            return keccak256(abi.encodePacked(b, a));
        }
    }

    /**
     * Adds two secp256k1 points together
     */
    // function ecadd

    /**
     * Multiplies a secp256k1 point by a cofactor
     */
    // function ecmul
}