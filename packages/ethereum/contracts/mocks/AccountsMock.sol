// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "../HoprChannels/Accounts.sol";

contract AccountsMock is Accounts {
    function initializeAccount(
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

    function updateAccount(
        address sender,
        bytes32 secret
    ) external {
        _updateAccount(sender, secret);
    }
}
