// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity ^0.6.0;

import "./Accounts.sol";
import "./Channels.sol";
import "../utils/ECDSA.sol";

contract Tickets is Accounts, Channels {
    /**
     * @dev Stored hashes of tickets keyed by their challenge,
     * true if ticket has been redeemed.
     */
    mapping(bytes32 => bool) public tickets;

    /**
     * @dev Redeem a ticket
     * @param recipient the recipient address
     * @param counterparty the counterparty address
     * @param secretPreImage the secretPreImage that results to the recipients account secret
     * @param proofOfRelaySecret the proof of relay secret
     * @param winProb the winning probability of the ticket
     * @param amount the amount in the ticket
     * @param r part of the signature
     * @param s part of the signature
     * @param v part of the signature
     */
    function _redeemTicket(
        address recipient,
        address counterparty,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        uint256 amount,
        bytes32 winProb,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) internal {
        (,,, Channel storage channel) = _getChannel(
            recipient,
            counterparty
        );
        require(
            _getChannelStatus(channel.status) != ChannelStatus.CLOSED,
            "channel must be open or pending to close"
        );

        Account storage account = accounts[recipient];
        require(
            account.secret == keccak256(abi.encodePacked(secretPreImage)),
            "secretPreImage must be the hash of secret"
        );

        bytes32 ticketHash = _getTicketHash(
            recipient,
            account.counter,
            proofOfRelaySecret,
            _getChannelIteration(channel.status),
            amount,
            winProb
        );
        require(!tickets[ticketHash], "ticket must not be used twice");
        require(ECDSA.recover(ticketHash, r, s, v) == counterparty, "signer must match the counterparty");
        require(
            uint256(_getTicketLuck(
                ticketHash,
                secretPreImage,
                proofOfRelaySecret,
                winProb
            )) <= uint256(winProb),
            "ticket must be a win"
        );

        account.secret = secretPreImage;
        tickets[ticketHash] = true;

        if (_isPartyA(recipient, counterparty)) {
            // @TODO: add SafeMath
            channel.partyABalance += amount;
        } else {
            // @TODO: add SafeMath
            channel.partyABalance -= amount;
        }
    }

    function _getTicketHash(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        bytes32 winProb
    ) internal pure returns (bytes32) {
        bytes32 challenge = keccak256(abi.encodePacked(proofOfRelaySecret));

        return ECDSA.toEthSignedMessageHash(
            "109",
            abi.encodePacked(
                recipient,
                challenge,
                recipientCounter,
                amount,
                winProb,
                channelIteration
            )
        );
    }

    function _getTicketLuck(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(ticketHash, secretPreImage, proofOfRelaySecret));
    }
}