// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity ^0.6.0;

import "./AccountsMock.sol";
import "./ChannelsMock.sol";
import "../HoprChannels/Tickets.sol";

contract TicketsMock is AccountsMock, ChannelsMock, Tickets {
    constructor(uint256 _secsClosure) ChannelsMock(_secsClosure) public {}

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

    function getTicketHash(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        bytes32 winProb
    ) external pure {
        _getTicketHash(recipient, recipientCounter, proofOfRelaySecret, channelIteration, amount, winProb);
    }

    function getTicketLuck(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) external pure {
        _getTicketLuck(ticketHash, secretPreImage, proofOfRelaySecret, winProb);
    }
}