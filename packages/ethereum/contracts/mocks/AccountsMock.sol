// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "../HoprChannels.sol";

contract AccountsMock is HoprChannels {
    constructor(address _token, uint32 _secsClosure)
    HoprChannels(_token, _secsClosure) {}

    function initializeAccountInternal(
        address sender,
        uint256 pubKeyFirstHalf,
        uint256 pubKeySecondHalf,
        bytes32 secret
    ) external {
        _initializeAccount(
            sender,
            pubKeyFirstHalf,
            pubKeySecondHalf,
            secret
        );
    }

    function updateAccountSecretInternal(
        address sender,
        bytes32 secret
    ) external {
        _updateAccountSecret(sender, secret);
    }
}
