// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "../HoprChannels.sol";

contract ChannelsMock is HoprChannels {
    constructor(address _token, uint32 _secsClosure)
    HoprChannels(_token, _secsClosure) {}

    function fundChannelInternal(
        address funder,
        address accountA,
        address accountB,
        uint256 amountA,
        uint256 amountB
    ) external {
        _fundChannel(funder, accountA, accountB, amountA, amountB);
    }

    function openChannelInternal(
        address opener,
        address counterparty
    ) external {
        _openChannel(opener, counterparty);
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

    function getChannelStatusInternal(
        uint24 status
    ) external pure returns (ChannelStatus) {
        return _getChannelStatus(status);
    }

    function getChannelIterationInternal(
        uint24 status
    ) external pure returns (uint256) {
        return _getChannelIteration(status);
    }

    function isPartyAInternal(
        address accountA,
        address accountB
    ) external pure returns (bool) {
        return _isPartyA(accountA, accountB);
    }

    function getPartiesInternal(
        address accountA,
        address accountB
    ) external pure returns (address, address) {
        return _getParties(accountA, accountB);
    }
}
