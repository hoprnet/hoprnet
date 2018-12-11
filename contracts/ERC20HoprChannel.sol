pragma solidity ^0.5.0;

// import "openzeppelin-solidity/contracts/cryptography/ECDSA.sol";
import "./math/SafeMath.sol";
import "./token/ERC20/IERC20.sol";


/* solhint-disable max-line-length */
contract HoprChannel {
    using SafeMath for uint256;
    // using ECDSA for bytes32;

    // ERC20 Token used to pay the HOPR fees
    IERC20 public token;
    
    // constant RELAY_FEE = 1
    uint8 constant private BLOCK_CONFIRMATION = 15;
    
    // Tell payment channel partners that the channel has been settled
    event SettledChannel(bytes32 channelId, uint256 nonce);

    // Tell payment channel partners that the channel has been opened
    event OpenedChannel(bytes32 channelId, uint256 amount);

    // Track the state of the channels
    enum ChannelState {
        UNINITIALIZED, // 0
        PARTYA_FUNDED, // 1
        PARTYB_FUNDED, // 2
        ACTIVE, // 3
        PENDING_SETTLEMENT, // 4
        WITHDRAWN // 5
    }

    struct Channel {
        ChannelState state;
        uint256 balance;
        uint256 balanceA;
        uint256 index;
        uint256 settlementBlock;
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
        uint256 stakedFunds;
        int32 counter;
    }

    // Keeps track of the states of the
    // participating parties.
    mapping(address => State) public states;

    modifier enoughFunds(uint256 amount) {
        require(amount <= states[msg.sender].stakedFunds, "Insufficient funds.");
        _;
    }

    modifier channelExists(address counterParty) {
        bytes32 channelId = getId(counterParty);
        Channel memory channel = channels[channelId];
        
        require(channel.state > ChannelState.UNINITIALIZED && channel.state < ChannelState.WITHDRAWN, "Channel does not exist.");
        _;
    }

    /**
    * @notice configures the ERC20 token variable
    * @param _token address of the ERC20 token
    */
    constructor(address _token) public {
        require(_token != address(0), "Invalid address");

        token = IERC20(_token);
    }

    /**
    * @notice desposit tokens to stake
    */
    function stakeFunds(uint256 amount) public {
        require(amount > 0, "Please provide a non-zero amount of tokens.");
        
        states[msg.sender].isSet = true;
        states[msg.sender].stakedFunds = states[msg.sender].stakedFunds.add(amount);
    }
    
    /**
    * @notice withdrawal staked tokens
    * @param amount uint256
    */
    function unstakeFunds(uint256 amount) public enoughFunds(amount) {
        require(states[msg.sender].openChannels == 0, "Waiting for remaining channels to close.");
        
        if (amount == states[msg.sender].stakedFunds) {
            delete states[msg.sender];
        } else {
            states[msg.sender].stakedFunds = states[msg.sender].stakedFunds.sub(amount);
        }

        // msg.sender.transfer(amount);
        token.transfer(msg.sender, amount);
    }

    function getStakedAmount(address _address) public view returns (uint256) {
        return states[_address].stakedFunds;
    }
    
    /**
    * @notice create payment channel TODO: finish desc
    * @param counterParty address of the counter party
    * @param amount uint256
    */
    function create(address counterParty, uint256 amount) public enoughFunds(amount) {
        require(channels[getId(counterParty)].state == ChannelState.UNINITIALIZED, "Channel already exists.");
        
        states[msg.sender].stakedFunds = states[msg.sender].stakedFunds.sub(amount);
        
        // Register the channels at both participants' state
        states[msg.sender].openChannels = states[msg.sender].openChannels.add(1);
        states[counterParty].openChannels = states[counterParty].openChannels.add(1);
        
        if (isPartyA(counterParty)) {
            // msg.sender == partyB
            channels[getId(counterParty)] = Channel(ChannelState.PARTYB_FUNDED, amount, 0, 0, 0);
        } else {
            // msg.sender == partyA
            channels[getId(counterParty)] = Channel(ChannelState.PARTYA_FUNDED, amount, amount, 0, 0);
        }
    }
    
    /**
    * @notice fund payment channel TODO: finish desc
    * @param counterParty address of the counter party
    * @param amount uint256
    */
    function fund(address counterParty, uint256 amount) public enoughFunds(amount) channelExists(counterParty) {
        states[msg.sender].stakedFunds = states[msg.sender].stakedFunds.sub(amount);

        Channel storage channel = channels[getId(counterParty)];

        if (isPartyA(counterParty)) {
            // msg.sender == partyB
            require(channel.state == ChannelState.PARTYA_FUNDED, "Channel already exists.");
            
            channel.balance = channel.balance.add(amount);
        } else {
            // msg.sender == partyA
            require(channel.state == ChannelState.PARTYB_FUNDED, "Channel already exists.");
            
            channel.balance = channel.balance.add(amount);
            channel.balanceA = channel.balanceA.add(amount);
        }
        channel.state = ChannelState.ACTIVE;
    }
    
    /**
    * @notice pre-fund channel with staked tokens of both parties
    * @param counterParty address of the counter party
    * @param amount uint256 how much money both parties put into the channel
    * @param r bytes32 signature first part
    * @param s bytes32 signature second part
    * @param v uint8 version
     */
    function createFunded(address counterParty, uint256 amount, bytes32 r, bytes32 s, bytes1 v) public enoughFunds(amount) {
        require(channels[getId(counterParty)].state == ChannelState.UNINITIALIZED, "Channel already exists.");

        require(states[counterParty].stakedFunds >= amount, "Insufficient funds");

        bytes32 hashedMessage = keccak256(abi.encodePacked(amount, uint256(0), getId(counterParty)));

        require(ecrecover(hashedMessage, uint8(v) + 27, r, s) == counterParty, "Invalid opening transaction");

        states[msg.sender].stakedFunds = states[msg.sender].stakedFunds - amount;
        states[counterParty].stakedFunds = states[counterParty].stakedFunds - amount;

        states[msg.sender].openChannels = states[msg.sender].openChannels + 1;
        states[counterParty].openChannels = states[counterParty].openChannels + 1;
        
        channels[getId(counterParty)] = Channel(ChannelState.ACTIVE, 2 * amount, amount, 0, 0);        
    }

    /**
    * @notice settle payment channel TODO: finish desc
    * @param counterParty address of the counter party
    * @param index uint256
    * @param balanceA uint256
    * @param r bytes32
    * @param s bytes32
    */
    function settle(address counterParty, uint256 index, uint256 balanceA, bytes32 r, bytes32 s) public channelExists(counterParty) {
        bytes32 channelId = getId(counterParty);
        Channel storage channel = channels[channelId];
        
        require(
            channel.index < index &&
            channel.state == ChannelState.PARTYA_FUNDED || channel.state == ChannelState.PARTYB_FUNDED,
            "Invalid channel.");
               
        // is the proof valid?
        bytes32 hashedMessage = keccak256(abi.encodePacked(channelId, balanceA, index));
        require(ecrecover(hashedMessage, 0, r, s) == counterParty, "Invalid signature.");
        
                
        channel.state = ChannelState.PENDING_SETTLEMENT;
        channel.settlementBlock = block.number;
        
        emit SettledChannel(channelId, index);
    }
    
    /**
    * @notice TODO: finish desc
    * @param counterParty address of the counter party
    */
    function withdraw(address counterParty) public channelExists(counterParty) {
        Channel storage channel = channels[getId(counterParty)];
        require(
            channel.state == ChannelState.PENDING_SETTLEMENT && 
            channel.balanceA <= channel.balance, 
            "Invalid channel.");

        require(channel.settlementBlock.add(BLOCK_CONFIRMATION) >= block.number, "Channel not withdrawable yet.");
        
        channel.state = ChannelState.WITHDRAWN;
        
        require(
            states[msg.sender].openChannels > 0 &&
            states[counterParty].openChannels > 0, 
            "Something went wrong. Double spend?");

        states[msg.sender].openChannels = states[msg.sender].openChannels.sub(1);
        states[counterParty].openChannels = states[counterParty].openChannels.sub(1);
        
        if (isPartyA(counterParty)) {
            // msg.sender == partyB
            states[msg.sender].stakedFunds = states[msg.sender].stakedFunds.add((channel.balance.sub(channel.balanceA)));
            states[counterParty].stakedFunds = states[counterParty].stakedFunds.add(channel.balanceA);
        } else {
            // msg.sender == partyA
            states[counterParty].stakedFunds = states[counterParty].stakedFunds.add((channel.balance.sub(channel.balanceA)));
            states[msg.sender].stakedFunds = states[msg.sender].stakedFunds.add(channel.balanceA); 
        }

        delete channels[getId(counterParty)];
    }

    /*** PRIVATE | INTERNAL ***/
    /**
    * @notice compares addresses `msg.sender` vs `counterParty` 
    * @param counterParty address of the counter party
    * @return bool 
    */
    function isPartyA(address counterParty) private view returns (bool) {
        require(msg.sender != counterParty, "Cannot open channel between one party.");

        return bytes20(msg.sender) < bytes20(counterParty);
    }

    /**
    * @notice returns keccak256 hash of `counterParty` & `msg.sender` which is used as an id
    * @dev order of arguements pending result of `isPartyA(counterParty)`
    * @param counterParty address of the counter party
    * @return bytes32 
    */
    function getId(address counterParty) private view returns (bytes32) {
        if (isPartyA(counterParty)) {
            return keccak256(abi.encodePacked(msg.sender, counterParty));
        } else {
            return keccak256(abi.encodePacked(counterParty, msg.sender));
        }
    }
}
