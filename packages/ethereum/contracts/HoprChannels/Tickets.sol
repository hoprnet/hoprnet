// SPDX-License-Identifier: GPL-3.0
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
        Account storage account = accounts[recipient];
        require(
            account.secret == keccak256(abi.encodePacked(secretPreImage)),
            "secretPreImage must be the hash of recipient's secret"
        );

        (,,, Channel storage channel) = _getChannel(
            recipient,
            counterparty
        );
        require(
            _getChannelStatus(channel.status) != ChannelStatus.CLOSED,
            "channel must be open or pending to close"
        );

        bytes32 ticketHash = _getTicketHash(
            _getEncodedTicket(
                recipient,
                account.counter,
                proofOfRelaySecret,
                _getChannelIteration(channel.status),
                amount,
                winProb
            )
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

    /**
     * @dev Encode ticket data
     * @return bytes
     */
    function _getEncodedTicket(
        address recipient,
        uint256 recipientCounter,
        bytes32 proofOfRelaySecret,
        uint256 channelIteration,
        uint256 amount,
        bytes32 winProb
    ) internal pure returns (bytes memory) {
        bytes32 challenge = keccak256(abi.encodePacked(proofOfRelaySecret));

        return abi.encodePacked(
            recipient,
            challenge,
            recipientCounter,
            amount,
            winProb,
            channelIteration
        );
    }

    /**
     * @dev Prefix the ticket message and return
     * the actual hash that was used to sign
     * the ticket with.
     * @return prefixed ticket hash
     */
    function _getTicketHash(
        bytes memory packedTicket
    ) internal pure returns (bytes32) {
        return ECDSA.toEthSignedMessageHash(
            "187",
            packedTicket
        );
    }

    /**
     * @dev Get the ticket's "luck" by
     * hashing provided values.
     * @return luck
     */
    function _getTicketLuck(
        bytes32 ticketHash,
        bytes32 secretPreImage,
        bytes32 proofOfRelaySecret,
        bytes32 winProb
    ) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(ticketHash, secretPreImage, proofOfRelaySecret, winProb));
    }
}