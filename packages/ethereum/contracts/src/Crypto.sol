// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

error InvalidFieldElement();
error InvalidCurvePoint();

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

  // hash-to-curve: 3-isogeny map for secp256k1
  uint256 K_10 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7;
  uint256 K_11 = 0x07d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581;
  uint256 K_12 = 0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262;
  uint256 K_13 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c;
  uint256 K_20 = 0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b;
  uint256 K_21 = 0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14;
  uint256 K_30 = 0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c;
  uint256 K_31 = 0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3;
  uint256 K_32 = 0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931;
  uint256 K_33 = 0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84;
  uint256 K_40 = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b;
  uint256 K_41 = 0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573;
  uint256 K_42 = 0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f;


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
  modifier isCurvePoint(CurvePoint calldata p)  {
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
      return 0;
    }
    assembly {
      //  define pointer
      let p := mload(0x40)
      // store data assembly-favouring ways
      mstore(p, 0x20)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), el)  // Base
      mstore(add(p, 0x80), sub(SECP256K1_BASE_FIELD_ORDER, 2))     // Exponent
      mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
        revert(0, 0)
      }

      o := mload(p)
    }
  } 

  function sqrtMod(uint256 el) public view returns (uint256 o) {
    assembly {
      //  define pointer
      let p := mload(0x40)
      // store data assembly-favouring ways
      mstore(p, 0x20)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), el)  // Base
      mstore(add(p, 0x80), div(add(SECP256K1_BASE_FIELD_ORDER, 1), 4))    // Exponent
      mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
        revert(0, 0)
      }

      o := mload(p)
    }
  }

  function isSquare(uint256 el) public view returns (bool res) {
    assembly {
      //  define pointer
      let p := mload(0x40)
      // store data assembly-favouring ways
      mstore(p, 0x20)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), el)  // Base
      mstore(add(p, 0x80), div(sub(SECP256K1_BASE_FIELD_ORDER, 1), 2))    // Exponent
      mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) {
        revert(0, 0)
      }

      switch mload(p) 
      case 0 {
        res := true
      }
      case 1 {
        res := true
      }
      case 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F {
        res := true
      }
      default {
        res := false
      }
    }
  }

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
   * Converts a curve point to an Ethereum address
   */
  function pointToAddress(CurvePoint calldata p) public pure returns (address) {
    return address(uint160(uint256(keccak256(abi.encodePacked(p.x, p.y)))));
  }

  function ecAdd(CurvePoint calldata p, CurvePoint calldata q) public view returns (CurvePoint memory r)  {
    if (p.x == 0 && p.y == 0) { // Inf + Q = Q
      r.x = q.x;
      r.y = q.y;
    }

    if (q.x == 0 && q.y == 0) { // P + Inf = P
      r.x = p.x;
      r.y = p.y;
    }

    // uint256 lambda = mulmod( // (q.y - p.y) * (q.x - p.x) ^ -1
    //   addmod( // q.y - p.y
    //     q.y, 
    //     SECP256K1_BASE_FIELD_ORDER - p.y, // - p.y
    //     SECP256K1_BASE_FIELD_ORDER
    //   ), 
    //   invMod( // (q.x - p.x) ^ -1
    //     addmod( // q.x - p.x
    //       q.x, 
    //       SECP256K1_BASE_FIELD_ORDER - p.x, // - p.x
    //       SECP256K1_BASE_FIELD_ORDER
    //     )
    //   ), SECP256K1_BASE_FIELD_ORDER);

    // r.x = 
    //   addmod( // lambda^2 - q.x - p.x
    //     mulmod(lambda, lambda, uint256(SECP256K1_BASE_FIELD_ORDER)), // lambda^2
    //     addmod( // - q.x - p.x
    //       SECP256K1_BASE_FIELD_ORDER - q.x, // - q.x 
    //       SECP256K1_BASE_FIELD_ORDER - p.x, // - p.x
    //       SECP256K1_BASE_FIELD_ORDER),
    //     SECP256K1_BASE_FIELD_ORDER
    //   );
    // r.y = 
    //   addmod( // lambda * (p.x - r.x) - p.y
    //     mulmod( // lambda * (p.x - r.x)
    //       lambda, 
    //       addmod( // p.x - r.x
    //         p.x, 
    //         SECP256K1_BASE_FIELD_ORDER - r.x, // - r.x
    //         SECP256K1_BASE_FIELD_ORDER
    //       ), 
    //       SECP256K1_BASE_FIELD_ORDER
    //     ),
    //     SECP256K1_BASE_FIELD_ORDER - p.y, 
    //     SECP256K1_BASE_FIELD_ORDER
    //   )
    // ;

    assembly {
      let toInvert := addmod( // q.x - p.x
        calldataload(q), // q.x 
        sub(SECP256K1_BASE_FIELD_ORDER, calldataload(p)), // - p.x
        SECP256K1_BASE_FIELD_ORDER
      )

      let payload := mload(0x40)
      // store data assembly-favouring ways
      mstore(payload, 0x20)             // Length of Base
      mstore(add(payload, 0x20), 0x20)  // Length of Exponent
      mstore(add(payload, 0x40), 0x20)  // Length of Modulus
      mstore(add(payload, 0x60), toInvert)  // Base
      mstore(add(payload, 0x80), sub(SECP256K1_BASE_FIELD_ORDER, 2))     // Exponent
      mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
        revert(0, 0)
      }

      let lambda := mulmod( // (q.y - p.y) * (q.x - p.x) ^ -1
        addmod( // q.y - p.y
          calldataload(add(q, 0x20)), // q.y
          sub(SECP256K1_BASE_FIELD_ORDER, calldataload(add(p, 0x20))), // - p.y
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
            sub(SECP256K1_BASE_FIELD_ORDER, calldataload(q)), // - q.x 
            sub(SECP256K1_BASE_FIELD_ORDER, calldataload(p)), // - p.x
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
              calldataload(p), // p.x 
              sub(SECP256K1_BASE_FIELD_ORDER, mload(r)), // - r.x
              SECP256K1_BASE_FIELD_ORDER
            ), 
            SECP256K1_BASE_FIELD_ORDER
          ),
          sub(SECP256K1_BASE_FIELD_ORDER, calldataload(add(p, 0x20))), 
          SECP256K1_BASE_FIELD_ORDER
        )
      )
    }
  }

  function mapPoint(CurvePoint memory p) public view returns (CurvePoint memory r) {
    uint256 pxSquare = mulmod(p.x, p.x, SECP256K1_BASE_FIELD_ORDER);
    uint256 pxCubic = mulmod(p.x, pxSquare, SECP256K1_BASE_FIELD_ORDER);

    uint256 x_num = addmod(addmod(mulmod(K_13, pxCubic, SECP256K1_BASE_FIELD_ORDER), mulmod(K_12, pxSquare, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER),addmod(mulmod(K_11, p.x, SECP256K1_BASE_FIELD_ORDER), K_10, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER);
    uint256 x_den = addmod(addmod(pxSquare, mulmod(K_21, p.x, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER), K_20, SECP256K1_BASE_FIELD_ORDER);

    r.x = mulmod(x_num, invMod(x_den), SECP256K1_BASE_FIELD_ORDER);

    uint256 y_num = addmod(addmod(mulmod(K_33, pxCubic, SECP256K1_BASE_FIELD_ORDER), mulmod(K_32, pxSquare, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER),addmod(mulmod(K_31, p.x, SECP256K1_BASE_FIELD_ORDER), K_30, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER);
    uint256 y_den = addmod(addmod(pxCubic, mulmod(K_42, pxSquare, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER), addmod(mulmod(K_41, p.x, SECP256K1_BASE_FIELD_ORDER), K_40, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER);
    
    r.y = mulmod(mulmod(p.y, y_num, SECP256K1_BASE_FIELD_ORDER), invMod(y_den), SECP256K1_BASE_FIELD_ORDER);
  }

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

  // function map_to_curve_simple_swu(uint256 u) public view returns (CurvePoint memory r) {
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



  // Z = -11
  // A = 3
  // B = 7

  uint256 constant A_Prime = 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533;
  uint256 constant B_Prime = 1771;

  uint256 constant Z = SECP256K1_BASE_FIELD_ORDER - 11;
  uint256 constant Sqrt_Minus_Z = 0x31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a59;

  uint256 constant C1 = 0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffffbfffff0b;
//   Steps:
// 1.  tv1 = u^2
// 2.  tv1 = Z * tv1
// 3.  tv2 = tv1^2
// 4.  tv2 = tv2 + tv1
// 5.  tv3 = tv2 + 1
// 6.  tv3 = B * tv3
// 7.  tv4 = CMOV(Z, -tv2, tv2 != 0)
// 8.  tv4 = A * tv4
// 9.  tv2 = tv3^2
// 10. tv6 = tv4^2
// 11. tv5 = A * tv6
// 12. tv2 = tv2 + tv5
// 13. tv2 = tv2 * tv3
// 14. tv6 = tv6 * tv4
// 15. tv5 = B * tv6
// 16. tv2 = tv2 + tv5
// 17.   x = tv1 * tv3
// 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)
// 19.   y = tv1 * u
// 20.   y = y * y1
// 21.   x = CMOV(x, tv3, is_gx1_square)
// 22.   y = CMOV(y, y1, is_gx1_square)
// 23.  e1 = sgn0(u) == sgn0(y)
// 24.   y = CMOV(-y, y, e1)
// 25.   x = x / tv4
// 26. return (x, y)


  function map_to_curve_simple_swu(uint256 u) public view returns (CurvePoint memory r) {
    uint256 tv1 = mulmod(u, u, SECP256K1_BASE_FIELD_ORDER); // 1.  tv1 = u^2
    tv1 = mulmod(SECP256K1_BASE_FIELD_ORDER - 11, tv1, SECP256K1_BASE_FIELD_ORDER); // 2.  tv1 = Z * tv1
    uint256 tv2 = mulmod(tv1, tv1, SECP256K1_BASE_FIELD_ORDER); // 3.  tv2 = tv1^2
    tv2 = addmod(tv2,tv1, SECP256K1_BASE_FIELD_ORDER); // 4.  tv2 = tv2 + tv1
    uint256 tv3 = addmod(tv2, 1, SECP256K1_BASE_FIELD_ORDER); // 5.  tv3 = tv2 + 1
    tv3 = mulmod(tv3, B_Prime, SECP256K1_BASE_FIELD_ORDER); // 6.  tv3 = B * tv3

    uint256 tv4;
    if (tv2 == 0) { // 7.  tv4 = CMOV(Z, -tv2, tv2 != 0)
      tv4 =  SECP256K1_BASE_FIELD_ORDER -11;
    } else {
      tv4 = SECP256K1_BASE_FIELD_ORDER - tv2;
    }
    tv4 = mulmod(A_Prime, tv4, SECP256K1_BASE_FIELD_ORDER); // 8.  tv4 = A * tv4
    tv2 = mulmod(tv3, tv3, SECP256K1_BASE_FIELD_ORDER); // 9.  tv2 = tv3^2
    uint256 tv6 = mulmod(tv4, tv4, SECP256K1_BASE_FIELD_ORDER); // 10. tv6 = tv4^2
    uint256 tv5 = mulmod(A_Prime, tv6, SECP256K1_BASE_FIELD_ORDER); // 11. tv5 = A * tv6
    tv2 = addmod(tv2, tv5, SECP256K1_BASE_FIELD_ORDER); // 12. tv2 = tv2 + tv5
    tv2 = mulmod(tv2, tv3, SECP256K1_BASE_FIELD_ORDER); // 13. tv2 = tv2 * tv3
    tv6 = mulmod(tv6, tv4, SECP256K1_BASE_FIELD_ORDER); // 14. tv6 = tv6 * tv4
    tv5 = mulmod(B_Prime, tv6, SECP256K1_BASE_FIELD_ORDER); // 15. tv5 = B * tv6
    r.x = mulmod(tv1, tv3, SECP256K1_BASE_FIELD_ORDER); // 17.   x = tv1 * tv3

    (bool is_square, uint256 y1) = sqrt_ratio_3mod4(tv2, tv6); // 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)


    r.y = mulmod(tv1, u, SECP256K1_BASE_FIELD_ORDER); // 19.   y = tv1 * u
    r.y = mulmod(r.y, y1, SECP256K1_BASE_FIELD_ORDER); // 20.   y = y * y1

    if (is_square) {
      r.x = tv3; // 21.   x = CMOV(x, tv3, is_gx1_square)
      r.y = y1; // 22.   y = CMOV(y, y1, is_gx1_square)
    }

    if (u % 2 != r.y % 2) { // 23.  e1 = sgn0(u) == sgn0(y)
      r.y = SECP256K1_BASE_FIELD_ORDER - r.y; // 24.   y = CMOV(-y, y, e1)
    }

    r.x = mulmod(r.x, invMod(tv4), SECP256K1_BASE_FIELD_ORDER); // 25.   x = x / tv4

  }

  function sqrt_ratio_3mod4(uint256 u, uint256 v) public view returns (bool, uint256) {
    uint256 tv1 = mulmod(v,v, SECP256K1_BASE_FIELD_ORDER); // 1. tv1 = v^2
    uint256 tv2 = mulmod(u,v, SECP256K1_BASE_FIELD_ORDER); // 2. tv2 = u * v
    tv1 = mulmod(tv1, tv2, SECP256K1_BASE_FIELD_ORDER); // 3. tv1 = tv1 * tv2
    uint256 y1 = expmod(tv1, C1, SECP256K1_BASE_FIELD_ORDER); // 4. y1 = tv1^c1
    y1 = mulmod(y1, tv2, SECP256K1_BASE_FIELD_ORDER); // 5. y1 = y1 * tv2
    uint256 y2 = mulmod(y1, Sqrt_Minus_Z, SECP256K1_BASE_FIELD_ORDER); // 6. y2 = y1 * c2
    uint256 tv3 = mulmod(y1, y1, SECP256K1_BASE_FIELD_ORDER); // 7. tv3 = y1^2
    tv3 = mulmod(tv3, v, SECP256K1_BASE_FIELD_ORDER); // 8. tv3 = tv3 * v

    if (tv3 == u) { // 9. isQR = tv3 == u
      return (true, y1);
    } else {
      return (false, y2);
    } // 10. y = CMOV(y2, y1, isQR)
  }
//     sqrt_ratio_3mod4(u, v)

// Parameters:
// - F, a finite field of characteristic p and order q = p^m,
//   where q = 3 mod 4.
// - Z, the constant from the simplified SWU map.

// Input: u and v, elements of F, where v != 0.
// Output: (b, y), where
//   b = True and y = sqrt(u / v) if (u / v) is square in F, and
//   b = False and y = sqrt(Z * (u / v)) otherwise.

// Constants:
// 1. c1 = (q - 3) / 4     # Integer arithmetic = 28948022309329048855892746252171976963317496166410141009864396001977208667915
// 2. c2 = sqrt(-Z)

// Procedure:
// 1. tv1 = v^2
// 2. tv2 = u * v
// 3. tv1 = tv1 * tv2
// 4. y1 = tv1^c1
// 5. y1 = y1 * tv2
// 6. y2 = y1 * c2
// 7. tv3 = y1^2
// 8. tv3 = tv3 * v
// 9. isQR = tv3 == u
// 10. y = CMOV(y2, y1, isQR)
// 11. return (isQR, y)
  // }
}