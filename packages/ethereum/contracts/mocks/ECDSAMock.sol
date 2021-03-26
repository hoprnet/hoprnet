// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.7.5;

import "../utils/ECDSA.sol";

contract ECDSAMock {
    function uncompressedPubKeyToAddress(
        bytes calldata uncompressedPubKey
    ) external pure returns (address) {
        return ECDSA.uncompressedPubKeyToAddress(uncompressedPubKey);
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
