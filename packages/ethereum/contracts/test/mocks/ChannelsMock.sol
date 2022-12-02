// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import "../../src/HoprChannels.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract ChannelsMockTest is HoprChannels {
    constructor(address _token, uint32 _secsClosure)
    HoprChannels(_token, _secsClosure) {}

    function getChannelIdInternal(
        address partyA,
        address partyB
    ) external pure returns (bytes32) {
        return _getChannelId(partyA, partyB);
    }

    function getEncodedTicketInternal(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        uint256 ticketIndex,
        uint256 winProb
    ) external pure returns (bytes memory) {
        return _getEncodedTicket(recipient, recipientCounter, proofOfRelaySecret, channelIteration, amount, ticketIndex, winProb);
    }

    function getTicketLuckInternal(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret
    ) external pure returns (uint256) {
        return _getTicketLuck(ticketHash, secretPreImage, proofOfRelaySecret);
    }

    function getTicketHashInternal(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        uint256 ticketIndex,
        uint256 winProb
    ) external pure returns (bytes32) {
        return ECDSA.toEthSignedMessageHash(
            keccak256(_getEncodedTicket(recipient, recipientCounter, proofOfRelaySecret, channelIteration, amount, ticketIndex, winProb))
        );
    }

    function computeChallengeInternal(bytes32 response) external pure returns (address) {
        return _computeChallenge(response);
    }
}
