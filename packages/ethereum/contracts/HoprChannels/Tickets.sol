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
        _validateAccountPreImage(recipient, secretPreImage);
        (,,, Channel storage channel) = _getChannel(
            recipient,
            counterparty
        );
        _validateChannelStatus(channel);

        bytes32 ticketHash = _getTicketHash(
            recipient,
            accounts[recipient].counter,
            proofOfRelaySecret,
            _getChannelIteration(channel.status),
            amount,
            winProb
        );
        _validateTicketHash(ticketHash, counterparty, r, s, v);
        _validateLuck(ticketHash, secretPreImage, proofOfRelaySecret, winProb);

        accounts[recipient].secret = secretPreImage;
        tickets[ticketHash] = true;

        if (_isPartyA(recipient, counterparty)) {
            // @TODO: add SafeMath
            channel.partyABalance += amount;
        } else {
            // @TODO: add SafeMath
            channel.partyABalance -= amount;
        }
    }

    function _validateAccountPreImage(
        address account,
        bytes32 secretPreImage
    ) internal {
        require(
            accounts[account].secret == keccak256(abi.encodePacked(secretPreImage)),
            "secretPreImage must be the hash of account's secret"
        );
    }

    function _validateChannelStatus(
        Channel memory channel
    ) internal {
        ChannelStatus channelStatus = _getChannelStatus(channel.status);
        require(
            channelStatus == ChannelStatus.OPEN || channelStatus == ChannelStatus.PENDING_TO_CLOSE,
            "channel must be open or pending to close"
        );
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

    function _validateTicketHash(
        bytes32 ticketHash,
        address counterparty,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) internal {
        require(ECDSA.recover(ticketHash, r, s, v) == counterparty, "signer must match the counterparty");
        require(!tickets[ticketHash], "ticket must not be used twice");
    }

    function _validateLuck(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) internal {
        bytes32 luck = keccak256(abi.encodePacked(ticketHash, secretPreImage, proofOfRelaySecret));
        require(uint256(luck) <= uint256(winProb), "ticket must be a win");
    }
}