// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import "../HoprChannels.sol";

contract TicketsMock is HoprChannels {
    constructor(address _token, uint32 _secsClosure)
    HoprChannels(_token, _secsClosure) {}
  
    function fundChannelInternal(
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        _fundChannel(accountA, accountB, amountA, amountB);
    }

    function initiateChannelClosureInternal(
        address initiator,
        address counterparty
    ) external {
        _initiateChannelClosure(initiator, counterparty);
    }

    function finalizeChannelClosureInternal(
        address initiator,
        address counterparty
    ) external {
        _finalizeChannelClosure(initiator, counterparty);
    }

    function redeemTicketInternal(
        address recipient,
        address counterparty,
        bytes32 nextCommitment,
        uint256 ticketEpoch,
        uint256 ticketIndex,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes memory signature
    ) external {
        _redeemTicket(
            recipient,
            counterparty,
            nextCommitment,
            ticketEpoch,
            ticketIndex,
            proofOfRelaySecret,
            amount,
            winProb,
            signature
        );
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
