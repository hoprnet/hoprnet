pragma solidity ^0.5.0;

/// @title A simple payment channel mock
/// @author Sebastian C. Buergel
/// @notice Use this smart contract to test opening and closing of payment channels without any tokens involved
/// @dev Just a mock contract, intended to test the event readout - thus do NOT read any storage variables as those will not be present in the actual implementation
contract mockChannel {
    
    event openedChannel(bytes32 indexed channelId, uint256 amount, uint256 time);
    
    event closedChannel(bytes32 indexed channelId, uint256 amount, uint256 time);
    
    struct channel {
        address partyA;
        address partyB;
        uint256 amount;
        bool isOpen;
    }
    
    mapping(bytes32 => channel) public Channels;
    
    uint256 public totalAmount; // just for debugging purposes - UI should not read this value as it will not be used in actual channel implementation!
    uint256 public numChannels; // just for debugging purposes - UI should not read this value as it will not be used in actual channel implementation!
    
    function open(address partyB, uint256 amount) public {
        bytes32 channelId = keccak256(abi.encodePacked(msg.sender, partyB));
        if (!Channels[channelId].isOpen) {
            Channels[channelId].partyA = msg.sender;
            Channels[channelId].partyB = partyB;
            numChannels++;
            emit openedChannel(channelId, amount, block.timestamp);
        }
        Channels[channelId].amount += amount;
        totalAmount += amount;
    }
    
    function close(bytes32 channelId) public {
        if (Channels[channelId].isOpen) {
            emit closedChannel(channelId, Channels[channelId].amount, block.timestamp);
            totalAmount -= Channels[channelId].amount;
            numChannels--;
            Channels[channelId].isOpen = false;
            Channels[channelId].amount = 0;
        }
    }
}
