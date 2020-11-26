// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity ^0.6.0;

/**
 * @dev Elliptic Curve Digital Signature Algorithm (ECDSA) operations.
 *
 * These functions can be used to verify that a message was signed by the holder
 * of the private keys of a given address.
 */
library ECDSA {
    // y^2 = x^3 + 7 mod p, where p is FIELD_ORDER
    uint256 constant FIELD_ORDER = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f;
    uint256 constant CURVE_ORDER = 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141;
    uint256 constant HALF_CURVE_ORDER = (CURVE_ORDER - 1) / 2;

    /**
     * @dev Computes the Ethereum address from a public key given as an
     * uncompressed EC-point.
     */
    function pubKeyToEthereumAddress(uint256 x, uint256 y) internal pure returns (address) {
        require(validate(x, y), "Point must be on the curve.");
        return address(bytes20(bytes32(keccak256(abi.encodePacked(x, y)) << 96)));
    }

    /**
     * @dev Computes the Ethereum address from a public key given as a
     * compressed EC-point.
     */
    function compressedPubKeyToEthereumAddress(uint256 compressedX, uint8 odd) internal returns (address) {
        (uint256 x, uint256 y) = decompress(compressedX, odd);
        return pubKeyToEthereumAddress(x, y);
    }

    function compress(uint256 x, uint256 y) internal pure returns (uint256, uint8) {
        return (x, uint8(y % 2));
    }

    /**
     * @dev Decompresses a compressed elliptic curve point and
     * returns the uncompressed version.
     * @notice secp256k1: y^2 = x^3 + 7 (mod p)
     * "Converts from (x, 1 / 0) to (x,y)"
     */
    function decompress(uint256 x, uint8 odd) internal returns (uint256, uint256) {
        uint256 sqrY = addmod(7, mulmod(mulmod(x, x, FIELD_ORDER), x, FIELD_ORDER), FIELD_ORDER);

        uint256 sqrtExponent = (FIELD_ORDER + 1) / 4;

        uint256 y;

        /* solhint-disable no-inline-assembly */
        assembly {
            // free memory pointer
            let memPtr := mload(0x40)

            // length of base, exponent, modulus
            mstore(memPtr, 0x20)
            mstore(add(memPtr, 0x20), 0x20)
            mstore(add(memPtr, 0x40), 0x20)

            // assign base, exponent, modulus
            mstore(add(memPtr, 0x60), sqrY)
            mstore(add(memPtr, 0x80), sqrtExponent)
            mstore(add(memPtr, 0xa0), FIELD_ORDER)

            // call the precompiled contract BigModExp (0x05)
            let success := call(gas(), 0x05, 0x0, memPtr, 0xc0, memPtr, 0x20)

            switch success
                case 0 {
                    revert(0x0, 0x0)
                }
                default {
                    y := mload(memPtr)
                }
        }
        /* solhint-enable no-inline-assembly */

        bool isOdd = y % 2 == 1;

        if ((isOdd && odd == 0) || (!isOdd && odd == 1)) {
            y = FIELD_ORDER - y;
        }

        return (x, y);
    }

    function validate(uint256 x, uint256 y) public pure returns (bool) {
        uint256 rightHandSide = addmod(7, mulmod(mulmod(x, x, FIELD_ORDER), x, FIELD_ORDER), FIELD_ORDER);

        uint256 leftHandSide = mulmod(y, y, FIELD_ORDER);

        return leftHandSide == rightHandSide;
    }

    /**
     * @dev Returns the address that signed a hashed message (`hash`) with
     * `signature`. This address can then be used for verification purposes.
     *
     * The `ecrecover` EVM opcode allows for malleable (non-unique) signatures:
     * this function rejects them by requiring the `s` value to be in the lower
     * half order, and the `v` value to be either 27 or 28.
     *
     * IMPORTANT: `hash` _must_ be the result of a hash operation for the
     * verification to be secure: it is possible to craft signatures that
     * recover to arbitrary addresses for non-hashed data. A safe way to ensure
     * this is by receiving a hash of the original message (which may otherwise
     * be too long), and then calling {toEthSignedMessageHash} on it.
     */
    function recover(
        bytes32 hash,
        bytes32 r,
        bytes32 s,
        uint8 v
    ) internal pure returns (address) {
        // EIP-2 still allows signature malleability for ecrecover(). Remove this possibility and make the signature
        // unique. Appendix F in the Ethereum Yellow paper (https://ethereum.github.io/yellowpaper/paper.pdf), defines
        // the valid range for s in (281): 0 < s < secp256k1n ÷ 2 + 1, and for v in (282): v ∈ {27, 28}. Most
        // signatures from current libraries generate a unique signature with an s-value in the lower half order.
        //
        // If your library generates malleable signatures, such as s-values in the upper range, calculate a new s-value
        // with 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141 - s1 and flip v from 27 to 28 or
        // vice versa. If your library also generates signatures with 0/1 for v instead 27/28, add 27 to v to accept
        // these malleable signatures as well.
        if (uint256(s) > HALF_CURVE_ORDER) {
            revert("ECDSA: invalid signature 's' value");
        }

        if (v != 27 && v != 28) {
            revert("ECDSA: invalid signature 'v' value");
        }

        // If the signature is valid (and not malleable), return the signer address
        address signer = ecrecover(hash, v, r, s);
        require(signer != address(0), "ECDSA: invalid signature");

        return signer;
    }

    /**
     * @dev Returns an Ethereum and HOPRnet Signed Message.
     * Replicates the behavior of the https://github.com/ethereum/wiki/wiki/JSON-RPC#eth_sign[`eth_sign`]
     * JSON-RPC method but also includes "HOPRnet" in the message.
     */
    function toEthSignedMessageHash(string memory length, bytes memory message) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n", length, "HOPRnet", message));
    }
}
