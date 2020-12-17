// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./Accounts.sol";
import "./Channels.sol";
import "../utils/ECDSA.sol";

contract Tickets is Accounts, Channels {
    using SafeMath for uint256;

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
        require(recipient != address(0), "recipient must not be empty");
        require(counterparty != address(0), "counterparty must not be empty");
        require(secretPreImage != bytes32(0), "secretPreImage must not be empty");
        require(proofOfRelaySecret != bytes32(0), "proofOfRelaySecret must not be empty");
        require(amount != uint256(0), "amount must not be empty");
        // require(winProb != bytes32(0), "winProb must not be empty");
        require(r != bytes32(0), "r must not be empty");
        require(s != bytes32(0), "s must not be empty");
        require(v != uint8(0), "v must not be empty");

        Account storage account = accounts[recipient];
        require(
            account.secret == keccak256(abi.encodePacked(secretPreImage)),
            // @TODO: add salt
            // accounts[msg.sender].hashedSecret == bytes27(keccak256(abi.encodePacked("HOPRnet", msg.sender, bytes27(preImage)))),
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
            channel.partyABalance = channel.partyABalance.add(amount);
        } else {
            channel.partyABalance = channel.partyABalance.sub(amount);
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