// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import "../HoprChannels.sol";

contract ChannelsMock is HoprChannels {
    constructor(address _token, uint32 _secsClosure)
    HoprChannels(_token, _secsClosure) {}

    function getChannelInternal(
        address accountA,
        address accountB
    ) external view returns (
        address,
        address,
        bytes32
    ) {
        (address partyA, address partyB, bytes32 channelId,) = _getChannel(accountA, accountB);
        return (partyA, partyB, channelId);
    }

    function getChannelIdInternal(
        address partyA,
        address partyB
    ) external pure returns (bytes32) {
        return _getChannelId(partyA, partyB);
    }

    function isPartyAInternal(
        address accountA,
        address accountB
    ) external pure returns (bool) {
        return _isPartyA(accountA, accountB);
    }

    function getPartiesInternal(
        address account1,
        address account2
    ) external pure returns (address, address) {
        return _sortAddresses(account1,account2);
    }

    function getEncodedTicketInternal(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        bytes32 winProb
    ) external pure returns (bytes memory) {
        return _getEncodedTicket(recipient, recipientCounter, proofOfRelaySecret, channelIteration, amount, winProb);
    }

    function getTicketLuckInternal(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) external pure returns (bytes32) {
        return _getTicketLuck(ticketHash, secretPreImage, proofOfRelaySecret, winProb);
    }
}
