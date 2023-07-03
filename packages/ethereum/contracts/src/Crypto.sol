// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

error InvalidFieldElement();
error InvalidCurvePoint();
error GeneralError();
error MessageTooLong();

abstract contract HoprCrypto {
  // secp256k1: y^2 = x^3 + ax + b (mod F_p)

  uint256 constant SECP256K1_B = 0x0000000000000000000000000000000000000000000000000000000000000007;
  // Field order created by secp256k1 curve
  uint256 constant SECP256K1_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

  // 
  uint256 constant SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;
  // x-component of base point of secp256k1 curve
  uint256 constant SECP256K1_BASE_POINT_X_COMPONENT = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;

  // encoded sign of y-component of base point of secp256k1 curve
  uint8 constant SECP256K1_BASE_POINT_Y_COMPONENT_SIGN = 27;

  /**
   * Holds a compact ECDSA signature, following ERC-2098
   */
  struct CompactSignature {
    bytes32 r;
    bytes32 vs;
  }

  struct CurvePoint {
    uint256 x;
    uint256 y;
  }

  /**
   * Checks whether given value is an element of the secp256k1 field
   */
  modifier isFieldElement(uint256 el) {
    if (0 == el || el >= SECP256K1_FIELD_ORDER) {
      revert InvalidFieldElement();
    }
    _;
  }

  /**
   * Checks whether given coordinates fulfill the secp256k1 curve equation
   */
  modifier isCurvePoint(CurvePoint memory p)  {
    if ( 
      p.x == 0 || p.x >= SECP256K1_BASE_FIELD_ORDER ||
      p.y == 0 || p.y >= SECP256K1_BASE_FIELD_ORDER ||
      mulmod(p.y, p.y, SECP256K1_BASE_FIELD_ORDER) != 
      (mulmod(mulmod(p.x, p.x, SECP256K1_BASE_FIELD_ORDER), p.x, SECP256K1_BASE_FIELD_ORDER) + 7) % SECP256K1_BASE_FIELD_ORDER
    ) {
      revert InvalidCurvePoint();
    }
    _;
  }

  /**
   * Ticket redemption uses an asymmetric challenge-response mechanism whose verification
   * requires scalar multiplication of a secp256k1 curve point.
   *
   * Due to the lack of a cheap secp256k1 ECMUL precompile, the construction misuses
   * the ECRECOVER precompile to compute the scalar multiplication over secp256k1.
   * Although this returns an Ethereum address, the result is usable to validate the response
   * against the stated challenge.
   *
   * For more information see
   * https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
   *
   * @param scalar to multiply with secp256k1 base point
   */
  function scalarTimesBasepoint(uint256 scalar) public pure returns (address) {
    return
      ecrecover(
        0,
        SECP256K1_BASE_POINT_Y_COMPONENT_SIGN,
        bytes32(SECP256K1_BASE_POINT_X_COMPONENT),
        bytes32(mulmod(scalar, SECP256K1_BASE_POINT_X_COMPONENT, SECP256K1_FIELD_ORDER))
      );
  }

  /**
   * Computes multiplicative inverse of secp256k1 field element.
   *
   * Uses little Fermat because expmod precompile is more gas-efficient
   * than extended euclidian algorithm in Solidity.
   */
  function invMod(uint256 el) public view returns (uint256 o) {
    if (el == 0) {
      revert InvalidFieldElement();
    }

    assembly {
      //  define pointer
      let p := mload(0x40)
      // store data assembly-favouring ways
      mstore(p, 0x20)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), el)  // Base
      mstore(add(p, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 2
      mstore(add(p, 0xa0), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F) // p
      if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
        revert(0, 0)
      }

      o := mload(p)
    }
  } 

  /**
   * Performs a modular exponentiation using the expmod precompile.
   */
  function expmod(uint256 base, uint256 exponent, uint256 modulus) public view returns (uint256 o) {
    assembly {
      //  define pointer
      let p := mload(0x40)
      // store data assembly-favouring ways
      mstore(p, 0x20)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), base)  // Base
      mstore(add(p, 0x80), exponent)    // Exponent
      mstore(add(p, 0xa0), modulus)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
        revert(0, 0)
      }

      o := mload(p)
    }
  }

  /**
   * Converts a curve point to an Ethereum address.
   *
   * This function can be used to witness the result of a scalar
   * multiplication.
   */
  function pointToAddress(CurvePoint calldata p) public pure returns (address) {
    return address(uint160(uint256(keccak256(abi.encodePacked(p.x, p.y)))));
  }

  /**
   * Adds two elliptic curve points using the general implementation.
   *
   * This function is optimized to perform one point addition.
   *
   * Optimizations:
   * - solidity assembly
   * - optimize for a single addition
   */
  function ecAdd(CurvePoint memory p, CurvePoint memory q) public view returns (CurvePoint memory r)  {
    if (p.x == 0 && p.y == 0) { // Inf + Q = Q
      return q;
    }

    if (q.x == 0 && q.y == 0) { // P + Inf = P
      return p;
    }

    assembly {
      let toInvert := addmod( // q.x - p.x
        mload(q), // q.x 
        sub(SECP256K1_BASE_FIELD_ORDER, mload(p)), // - p.x
        SECP256K1_BASE_FIELD_ORDER
      )

      let payload := mload(0x40)
      mstore(payload, 0x20)             // Length of Base
      mstore(add(payload, 0x20), 0x20)  // Length of Exponent
      mstore(add(payload, 0x40), 0x20)  // Length of Modulus
      mstore(add(payload, 0x60), toInvert)  // Base
      mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
      mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
        revert(0, 0)
      }

      let lambda := mulmod( // (q.y - p.y) * (q.x - p.x) ^ -1
        addmod( // q.y - p.y
          mload(add(q, 0x20)), // q.y
          sub(SECP256K1_BASE_FIELD_ORDER, mload(add(p, 0x20))), // - p.y
          SECP256K1_BASE_FIELD_ORDER
        ), 
        mload(payload), // (q.x - p.x) ^ -1
        SECP256K1_BASE_FIELD_ORDER
      )

      mstore(
        r,
        addmod( // lambda^2 - q.x - p.x
          mulmod(lambda, lambda, SECP256K1_BASE_FIELD_ORDER), // lambda^2
          addmod( // - q.x - p.x
            sub(SECP256K1_BASE_FIELD_ORDER, mload(q)), // - q.x 
            sub(SECP256K1_BASE_FIELD_ORDER, mload(p)), // - p.x
            SECP256K1_BASE_FIELD_ORDER
          ),
          SECP256K1_BASE_FIELD_ORDER
        )
      )

      mstore(
        add(r, 0x20), // r.y
        addmod( // lambda * (p.x - r.x) - p.y
          mulmod( // lambda * (p.x - r.x)
            lambda, 
            addmod( // p.x - r.x
              mload(p), // p.x 
              sub(SECP256K1_BASE_FIELD_ORDER, mload(r)), // - r.x
              SECP256K1_BASE_FIELD_ORDER
            ), 
            SECP256K1_BASE_FIELD_ORDER
          ),
          sub(SECP256K1_BASE_FIELD_ORDER, mload(add(p, 0x20))), 
          SECP256K1_BASE_FIELD_ORDER
        )
      )
    }
  }

  /**
   * Consumes a byte string and returns a sep256k1 curvepoint.
   * 
   * Implements secp256k1_XMD:SHA-256_SSWU_RO_, see
   * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html
   */
  function hashToCurve(bytes memory payload, bytes memory DST) public view returns (CurvePoint memory) {
    (uint256 u_0, uint256 u_1) = hash_to_field(abi.encodePacked(payload), abi.encodePacked(DST));

    CurvePoint memory q_0 = map_to_curve_simple_swu(uint256(u_0));
    CurvePoint memory q_1 = map_to_curve_simple_swu(uint256(u_1));

    CurvePoint memory sum = ecAdd(q_0, q_1);

    return mapPoint(sum);
  } 

  /**
   * Map a curve point on E': y^2 = A'x^3 + B' to secp256k1. This function is necessary because
   * A*B = 0 for secp256k1 and thus the simplified SWU mapping is not applicable.
   *
   * A' := 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533
   * B' := 1771
   * modulus 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
   *
   * see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#appx-iso-secp256k1
   *
   * Optimizations:
   * - mathematical optimization: reduce expmod / mulmod / addmod operations
   * - few temporary values to reduce memory expansion
   * - inlined constants to reduce memory expansion and prevent MLOAD instructions
   */
  function mapPoint(CurvePoint memory p) public view returns (CurvePoint memory r) {
    // uint256 SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;

    // hash-to-curve: 3-isogeny map for secp256k1
    // uint256 K_10 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7;
    // uint256 K_11 = 0x07d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581;
    // uint256 K_12 = 0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262;
    // uint256 K_13 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c;
    // uint256 K_20 = 0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b;
    // uint256 K_21 = 0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14;
    // uint256 K_30 = 0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c;
    // uint256 K_31 = 0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3;
    // uint256 K_32 = 0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931;
    // uint256 K_33 = 0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84;
    // uint256 K_40 = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b;
    // uint256 K_41 = 0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573;
    // uint256 K_42 = 0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f;

    uint256 pxSquare = mulmod(p.x, p.x, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);
    uint256 pxCubic = mulmod(p.x, pxSquare, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);

    // x_num = k_(1,3) * x'^3 + k_(1,2) * x'^2 + k_(1,1) * x' + k_(1,0)
    uint256 tv1 = addmod(addmod(mulmod(0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c, pxCubic, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), mulmod(0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262, pxSquare, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F),addmod(mulmod(0x07d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581, p.x, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);
    // x_den = x'^2 + k_(2,1) * x' + k_(2,0)
    uint256 tv2 = addmod(addmod(pxSquare, mulmod(0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14, p.x, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);

    // x = x_num / x_den
    r.x = mulmod(tv1, invMod(tv2), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);

    // y_num = k_(3,3) * x'^3 + k_(3,2) * x'^2 + k_(3,1) * x' + k_(3,0)
    tv1 = addmod(addmod(mulmod(0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84, pxCubic, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), mulmod(0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931, pxSquare, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F),addmod(mulmod(0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3, p.x, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);
    // y_den = x'^3 + k_(4,2) * x'^2 + k_(4,1) * x' + k_(4,0)
    tv2 = addmod(addmod(pxCubic, mulmod(0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f, pxSquare, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), addmod(mulmod(0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573, p.x, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);
    
    // y = y' * y_num / y_den
    r.y = mulmod(mulmod(p.y, tv1, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F), invMod(tv2), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F);
  }

  /**
   * Implements the simplified SWU mapping. Uses the optimized sample implementation from
   * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-simplified-swu-method
   *
   * Optimizations:
   * - mathematical optimization: reduce expmod / mulmod / addmod operations
   * - few temporary values to reduce memory expansion
   * - inlined constants to reduce memory expansion and prevent MLOAD instructions
   */
  function map_to_curve_simple_swu(uint256 u) public view returns (CurvePoint memory r) {
    // uint256 Z = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC24;
    // uint256 C1 = 0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffffbfffff0b;
    // uint256 C2 = 0x31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a59; // sqrt(-Z)
    // uint256 SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;

    uint256 tv1 = mulmod(u, u, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 1.  tv1 = u^2
    tv1 = mulmod(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC24, tv1, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 2.  tv1 = Z * tv1
    uint256 tv2 = mulmod(tv1, tv1, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 3.  tv2 = tv1^2
    tv2 = addmod(tv2,tv1, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 4.  tv2 = tv2 + tv1
    uint256 tv3 = addmod(tv2, 1, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 5.  tv3 = tv2 + 1
    tv3 = mulmod(tv3, 1771, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 6.  tv3 = B * tv3

    uint256 tv4;
    if (tv2 == 0) { // 7.  tv4 = CMOV(Z, -tv2, tv2 != 0)
      tv4 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC24;
    } else {
      tv4 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F - tv2;
    }
    tv4 = mulmod(0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533, tv4, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 8.  tv4 = A * tv4
    tv2 = mulmod(tv3, tv3, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 9.  tv2 = tv3^2
    uint256 tv6 = mulmod(tv4, tv4, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 10. tv6 = tv4^2
    uint256 tv5 = mulmod(0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533, tv6, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 11. tv5 = A * tv6
    tv2 = addmod(tv2, tv5, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 12. tv2 = tv2 + tv5
    tv2 = mulmod(tv2, tv3, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 13. tv2 = tv2 * tv3
    tv6 = mulmod(tv6, tv4, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 14. tv6 = tv6 * tv4
    tv5 = mulmod(1771, tv6, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 15. tv5 = B * tv6
    tv2 = addmod(tv2, tv5, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 16. tv2 = tv2 + tv5
    uint256 x = mulmod(tv1, tv3, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 17.   x = tv1 * tv3

    (bool is_square, uint256 y1) = sqrt_ratio_3mod4(tv2, tv6); // 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)

    uint256 y = mulmod(tv1, u, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 19.   y = tv1 * u
    y = mulmod(y, y1, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 20.   y = y * y1

    if (is_square) {
      x = tv3; // 21.   x = CMOV(x, tv3, is_gx1_square)
      y = y1; // 22.   y = CMOV(y, y1, is_gx1_square)
    }

    if (u % 2 != y % 2) { // 23.  e1 = sgn0(u) == sgn0(y)
      y = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F - y; // 24.   y = CMOV(-y, y, e1)
    }

    r.x = mulmod(x, invMod(tv4), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 25.   x = x / tv4
    r.y = y;
  }

  /**
   * Square root subroutine used by simplified SWU mapping.
   *
   * Returns (true, sqrt(u / v)) if (u / v) is square in the field
   * Returns (false, sqrt(Z * (u / v))) otherwise
   *
   * Optimizations:
   * - mathematical optimization: reduce expmod / mulmod / addmod operations
   * - few temporary values to reduce memory expansion
   * - inlined constants to reduce memory expansion and prevent MLOAD instructions
   */
  function sqrt_ratio_3mod4(uint256 u, uint256 v) public view returns (bool, uint256) {
    // uint256 C1 = 0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffffbfffff0b;
    // uint256 C2 = 0x31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a59;
    // uint256 SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;

    uint256 tv1 = mulmod(v,v, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 1. tv1 = v^2
    uint256 tv2 = mulmod(u,v, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 2. tv2 = u * v
    tv1 = mulmod(tv1, tv2, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 3. tv1 = tv1 * tv2
    uint256 y1 = expmod(tv1, 0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffffbfffff0b, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 4. y1 = tv1^c1
    y1 = mulmod(y1, tv2, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 5. y1 = y1 * tv2
    uint256 y2 = mulmod(y1, 0x31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a59, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 6. y2 = y1 * c2
    uint256 tv3 = mulmod(y1, y1, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 7. tv3 = y1^2
    tv3 = mulmod(tv3, v, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F); // 8. tv3 = tv3 * v

    if (tv3 == u) { // 9. isQR = tv3 == u
      return (true, y1);
    } else {
      return (false, y2);
    } // 10. y = CMOV(y2, y1, isQR)
  }

  /**
   * Consumes a byte-string and returns a field element of the secp256k1 base field.

   */
  function hash_to_field(bytes memory message, bytes memory DST) public view returns (uint256 u_0, uint256 u_1) {
    // uint256 SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;

    (bytes32 b_1, bytes32 b_2, bytes32 b_3) = expand_message_xmd(message, DST);

    // computes [...b_1[..], ...b_2[0..16]] ^ 1 mod n
    assembly {
      //  define pointer
      let p := mload(0x40)
      mstore(p, 0x30)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), b_1)  // Base
      mstore(add(p, 0x80), b_2)
      mstore(add(p, 0x90), 1)    // Exponent
      mstore(add(p, 0xb0), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) {
        revert(0, 0)
      }

      u_0 := mload(p)
    }

    // computes [...b_2[16..32], ...b_3[..]] ^ 1 mod n
    assembly {
      //  define pointer
      let p := mload(0x40)

      mstore(p, 0x30)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x50), b_2)
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x70), b_3)  // Base
      mstore(add(p, 0x90), 1)    // Exponent
      mstore(add(p, 0xb0), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) {
        revert(0, 0)
      }

      u_1 := mload(p)
    }

  }

  /**
   * Expands a byte-string to 96 bits using the `expand_message_method` described in
   * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html
   *
   * This is not a general implementation as the output length fixed.
   */
  function expand_message_xmd(bytes memory message, bytes memory DST) public pure returns (bytes32 b_1, bytes32 b_2, bytes32 b_3) {
    uint256 ell; 

    if (message.length % 32 == 0) {
      ell = message.length / 32;
    } else {
      ell = message.length / 32 + 1;
    }

    if (ell > 255 || message.length > 2040 || DST.length > 255) {
      revert MessageTooLong();
    }

    bytes memory DST_prime = abi.encodePacked(DST, uint8(DST.length));
    bytes memory message_prime = abi.encodePacked(
      bytes32(0), bytes32(0), // block size of SHA256 is 64
      message, 
      bytes2(0x0060), // only supports 96 bytes output length
      uint8(0), DST_prime); 
    
    bytes32 b_0 = sha256(message_prime);
    b_1 = sha256(abi.encodePacked(b_0, uint8(1), DST_prime));
    b_2 = sha256(abi.encodePacked(b_0 ^ b_1, uint8(2), DST_prime));
    b_3 = sha256(abi.encodePacked(b_0 ^ b_2, uint8(3), DST_prime));
  }

  // ############ Alternative Implementations


  // function sqrtMod(uint256 el) public view returns (uint256 o) {
  //   assembly {
  //     //  define pointer
  //     let p := mload(0x40)
  //     // store data assembly-favouring ways
  //     mstore(p, 0x20)             // Length of Base
  //     mstore(add(p, 0x20), 0x20)  // Length of Exponent
  //     mstore(add(p, 0x40), 0x20)  // Length of Modulus
  //     mstore(add(p, 0x60), el)  // Base
  //     mstore(add(p, 0x80), div(add(SECP256K1_BASE_FIELD_ORDER, 1), 4))    // Exponent
  //     mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
  //     if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
  //       revert(0, 0)
  //     }

  //     o := mload(p)
  //   }
  // }
  
  // function map_to_curve_simple_swu(uint256 u) public view returns (CurvePoint memory r) {
  // 1. tv1 = inv0(Z^2 * u^4 + Z * u^2)
  // 2.  x1 = (-B / A) * (1 + tv1)
  // 3.  If tv1 == 0, set x1 = B / (Z * A)
  // 4. gx1 = x1^3 + A * x1 + B
  // 5.  x2 = Z * u^2 * x1
  // 6. gx2 = x2^3 + A * x2 + B
  // 7.  If is_square(gx1), set x = x1 and y = sqrt(gx1)
  // 8.  Else set x = x2 and y = sqrt(gx2)
  // 9.  If sgn0(u) != sgn0(y), set y = -y
  // 10. return (x, y)
  //   uint256 uSquare = mulmod(u, u, SECP256K1_BASE_FIELD_ORDER);
  //   uint256 zUSquare = mulmod(uSquare, Z, SECP256K1_BASE_FIELD_ORDER);
  //   uint256 uPowerFour = mulmod(uSquare, uSquare, SECP256K1_BASE_FIELD_ORDER); 
  //   uint256 zSquareUPowerFour = mulmod(mulmod(Z, Z, SECP256K1_BASE_FIELD_ORDER), uPowerFour, SECP256K1_BASE_FIELD_ORDER);
  //   uint256 tv1 = invMod(addmod(zSquareUPowerFour, zUSquare, SECP256K1_BASE_FIELD_ORDER));

  //   uint256 x1;
  //   if (tv1 == 0 || tv1 == SECP256K1_BASE_FIELD_ORDER) {
  //     x1 = mulmod(B_Prime, invMod(mulmod(Z, A_Prime, SECP256K1_BASE_FIELD_ORDER)), SECP256K1_BASE_FIELD_ORDER);
  //   } else {
  //     x1 = mulmod(mulmod((SECP256K1_BASE_FIELD_ORDER - B_Prime), invMod(A_Prime), SECP256K1_BASE_FIELD_ORDER), 1 + tv1, SECP256K1_BASE_FIELD_ORDER);
  //   }

  //   uint256 gx1 = addmod(addmod(mulmod(x1, mulmod(x1, x1, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER), mulmod(A_Prime, x1, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER), B_Prime, SECP256K1_BASE_FIELD_ORDER);

  //   if (isSquare(gx1)) {
  //     r.x = x1;
  //     r.y = sqrtMod(gx1);
  //   } else {
  //     uint256 x2 = mulmod(zUSquare, x1, SECP256K1_BASE_FIELD_ORDER);
  //     uint256 gx2 = addmod(addmod(mulmod(x2, mulmod(x2, x2, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER), mulmod(A_Prime, x2, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER), B_Prime, SECP256K1_BASE_FIELD_ORDER);

  //     r.x = x2;
  //     r.y = sqrtMod(gx2);
  //   }

  //   if (u % 2 != r.y % 2) {
  //     r.y = SECP256K1_BASE_FIELD_ORDER - r.y;
  //   }
  // }
  
  // function isSquare(uint256 el) public view returns (bool res) {
  //   assembly {
  //     //  define pointer
  //     let p := mload(0x40)
  //     // store data assembly-favouring ways
  //     mstore(p, 0x20)             // Length of Base
  //     mstore(add(p, 0x20), 0x20)  // Length of Exponent
  //     mstore(add(p, 0x40), 0x20)  // Length of Modulus
  //     mstore(add(p, 0x60), el)  // Base
  //     mstore(add(p, 0x80), div(sub(SECP256K1_BASE_FIELD_ORDER, 1), 2))    // Exponent
  //     mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
  //     if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
  //       revert(0, 0)
  //     }

  //     switch mload(p) 
  //     case 0 {
  //       res := true
  //     }
  //     case 1 {
  //       res := true
  //     }
  //     case 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F {
  //       res := true
  //     }
  //     default {
  //       res := false
  //     }
  //   }
  // }

  // function edAdd(CurvePoint calldata p, CurvePoint calldata q) returns (CurvePoint calldata r) {
  //   uint256 lambda = mulmod( // (q.y - p.y) * (q.x - p.x) ^ -1
  //     addmod( // q.y - p.y
  //       q.y, 
  //       SECP256K1_BASE_FIELD_ORDER - p.y, // - p.y
  //       SECP256K1_BASE_FIELD_ORDER
  //     ), 
  //     invMod( // (q.x - p.x) ^ -1
  //       addmod( // q.x - p.x
  //         q.x, 
  //         SECP256K1_BASE_FIELD_ORDER - p.x, // - p.x
  //         SECP256K1_BASE_FIELD_ORDER
  //       )
  //     ), SECP256K1_BASE_FIELD_ORDER);

  //   r.x = 
  //     addmod( // lambda^2 - q.x - p.x
  //       mulmod(lambda, lambda, SECP256K1_BASE_FIELD_ORDER), // lambda^2
  //       addmod( // - q.x - p.x
  //         SECP256K1_BASE_FIELD_ORDER - q.x, // - q.x 
  //         SECP256K1_BASE_FIELD_ORDER - p.x, // - p.x
  //         SECP256K1_BASE_FIELD_ORDER),
  //       SECP256K1_BASE_FIELD_ORDER
  //     );
  //   r.y = 
  //     addmod( // lambda * (p.x - r.x) - p.y
  //       mulmod( // lambda * (p.x - r.x)
  //         lambda, 
  //         addmod( // p.x - r.x
  //           p.x, 
  //           SECP256K1_BASE_FIELD_ORDER - r.x, // - r.x
  //           SECP256K1_BASE_FIELD_ORDER
  //         ), 
  //         SECP256K1_BASE_FIELD_ORDER
  //       ),
  //       SECP256K1_BASE_FIELD_ORDER - p.y, 
  //       SECP256K1_BASE_FIELD_ORDER
  //     );
  // }
}
