// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "../HoprChannels/Channels.sol";

contract ChannelsMock is Channels {
    constructor(uint32 _secsClosure) {
        secsClosure = _secsClosure;
    }

    function fundChannel(
        address funder,
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        _fundChannel(funder, accountA, accountB, amountA, amountB);
    }

    function openChannel(
        address opener,
        address counterparty
    ) external {
        _openChannel(opener, counterparty);
    }

    function initiateChannelClosure(
        address initiator,
        address counterparty
    ) external {
        _initiateChannelClosure(initiator, counterparty);
    }

    function finalizeChannelClosure(
        IERC20 token,
        address initiator,
        address counterparty
    ) external {
        _finalizeChannelClosure(token, initiator, counterparty);
    }

    function getChannel(
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

    function getChannelId(
        address partyA,
        address partyB
    ) external pure returns (bytes32) {
        return _getChannelId(partyA, partyB);
    }

    function getChannelStatus(
        uint24 status
    ) external pure returns (ChannelStatus) {
        return _getChannelStatus(status);
    }

    function getChannelIteration(
        uint24 status
    ) external pure returns (uint256) {
        return _getChannelIteration(status);
    }

    function isPartyA(
        address accountA,
        address accountB
    ) external pure returns (bool) {
        return _isPartyA(accountA, accountB);
    }

    function getParties(
        address accountA,
        address accountB
    ) external pure returns (address, address) {
        return _getParties(accountA, accountB);
    }
}
