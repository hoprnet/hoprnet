pragma solidity 0.5.0;

// import "github.com/OpenZeppelin/zeppelin-solidity/contracts/cryptography/ECDSA.sol";

contract Hopper {
    // using ECDSA for bytes32;
    
    // constant RELAY_FEE = 1
    uint8 constant private BLOCK_HEIGHT = 15;
    
    // Tell payment channel partners that channel has 
    // been settled
    event ChannelSettled(bytes32 channelId, uint nonce);

    // Track the state of the
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
        uint128 balance;
        uint128 balanceA;
        uint32 index;
        uint settlementBlock;
    }

    // Open channels
    mapping(bytes32 => Channel) private channels;
    
    struct State {
        bool isSet;
        // number of open channels
        // Note: the smart contract doesn't the actual
        //       channels but it knows how many open ones
        //       there are.
        uint16 openChannels;
        uint128 stakedMoney;
        int32 counter;
    }

    // Keeps track of the states of the
    // participating parties.
    mapping(address => State) private states;

    function stakeMoney() public payable {
        require(msg.value > 0, "Please provide a non-zero amount of money.");
        
        states[msg.sender].isSet = true;
        states[msg.sender].stakedMoney = states[msg.sender].stakedMoney + uint128(msg.value);
    }
    
    /* Dummy */
    function howMuchMoney() public view returns (uint256) {
        return states[msg.sender].stakedMoney;
    }
    
    function unstakeMoney(uint128 amount) public {
        require(states[msg.sender].openChannels == 0, "Waiting for remaining channels to close.");
        require(amount <= states[msg.sender].stakedMoney, "Insufficient funds.");
        
        states[msg.sender].stakedMoney = states[msg.sender].stakedMoney - amount;
        
        msg.sender.transfer(amount);
    }
    
    function create(address counterParty, uint128 amount) public {
        require(channels[getId(counterParty)].state == ChannelState.UNINITIALIZED, "Channel already exists.");
        require(states[msg.sender].isSet && states[msg.sender].stakedMoney >= amount, "Insufficient funds.");
        
        states[msg.sender].stakedMoney = states[msg.sender].stakedMoney - amount;
        
        // Register the channels at both participants' state
        states[msg.sender].openChannels = states[msg.sender].openChannels + 1;
        states[counterParty].openChannels = states[counterParty].openChannels + 1;
        
        if (isPartyA(counterParty)) {
            // msg.sender == partyB
            channels[getId(counterParty)] = Channel(ChannelState.PARTYB_FUNDED, amount, 0, 0, 0);
        } else {
            // msg.sender == partyA
            channels[getId(counterParty)] = Channel(ChannelState.PARTYA_FUNDED, amount, amount, 0, 0);
        }
    }

    function isOpen(address counterParty) public view returns (bool) {
        return channels[getId(counterParty)].state < ChannelState.UNINITIALIZED && channels[getId(counterParty)].state < ChannelState.WITHDRAWN;
    }
    
    function fund(address counterParty, uint128 amount) public {
        State storage state = states[msg.sender];
        require(
            state.isSet && 
            state.stakedMoney >= amount,
            "Insufficient funds");

        states[msg.sender].stakedMoney = states[msg.sender].stakedMoney - amount;

        Channel storage channel = channels[getId(counterParty)];
        if (isPartyA(counterParty)) {
            // msg.sender == partyB
            require(
                channel.state == ChannelState.PARTYA_FUNDED, 
                "Channel already exists.");
            
            channel.balance = channel.balance + amount;
        } else {
            // msg.sender == partyA
            require(
                channel.state == ChannelState.PARTYB_FUNDED, 
                "Channel already exists.");
            
            channel.balance = channel.balance + amount;
            channel.balanceA = channel.balanceA + amount;
        }
        channel.state = ChannelState.ACTIVE;
    }
    
    function settle(address counterParty, uint32 index, uint128 balanceA, bytes32 r, bytes32 s) public {
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
        
        emit ChannelSettled(channelId, index);
    }
    
    function withdraw(address counterParty) public {
        Channel storage channel = channels[getId(counterParty)];
        require(
            channel.state == ChannelState.PENDING_SETTLEMENT && 
            channel.balanceA <= channel.balance, 
            "Invalid channel.");

        require(
            channel.settlementBlock + BLOCK_HEIGHT <= channel.settlementBlock + block.number, 
            "Channel not withdrawable yet.");
        
        channel.state = ChannelState.WITHDRAWN;
        
        require(
            states[msg.sender].openChannels > 0 &&
            states[counterParty].openChannels > 0, 
            "Something went wrong. Double spend?");

        states[msg.sender].openChannels = states[msg.sender].openChannels - 1;
        states[counterParty].openChannels = states[counterParty].openChannels - 1;
        
        if (isPartyA(counterParty)) {
            // msg.sender == partyB
            states[msg.sender].stakedMoney = states[msg.sender].stakedMoney + (channel.balance - channel.balanceA);
            states[counterParty].stakedMoney = states[counterParty].stakedMoney + channel.balanceA;
        } else {
            // msg.sender == partyA
            states[counterParty].stakedMoney = states[counterParty].stakedMoney + (channel.balance - channel.balanceA);
            states[msg.sender].stakedMoney = states[msg.sender].stakedMoney + channel.balanceA; 
        }
        
        delete channels[getId(counterParty)];
    }
    
    function isPartyA(address counterParty) private view returns (bool) {
        require(msg.sender != counterParty, "Cannot open channel between one party.");

        return bytes20(msg.sender) < bytes20(counterParty);
    }
    
    function getId(address partyB) private view returns (bytes32) {
        if (isPartyA(partyB)) {
            return keccak256(abi.encodePacked(msg.sender, partyB));
        } else {
            return keccak256(abi.encodePacked(partyB, msg.sender));
        }
    }
    
}