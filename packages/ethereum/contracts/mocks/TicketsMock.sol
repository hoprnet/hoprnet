// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "./AccountsMock.sol";
import "./ChannelsMock.sol";
import "../HoprChannels/Tickets.sol";

contract TicketsMock is AccountsMock, ChannelsMock, Tickets {
    constructor(uint32 _secsClosure) ChannelsMock(_secsClosure) {}

    function redeemTicket(
        address recipient,
        address counterparty,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) external {
        _redeemTicket(recipient, counterparty, secretPreImage, proofOfRelaySecret, amount, winProb, r, s, v);
    }

    function getEncodedTicket(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        bytes32 winProb
    ) external pure returns (bytes memory) {
        return _getEncodedTicket(recipient, recipientCounter, proofOfRelaySecret, channelIteration, amount, winProb);
    }

    function getTicketHash(
        bytes calldata packedTicket
    ) external pure returns (bytes32) {
        return _getTicketHash(packedTicket);
    }

    function getTicketLuck(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) external pure returns (bytes32) {
        return _getTicketLuck(ticketHash, secretPreImage, proofOfRelaySecret, winProb);
    }
}