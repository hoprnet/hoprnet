// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

/**
 *    &&&&
 *    &&&&
 *    &&&&
 *    &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *    &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *     &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *     &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *     &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *     %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *    %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                          %%%%
 *                                          %%%%
 *                                          %%%%
 *
 * Bundles cryptographic primitives used by the HOPR protocol
 *
 */
abstract contract HoprCrypto {
    error InvalidFieldElement();
    error InvalidCurvePoint();
    error InvalidPointWitness();

    // secp256k1: y^2 = x^3 + b (mod F_p)
    uint256 internal constant SECP256K1_B = 0x0000000000000000000000000000000000000000000000000000000000000007;
    // Field order created by secp256k1 curve
    // solhint-disable-next-line max-line-length
    uint256 internal constant SECP256K1_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

    // Order of the underlying field used for secp256k1
    uint256 internal constant SECP256K1_BASE_FIELD_ORDER =
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;
    // x-component of base point of secp256k1 curve
    uint256 internal constant SECP256K1_BASE_POINT_X_COMPONENT =
        0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;

    // encoded sign of y-component of base point of secp256k1 curve
    uint8 internal constant SECP256K1_BASE_POINT_Y_COMPONENT_SIGN = 27;

    // E': y^2 = x^3 + A_PRIME + B_PRIME (mod F_p)
    // used by `hash_to_curve` function
    uint256 private constant A_PRIME = 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533;
    uint256 private constant B_PRIME = 1771;

    // Coefficients used for isogeneous mapping from E' to secp256k1
    // see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#appx-iso-secp256k1
    //
    // used by `hash_to_curve` function
    uint256 private constant K_10 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7;
    uint256 private constant K_11 = 0x07d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581;
    uint256 private constant K_12 = 0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262;
    uint256 private constant K_13 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c;
    uint256 private constant K_20 = 0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b;
    uint256 private constant K_21 = 0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14;
    uint256 private constant K_30 = 0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c;
    uint256 private constant K_31 = 0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3;
    uint256 private constant K_32 = 0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931;
    uint256 private constant K_33 = 0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84;
    uint256 private constant K_40 = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b;
    uint256 private constant K_41 = 0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573;
    uint256 private constant K_42 = 0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f;

    // Coefficients used for simplified SWU mapping
    // see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-suites-for-secp256k1
    //
    // used by `hash_to_curve` function
    uint256 private constant Z = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC24;
    uint256 private constant C_1 = 0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffffbfffff0b;
    uint256 private constant C_2 = 0x31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a59; // sqrt(-Z)

    uint256 private constant KECCAK256_BLOCKSIZE = 136;
    /**
     * Holds a compact ECDSA signature, following ERC-2098
     */

    struct CompactSignature {
        bytes32 r;
        bytes32 vs;
    }

    /**
     * Checks whether given value is an element of the secp256k1 field
     *
     * @param el element to check
     */
    function isFieldElementInternal(uint256 el) internal pure returns (bool) {
        return 0 == el || el < SECP256K1_FIELD_ORDER;
    }

    /**
     * Checks whether given coordinates of P fulfill the secp256k1 curve equation
     *
     * @param pX first component of P
     * @param pY second component of P
     */
    function isCurvePointInternal(uint256 pX, uint256 pY) internal pure returns (bool r) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            r :=
                eq(
                    mulmod(pY, pY, SECP256K1_BASE_FIELD_ORDER),
                    addmod(
                        SECP256K1_B,
                        mulmod(mulmod(pX, pX, SECP256K1_BASE_FIELD_ORDER), pX, SECP256K1_BASE_FIELD_ORDER),
                        SECP256K1_BASE_FIELD_ORDER
                    )
                )
        }
    }

    modifier isCurvePoint(uint256 pX, uint256 pY) {
        if (!isCurvePointInternal(pX, pY)) {
            revert InvalidCurvePoint();
        }
        _;
    }

    modifier isFieldElement(uint256 el) {
        if (!isFieldElementInternal(el)) {
            revert InvalidFieldElement();
        }
        _;
    }

    /**
     * Takes a `scalar` and returns the Ethereum address associated to
     * `scalar * G` where `G` is the base point of the secp256k1 curve.
     *
     * This function is necessary due to the missing ECMUL operation in Ethereum. It misuses the
     * ECRECOVER precompile to perform the scalar multiplication in a gas-efficient way
     *
     * For more information see
     * https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
     *
     * @param scalar to multiply with secp256k1 base point
     */
    function scalarTimesBasepoint(uint256 scalar) internal pure returns (address) {
        return ecrecover(
            0,
            SECP256K1_BASE_POINT_Y_COMPONENT_SIGN,
            bytes32(SECP256K1_BASE_POINT_X_COMPONENT),
            bytes32(mulmod(scalar, SECP256K1_BASE_POINT_X_COMPONENT, SECP256K1_FIELD_ORDER))
        );
    }

    /**
     * Takes a curve point `P = (pX, pY)` and a scalar and returns the Ethereum address associated
     * to the point `scalar * P` on the secp256k1 curve.
     *
     * This function is necessary due to the missing ECMUL operation in Ethereum. It misuses the
     * ECRECOVER precompile to perform the scalar multiplication in a gas-efficient way
     *
     * For more information see
     * https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
     *
     * @param scalar values to multiply P with
     * @param pX first component of P
     * @param pY second component of P
     */
    function scalarPointMultiplication(uint256 scalar, uint256 pX, uint256 pY) internal pure returns (address) {
        uint8 sign;
        if (pY % 2 == 0) {
            sign = 27;
        } else {
            sign = 28;
        }

        return ecrecover(0, sign, bytes32(pX), bytes32(mulmod(scalar, pX, SECP256K1_FIELD_ORDER)));
    }

    /**
     * Converts a curve point P to an Ethereum address.
     *
     * This function can be used to witness the result of a scalar
     * multiplication.
     *
     * @param pX first component of P
     * @param pY second component of P
     */
    function pointToAddress(uint256 pX, uint256 pY) internal pure returns (address) {
        return address(uint160(uint256(keccak256(abi.encodePacked(pX, pY)))));
    }

    /**
     * Adds two elliptic curve points P and Q using the general implementation.
     *
     * This function is optimized to perform one single point addition, e.g.
     * when using in a VRF or hash_to_curve scheme.
     *
     * @dev Throws if Q = -P since Infinity point is not supported.
     *
     * @dev This function is meant to be part of another function and thus does
     *      not perform any sanity checks, such as if any of the given points
     *      fulfill the curve equation. These checks are left to the caller of
     *      the function.
     *
     * Optimizations:
     * - solidity assembly
     * - optimize for a single point addition
     * - inline modular inversion
     *
     * @param pX first component of P
     * @param pY second component of P
     * @param qX first component of Q
     * @param qY second component of Q
     * @param a curve parameter, y^2 = x^3 + a*x + b (mod p)
     */
    function ecAdd(
        uint256 pX,
        uint256 pY,
        uint256 qX,
        uint256 qY,
        uint256 a
    )
        internal
        view
        returns (uint256 rx, uint256 ry)
    {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            if and(eq(pX, qX), not(eq(pY, qY))) {
                // Q = -P
                // which means P + Q = P - P = 0 which is not supported
                revert(0, 0)
            }
            let lambda
            let toInvert
            switch and(eq(pX, qX), eq(pY, qY))
            // P == Q ?
            case true {
                // Point double
                toInvert := addmod(mulmod(2, pY, SECP256K1_BASE_FIELD_ORDER), a, SECP256K1_BASE_FIELD_ORDER) // 2 * p.y

                // compute (2 * p.y) ^ -1 using expmod precompile
                let payload := mload(0x40)
                mstore(payload, 0x20) // Length of Base
                mstore(add(payload, 0x20), 0x20) // Length of Exponent
                mstore(add(payload, 0x40), 0x20) // Length of Modulus
                mstore(add(payload, 0x60), toInvert) // Base
                mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
                mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER) // Modulus
                if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
                    // 0x05 == expmod precompile
                    revert(0, 0)
                }
                lambda :=
                    mulmod( // (3 * p.x ^ 2) * (2 * p.y) ^ -1
                        mulmod( // 3 * p.x ^ 2
                        3, mulmod(pX, pX, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER),
                        mload(payload),
                        SECP256K1_BASE_FIELD_ORDER
                    )
            }
            case false {
                // Point addition
                toInvert :=
                    addmod( // q.x - p.x
                        qX, // q.x
                        sub(SECP256K1_BASE_FIELD_ORDER, pX), // - p.x
                        SECP256K1_BASE_FIELD_ORDER
                    )

                // compute (q.x - p.x) ^ -1 using expmod precompile
                let payload := mload(0x40)
                mstore(payload, 0x20) // Length of Base
                mstore(add(payload, 0x20), 0x20) // Length of Exponent
                mstore(add(payload, 0x40), 0x20) // Length of Modulus
                mstore(add(payload, 0x60), toInvert) // Base
                mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
                mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER) // Modulus
                if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
                    // 0x05 == expmod precompile
                    revert(0, 0)
                }

                lambda :=
                    mulmod( // (q.y - p.y) * (q.x - p.x) ^ -1
                        addmod( // q.y - p.y
                            qY, // q.y
                            sub(SECP256K1_BASE_FIELD_ORDER, pY), // - p.y
                            SECP256K1_BASE_FIELD_ORDER
                        ),
                        mload(payload), // (q.x - p.x) ^ -1
                        SECP256K1_BASE_FIELD_ORDER
                    )
            }

            rx :=
                addmod( // lambda^2 - q.x - p.x
                    mulmod(lambda, lambda, SECP256K1_BASE_FIELD_ORDER), // lambda^2
                    addmod( // - q.x - p.x
                        sub(SECP256K1_BASE_FIELD_ORDER, qX), // - q.x
                        sub(SECP256K1_BASE_FIELD_ORDER, pX), // - p.x
                        SECP256K1_BASE_FIELD_ORDER
                    ),
                    SECP256K1_BASE_FIELD_ORDER
                )

            ry :=
                addmod( // lambda * (p.x - r.x) - p.y
                    mulmod( // lambda * (p.x - r.x)
                        lambda,
                        addmod( // p.x - r.x
                            pX, // p.x
                            sub(SECP256K1_BASE_FIELD_ORDER, rx), // - r.x
                            SECP256K1_BASE_FIELD_ORDER
                        ),
                        SECP256K1_BASE_FIELD_ORDER
                    ),
                    sub(SECP256K1_BASE_FIELD_ORDER, pY),
                    SECP256K1_BASE_FIELD_ORDER
                )
        }
    }

    /**
     * Consumes a byte string and returns a pseudo-random secp256k1 curvepoint.
     *
     * Implements secp256k1_XMD:KECCAK_256_SSWU_RO_, see
     * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html
     *
     * @dev DSTs longer than 255 bytes are considered unsound.
     *      see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-domain-separation
     *
     * @param payload values "to hash"
     * @param dst domain separation tag, used to makes protocol instantiations unique
     */
    function hashToCurve(bytes memory payload, bytes memory dst) internal view returns (uint256 rx, uint256 ry) {
        (uint256 u0, uint256 u1) = hash_to_field(payload, dst);

        (uint256 q0x, uint256 q0y) = mapToCurveSimpleSWU(uint256(u0)); // on isogenous curve
        (uint256 q1x, uint256 q1y) = mapToCurveSimpleSWU(uint256(u1)); // on isogenous curve

        // P + Q on isogenous curve
        (uint256 sx, uint256 sy) = ecAdd(q0x, q0y, q1x, q1y, A_PRIME);

        return mapPoint(sx, sy);
    }

    /**
     * Maps a curve point on E': y^2 = A'x^3 + B' to secp256k1. This function is necessary because
     * A*B = 0 for secp256k1 which is why the simplified SWU mapping is not directly applicable.
     *
     * A' := 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533
     * B' := 1771
     * modulus 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F (same as secp256k1)
     *
     * see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#appx-iso-secp256k1
     *
     * Optimizations:
     * - mathematical optimization: reduce expmod / mulmod / addmod operations
     * - few temporary values to reduce memory expansion
     * - use Solidity assembly
     *
     * @param pX first component of P
     * @param pY second component of P
     */
    function mapPoint(uint256 pX, uint256 pY) internal view returns (uint256 rx, uint256 ry) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            let pxSquare := mulmod(pX, pX, SECP256K1_BASE_FIELD_ORDER) // p.x * p.x
            let pxCubic := mulmod(pX, pxSquare, SECP256K1_BASE_FIELD_ORDER) // p.x * pxSquare

            // xNum = k_(1,3) * x'^3 + k_(1,2) * x'^2 + k_(1,1) * x' + k_(1,0)
            let xNum :=
                addmod(
                    addmod(
                        mulmod(K_13, pxCubic, SECP256K1_BASE_FIELD_ORDER),
                        mulmod(K_12, pxSquare, SECP256K1_BASE_FIELD_ORDER),
                        SECP256K1_BASE_FIELD_ORDER
                    ),
                    addmod(mulmod(K_11, pX, SECP256K1_BASE_FIELD_ORDER), K_10, SECP256K1_BASE_FIELD_ORDER),
                    SECP256K1_BASE_FIELD_ORDER
                )

            // xDen = x'^2 + k_(2,1) * x' + k_(2,0)
            let xDen :=
                addmod(
                    addmod(pxSquare, mulmod(K_21, pX, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER),
                    K_20,
                    SECP256K1_BASE_FIELD_ORDER
                )

            // computes xDen ^ -1 using expmod precompile
            let payload := mload(0x40)
            mstore(payload, 0x20) // Length of Base
            mstore(add(payload, 0x20), 0x20) // Length of Exponent
            mstore(add(payload, 0x40), 0x20) // Length of Modulus
            mstore(add(payload, 0x60), xDen) // Base
            mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
            mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER) // Modulus
            if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
                // 0x05 == expmod precompile
                revert(0, 0)
            }

            xDen := mload(payload)

            // x = xNum / xDen
            rx := mulmod(xNum, xDen, SECP256K1_BASE_FIELD_ORDER)

            // y_num = k_(3,3) * x'^3 + k_(3,2) * x'^2 + k_(3,1) * x' + k_(3,0)
            let y_num :=
                addmod(
                    addmod(
                        mulmod(K_33, pxCubic, SECP256K1_BASE_FIELD_ORDER),
                        mulmod(K_32, pxSquare, SECP256K1_BASE_FIELD_ORDER),
                        SECP256K1_BASE_FIELD_ORDER
                    ),
                    addmod(mulmod(K_31, pX, SECP256K1_BASE_FIELD_ORDER), K_30, SECP256K1_BASE_FIELD_ORDER),
                    SECP256K1_BASE_FIELD_ORDER
                )

            // y_den = x'^3 + k_(4,2) * x'^2 + k_(4,1) * x' + k_(4,0)
            let y_den :=
                addmod(
                    addmod(pxCubic, mulmod(K_42, pxSquare, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER),
                    addmod(mulmod(K_41, pX, SECP256K1_BASE_FIELD_ORDER), K_40, SECP256K1_BASE_FIELD_ORDER),
                    SECP256K1_BASE_FIELD_ORDER
                )

            // Computes (y_den ^ -1) using expmod precompile
            payload := mload(0x40)
            mstore(payload, 0x20) // Length of Base
            mstore(add(payload, 0x20), 0x20) // Length of Exponent
            mstore(add(payload, 0x40), 0x20) // Length of Modulus
            mstore(add(payload, 0x60), y_den) // Base
            mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
            mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER) // p
            if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
                // 0x05 == expmod precompile
                revert(0, 0)
            }

            y_den := mload(payload)

            // y = y' * y_num / y_den
            ry := mulmod(mulmod(pY, y_num, SECP256K1_BASE_FIELD_ORDER), y_den, SECP256K1_BASE_FIELD_ORDER)
        }
    }

    /**
     * Takes a field element and returns a curve point on an elliptic curve that is 3-isogenous
     * to secp256k1.
     *
     * Implements the simplified SWU mapping. Uses the optimized sample implementation from
     * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-simplified-swu-method
     *
     * Optimizations:
     * - mathematical optimization: reduce expmod / mulmod / addmod operations
     * - few temporary values to reduce memory expansion
     * - Solidity assembly
     *
     * @param u the field element to map to a secp256k1 curve point
     */
    function mapToCurveSimpleSWU(uint256 u) internal view returns (uint256 rx, uint256 ry) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            let tv1 := mulmod(u, u, SECP256K1_BASE_FIELD_ORDER) // 1.  tv1 = u^2
            tv1 := mulmod(Z, tv1, SECP256K1_BASE_FIELD_ORDER) // 2.  tv1 = Z * tv1
            let tv2 := mulmod(tv1, tv1, SECP256K1_BASE_FIELD_ORDER) // 3.  tv2 = tv1^2
            tv2 := addmod(tv2, tv1, SECP256K1_BASE_FIELD_ORDER) // 4.  tv2 = tv2 + tv1
            let tv3 := addmod(tv2, 1, SECP256K1_BASE_FIELD_ORDER) // 5.  tv3 = tv2 + 1
            tv3 := mulmod(tv3, B_PRIME, SECP256K1_BASE_FIELD_ORDER) // 6.  tv3 = B * tv3

            let tv4
            switch eq(tv2, 0)
            // 7.  tv4 = CMOV(Z, -tv2, tv2 != 0)
            case true { tv4 := Z }
            case false { tv4 := sub(SECP256K1_BASE_FIELD_ORDER, tv2) }

            tv4 := mulmod(A_PRIME, tv4, SECP256K1_BASE_FIELD_ORDER) // 8.  tv4 = A * tv4
            tv2 := mulmod(tv3, tv3, SECP256K1_BASE_FIELD_ORDER) // 9.  tv2 = tv3^2
            let tv6 := mulmod(tv4, tv4, SECP256K1_BASE_FIELD_ORDER) // 10. tv6 = tv4^2
            let tv5 := mulmod(A_PRIME, tv6, SECP256K1_BASE_FIELD_ORDER) // 11. tv5 = A * tv6
            tv2 := addmod(tv2, tv5, SECP256K1_BASE_FIELD_ORDER) // 12. tv2 = tv2 + tv5
            tv2 := mulmod(tv2, tv3, SECP256K1_BASE_FIELD_ORDER) // 13. tv2 = tv2 * tv3
            tv6 := mulmod(tv6, tv4, SECP256K1_BASE_FIELD_ORDER) // 14. tv6 = tv6 * tv4
            tv5 := mulmod(B_PRIME, tv6, SECP256K1_BASE_FIELD_ORDER) // 15. tv5 = B * tv6
            tv2 := addmod(tv2, tv5, SECP256K1_BASE_FIELD_ORDER) // 16. tv2 = tv2 + tv5
            rx := mulmod(tv1, tv3, SECP256K1_BASE_FIELD_ORDER) // 17.   x = tv1 * tv3

            // 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)
            let y1
            let isSquare

            // sqrt_ratio_3mod4(u,v) subroutine
            // tv1 -> tv7
            // tv2 -> tv8
            // u -> tv2
            // v -> tv6
            //
            // Algorithm from:
            // https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-optimized-sqrt_ratio-for-q-

            // ===================================
            {
                let tv7 := mulmod(tv6, tv6, SECP256K1_BASE_FIELD_ORDER) // 1. tv1 = v^2
                let tv8 := mulmod(tv2, tv6, SECP256K1_BASE_FIELD_ORDER) // 2. tv2 = u * v

                tv7 := mulmod(tv7, tv8, SECP256K1_BASE_FIELD_ORDER) // 3. tv1 = tv1 * tv2

                // 4. y1 = tv1^c1 (using expmod precompile)
                let p := mload(0x40)
                mstore(p, 0x20) // Length of Base
                mstore(add(p, 0x20), 0x20) // Length of Exponent
                mstore(add(p, 0x40), 0x20) // Length of Modulus
                mstore(add(p, 0x60), tv7) // Base
                mstore(add(p, 0x80), C_1) // Exponent
                mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER) // Modulus
                if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
                    // 0x05 == expmod precompile
                    revert(0, 0)
                }

                let y1Inner := mulmod(mload(p), tv8, SECP256K1_BASE_FIELD_ORDER) // 5. y1 = y1 * tv2
                let y2Inner := mulmod(y1Inner, C_2, SECP256K1_BASE_FIELD_ORDER) // 6. y2 = y1 * c2
                let tv9 := mulmod(y1Inner, y1Inner, SECP256K1_BASE_FIELD_ORDER) // 7. tv3 = y1^2
                tv9 := mulmod(tv9, tv6, SECP256K1_BASE_FIELD_ORDER) // 8. tv3 = tv3 * v

                switch eq(tv9, tv2)
                // 9. isQR = tv3 == u
                case true {
                    // 10. y = CMOV(y2, y1, isQR)
                    isSquare := true
                    y1 := y1Inner
                }
                case false {
                    isSquare := false
                    y1 := y2Inner
                }
            }

            // =====================================

            ry := mulmod(tv1, u, SECP256K1_BASE_FIELD_ORDER) // 19.   y = tv1 * u
            ry := mulmod(ry, y1, SECP256K1_BASE_FIELD_ORDER) // 20.   y = y * y1

            if isSquare {
                rx := tv3 // 21.   x = CMOV(x, tv3, is_gx1_square)
                ry := y1 // 22.   y = CMOV(y, y1, is_gx1_square)
            }

            // 23.  e1 = sgn0(u) == sgn0(y)
            if iszero(eq(mod(u, 2), mod(ry, 2))) {
                // sgn0(x) ~= x % 2
                ry := sub(SECP256K1_BASE_FIELD_ORDER, ry) // 24.   y = CMOV(-y, y, e1)
            }

            // compute tv4 ^ -1
            let payload := mload(0x40)
            mstore(payload, 0x20) // Length of Base
            mstore(add(payload, 0x20), 0x20) // Length of Exponent
            mstore(add(payload, 0x40), 0x20) // Length of Modulus
            mstore(add(payload, 0x60), tv4) // Base
            mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p-2
            mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER) // p
            if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
                // 0x05 == expmod precompile
                revert(0, 0)
            }

            rx := mulmod(rx, mload(payload), SECP256K1_BASE_FIELD_ORDER) // 25.   x = x / tv4
        }
    }

    /**
     * Takes an arbitrary byte-string and a domain seperation tag (dst) and returns
     * two elements of the field used to create the secp256k1 curve.
     *
     * @dev DSTs longer than 255 bytes are considered unsound.
     *      see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-domain-separation
     *
     * @param message the message to hash
     * @param dst domain separation tag, used to make protocol instantiations unique
     */
    function hash_to_field(bytes memory message, bytes memory dst) internal view returns (uint256 u0, uint256 u1) {
        (bytes32 b1, bytes32 b2, bytes32 b3) = expand_message_xmd_keccak256(message, dst);

        // computes [...b1[..], ...b2[0..16]] ^ 1 mod n
        // solhint-disable-next-line no-inline-assembly
        assembly {
            let p := mload(0x40) // next free memory slot
            mstore(p, 0x30) // Length of Base
            mstore(add(p, 0x20), 0x20) // Length of Exponent
            mstore(add(p, 0x40), 0x20) // Length of Modulus
            mstore(add(p, 0x60), b1) // Base
            mstore(add(p, 0x80), b2)
            mstore(add(p, 0x90), 1) // Exponent
            mstore(add(p, 0xb0), SECP256K1_BASE_FIELD_ORDER) // Modulus
            if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) { revert(0, 0) }

            u0 := mload(p)
        }

        // computes [...b2[16..32], ...b3[..]] ^ 1 mod n
        // solhint-disable-next-line no-inline-assembly
        assembly {
            let p := mload(0x40)
            mstore(p, 0x30) // Length of Base
            mstore(add(p, 0x20), 0x20) // Length of Exponent
            mstore(add(p, 0x50), b2)
            mstore(add(p, 0x40), 0x20) // Length of Modulus
            mstore(add(p, 0x70), b3) // Base
            mstore(add(p, 0x90), 1) // Exponent
            mstore(add(p, 0xb0), SECP256K1_BASE_FIELD_ORDER) // Modulus
            if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) { revert(0, 0) }

            u1 := mload(p)
        }
    }

    /**
     * Takes an arbitrary bytestring and a domain seperation tag and returns a
     * pseudo-random scalar in the secp256k1 curve field.
     *
     * @dev DSTs longer than 255 bytes are considered unsound.
     *      see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-domain-separation
     *
     * @param message the message to hash
     * @param dst domain separation tag, used to make protocol instantiations unique
     */
    function hashToScalar(bytes memory message, bytes memory dst) internal view returns (uint256 u) {
        (bytes32 b1, bytes32 b2) = expand_message_xmd_keccak256_single(message, dst);

        // computes [...b1[0..32], ...b2[0..16]] ^ 1 mod n
        // solhint-disable-next-line no-inline-assembly
        assembly {
            let p := mload(0x40) // next free memory slot
            mstore(p, 0x30) // Length of Base
            mstore(add(p, 0x20), 0x20) // Length of Exponent
            mstore(add(p, 0x40), 0x20) // Length of Modulus
            mstore(add(p, 0x60), b1) // Base
            mstore(add(p, 0x80), b2)
            mstore(add(p, 0x90), 1) // Exponent
            mstore(add(p, 0xb0), SECP256K1_FIELD_ORDER) // Modulus
            if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) { revert(0, 0) }

            u := mload(p)
        }
    }

    /**
     * Expands an arbitrary byte-string to 96 bytes using the `expand_message_xmd` method described in
     * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html
     *
     * Used for hash_to_curve functionality.
     *
     * @dev This is not a general implementation as the output length fixed. It is tailor-made
     *      for secp256k1_XMD:KECCAK_256_SSWU_RO_ hash_to_curve implementation.
     *
     * @dev DSTs longer than 255 bytes are considered unsound.
     *      see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-domain-separation
     *
     * @param message the message to hash
     * @param dst domain separation tag, used to make protocol instantiations unique
     */
    function expand_message_xmd_keccak256(
        bytes memory message,
        bytes memory dst
    )
        internal
        pure
        returns (bytes32 b1, bytes32 b2, bytes32 b3)
    {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            if gt(mload(dst), 255) { revert(0, 0) }

            let b0
            {
                // create payload for b0 hash
                let b0Payload := mload(0x40)

                // payload[0..KECCAK256_BLOCKSIZE] = 0

                let b0PayloadO := KECCAK256_BLOCKSIZE // leave first block empty
                let msg_o := 0x20 // skip length prefix

                // payload[KECCAK256_BLOCKSIZE..KECCAK256_BLOCKSIZE+message.len()] = message[0..message.len()]
                for { let i := 0 } lt(i, mload(message)) { i := add(i, 0x20) } {
                    mstore(add(b0Payload, b0PayloadO), mload(add(message, msg_o)))
                    b0PayloadO := add(b0PayloadO, 0x20)
                    msg_o := add(msg_o, 0x20)
                }

                // payload[KECCAK256_BLOCKSIZE+message.len()+1..KECCAK256_BLOCKSIZE+message.len()+2] = 96
                b0PayloadO := add(mload(message), 137)
                mstore8(add(b0Payload, b0PayloadO), 0x60) // only support for 96 bytes output length

                let dstO := 0x20
                b0PayloadO := add(b0PayloadO, 2)

                // payload[KECCAK256_BLOCKSIZE+message.len()+3..KECCAK256_BLOCKSIZE+message.len()+dst.len()]
                // = dst[0..dst.len()]
                for { let i := 0 } lt(i, mload(dst)) { i := add(i, 0x20) } {
                    mstore(add(b0Payload, b0PayloadO), mload(add(dst, dstO)))
                    b0PayloadO := add(b0PayloadO, 0x20)
                    dstO := add(dstO, 0x20)
                }

                // payload[KECCAK256_BLOCKSIZE+message.len()+dst.len()..KECCAK256_BLOCKSIZE+message.len()+dst.len()+1]
                // = dst.len()
                b0PayloadO := add(add(mload(message), mload(dst)), 139)
                mstore8(add(b0Payload, b0PayloadO), mload(dst))

                b0 := keccak256(b0Payload, add(140, add(mload(dst), mload(message))))
            }

            // create payload for b1, b2 ... hashes
            let bIPayload := mload(0x40)
            mstore(bIPayload, b0)
            // payload[32..33] = 1
            mstore8(add(bIPayload, 0x20), 1)

            let payloadO := 0x21
            let dstO := 0x20

            // payload[33..33+dst.len()] = dst[0..dst.len()]
            for { let i := 0 } lt(i, mload(dst)) { i := add(i, 0x20) } {
                mstore(add(bIPayload, payloadO), mload(add(dst, dstO)))
                payloadO := add(payloadO, 0x20)
                dstO := add(dstO, 0x20)
            }

            // payload[65+dst.len()..66+dst.len()] = dst.len()
            mstore8(add(bIPayload, add(0x21, mload(dst))), mload(dst))

            b1 := keccak256(bIPayload, add(34, mload(dst)))

            // payload[0..32] = b0 XOR b1
            mstore(bIPayload, xor(b0, b1))
            // payload[32..33] = 2
            mstore8(add(bIPayload, 0x20), 2)

            b2 := keccak256(bIPayload, add(34, mload(dst)))

            // payload[0..32] = b0 XOR b2
            mstore(bIPayload, xor(b0, b2))
            // payload[32..33] = 2
            mstore8(add(bIPayload, 0x20), 3)

            b3 := keccak256(bIPayload, add(34, mload(dst)))
        }
    }

    /**
     * Expands an arbitrary byte-string to 48 bytes using the `expand_message_xmd` method described in
     * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html
     *
     * Used for the VRF functionality.
     *
     * @dev This is not a general implementation as the output length fixed. It is tailor-made
     *      for secp256k1_XMD:KECCAK_256_SSWU_RO_ hash_to_curve implementation.
     *
     * @dev DSTs longer than 255 bytes are considered unsound.
     *      see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-domain-separation
     *
     * @param message the message to hash
     * @param dst domain separation tag, used to make protocol instantiations unique
     */
    function expand_message_xmd_keccak256_single(
        bytes memory message,
        bytes memory dst
    )
        internal
        pure
        returns (bytes32 b1, bytes32 b2)
    {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            if gt(mload(dst), 255) { revert(0, 0) }

            let b0
            {
                // create payload for b0 hash
                let b0Payload := mload(0x40)

                // payload[0..KECCAK256_BLOCKSIZE] = 0

                let b0PayloadO := KECCAK256_BLOCKSIZE // leave first block empty
                let msg_o := 0x20 // skip length prefix

                // payload[KECCAK256_BLOCKSIZE..KECCAK256_BLOCKSIZE+message.len()] = message[0..message.len()]
                for { let i := 0 } lt(i, mload(message)) { i := add(i, 0x20) } {
                    mstore(add(b0Payload, b0PayloadO), mload(add(message, msg_o)))
                    b0PayloadO := add(b0PayloadO, 0x20)
                    msg_o := add(msg_o, 0x20)
                }

                // payload[KECCAK256_BLOCKSIZE+message.len()+1..KECCAK256_BLOCKSIZE+message.len()+2] = 48
                b0PayloadO := add(mload(message), 137)
                mstore8(add(b0Payload, b0PayloadO), 0x30) // only support for 48 bytes output length

                let dstO := 0x20
                b0PayloadO := add(b0PayloadO, 2)

                // payload[KECCAK256_BLOCKSIZE+message.len()+3..KECCAK256_BLOCKSIZE+message.len()+dst.len()]
                // = dst[0..dst.len()]
                for { let i := 0 } lt(i, mload(dst)) { i := add(i, 0x20) } {
                    mstore(add(b0Payload, b0PayloadO), mload(add(dst, dstO)))
                    b0PayloadO := add(b0PayloadO, 0x20)
                    dstO := add(dstO, 0x20)
                }

                // payload[KECCAK256_BLOCKSIZE+message.len()+dst.len()..KECCAK256_BLOCKSIZE+message.len()+dst.len()+1]
                // = dst.len()
                b0PayloadO := add(add(mload(message), mload(dst)), 139)
                mstore8(add(b0Payload, b0PayloadO), mload(dst))

                b0 := keccak256(b0Payload, add(140, add(mload(dst), mload(message))))
            }

            // create payload for b1, b2 ... hashes
            let bIPayload := mload(0x40)
            mstore(bIPayload, b0)
            // payload[32..33] = 1
            mstore8(add(bIPayload, 0x20), 1)

            let payloadO := 0x21
            let dstO := 0x20

            // payload[33..33+dst.len()] = dst[0..dst.len()]
            for { let i := 0 } lt(i, mload(dst)) { i := add(i, 0x20) } {
                mstore(add(bIPayload, payloadO), mload(add(dst, dstO)))
                payloadO := add(payloadO, 0x20)
                dstO := add(dstO, 0x20)
            }

            // payload[65+dst.len()..66+dst.len()] = dst.len()
            mstore8(add(bIPayload, add(0x21, mload(dst))), mload(dst))

            b1 := keccak256(bIPayload, add(34, mload(dst)))

            // payload[0..32] = b0 XOR b1
            mstore(bIPayload, xor(b0, b1))
            // payload[32..33] = 2
            mstore8(add(bIPayload, 0x20), 2)

            b2 := keccak256(bIPayload, add(34, mload(dst)))
        }
    }

    /**
     * Bundles values to verify the validity of the VRF
     */
    struct VRFParameters {
        // the main deterministic pseudo-random values
        uint256 vx;
        uint256 vy;
        // s = r + h * a, where r, a are kept hidden
        uint256 s;
        // hash over computed values
        uint256 h;
        // Ethereum only supports scalar multiplication to address
        // so we provide witnesses that are checked against
        // computed values
        uint256 sBx; // s * B
        uint256 sBy;
        uint256 hVx; // h * V
        uint256 hVy;
    }

    /**
     * Bundles payload used to create a VRF-generated deterministic
     * pseudo-random value.
     */
    struct VRFPayload {
        // the main message, e.g. ticket Hash
        bytes32 message;
        // the "public key" of the signer,
        // necessary to make VRF individual for each Ethereum account
        address signer;
        // domain separation tag, make each protocol instantiation,
        // unique, such as staging and production environment,
        // must be at most 255 bytes, otherwise considered unsound
        // see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-domain-separation
        bytes dst;
    }

    /**
     * Implements a VRF based on public-key cryptography using hash_to_curve primitive.
     *
     * Algorithm highly inspired by:
     * https://www.signal.org/docs/specifications/xeddsa/#vxeddsa
     *
     * @param params necessary values verify validity of VRF
     * @param payload values over which the VRF was computed, e.g. ticketHash
     */
    function vrfVerify(VRFParameters memory params, VRFPayload memory payload) internal view returns (bool) {
        if (params.h >= SECP256K1_BASE_FIELD_ORDER || params.s >= SECP256K1_BASE_FIELD_ORDER) {
            revert InvalidFieldElement();
        }

        if (!isCurvePointInternal(params.vx, params.vy)) {
            revert InvalidCurvePoint();
        }

        // we get a pseudo-random curve point
        (uint256 bX, uint256 bY) = hashToCurve(abi.encodePacked(payload.signer, payload.message), payload.dst);

        // Mitigate missing ECMUL operation by using precomputed values and verify
        // against computed Ethereum address.
        address sBvMaybe = scalarPointMultiplication(params.s, bX, bY);

        if (sBvMaybe != pointToAddress(params.sBx, params.sBy)) {
            revert InvalidPointWitness();
        }

        address hVMaybe = scalarPointMultiplication(params.h, params.vx, params.vy);

        if (hVMaybe != pointToAddress(params.hVx, params.hVy)) {
            revert InvalidPointWitness();
        }

        // We checked the validity of precomputed sB and hV values,
        // now use them as if they were intermediate results.

        // R = sB - hV
        // solhint-disable-next-line max-line-length
        (uint256 rx, uint256 ry) = ecAdd(params.sBx, params.sBy, params.hVx, SECP256K1_BASE_FIELD_ORDER - params.hVy, 0);

        uint256 hCheck =
            hashToScalar(abi.encodePacked(payload.signer, params.vx, params.vy, rx, ry, payload.message), payload.dst);

        return hCheck == params.h;
    }
}
