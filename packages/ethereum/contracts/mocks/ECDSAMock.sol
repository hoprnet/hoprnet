// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.6.0;

import "../utils/ECDSA.sol";

contract ECDSAMock {
    function pubKeyToEthereumAddress(uint256 x, uint256 y) external pure returns (address) {
        return ECDSA.pubKeyToEthereumAddress(x, y);
    }

    function validate(uint256 x, uint256 y) external pure returns (bool) {
        return ECDSA.validate(x, y);
    }

    function recover(
        bytes32 hash,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) external pure returns (address) {
        return ECDSA.recover(hash, r, s, v);
    }

    function toEthSignedMessageHash(
        string calldata length,
        bytes calldata message
    ) external pure returns (bytes32) {
        return ECDSA.toEthSignedMessageHash(length, message);
    }
}
