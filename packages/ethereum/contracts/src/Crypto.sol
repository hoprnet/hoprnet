// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

import "forge-std/console2.sol";

error InvalidFieldElement();
error InvalidCurvePoint();
error InvalidPointWitness();
error GeneralError();
error MessageTooLong();

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
 **/
abstract contract HoprCrypto {
  // secp256k1: y^2 = x^3 + 7 (mod F_p)
  uint256 constant SECP256K1_B = 0x0000000000000000000000000000000000000000000000000000000000000007;
  // Field order created by secp256k1 curve
  uint256 constant SECP256K1_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

  // Order of the underlying field used for secp256k1
  uint256 constant SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;
  // x-component of base point of secp256k1 curve
  uint256 constant SECP256K1_BASE_POINT_X_COMPONENT = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;

  // encoded sign of y-component of base point of secp256k1 curve
  uint8 constant SECP256K1_BASE_POINT_Y_COMPONENT_SIGN = 27;

  // E': y^2 = x^3 + A_Prime + B_Prime (mod F_p)
  // used by `hash_to_curve` function
  uint256 constant A_Prime = 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533;
  uint256 constant B_Prime = 1771;

  // Coefficients used for isogeneous mapping from E' to secp256k1
  // see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#appx-iso-secp256k1
  //
  // used by `hash_to_curve` function
  uint256 constant K_10 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7;
  uint256 constant K_11 = 0x07d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581;
  uint256 constant K_12 = 0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262;
  uint256 constant K_13 = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c;
  uint256 constant K_20 = 0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b;
  uint256 constant K_21 = 0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14;
  uint256 constant K_30 = 0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c;
  uint256 constant K_31 = 0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3;
  uint256 constant K_32 = 0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931;
  uint256 constant K_33 = 0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84;
  uint256 constant K_40 = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b;
  uint256 constant K_41 = 0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573;
  uint256 constant K_42 = 0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f;

  // Coefficients used for simplified SWU mapping
  // see https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-suites-for-secp256k1
  //
  // used by `hash_to_curve` function
  uint256 constant Z = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC24;
  uint256 constant C_1 = 0x3fffffffffffffffffffffffffffffffffffffffffffffffffffffffbfffff0b;
  uint256 constant C_2 = 0x31fdf302724013e57ad13fb38f842afeec184f00a74789dd286729c8303c4a59; // sqrt(-Z)

  uint256 constant KECCAK256_BLOCKSIZE = 136;
  /**
   * Holds a compact ECDSA signature, following ERC-2098
   */
  struct CompactSignature {
    bytes32 r;
    bytes32 vs;
  }

  /**
   * Checks whether given value is an element of the secp256k1 field
   */
  function isFieldElementInternal(uint256 el) public pure returns (bool) {
    return 0 == el || el < SECP256K1_FIELD_ORDER;
  }

  /**
   * Checks whether given coordinates fulfill the secp256k1 curve equation
   */
  function isCurvePointInternal(uint256 p_x, uint256 p_y) public pure returns (bool r) {
    assembly {
      r := eq(
        mulmod(
          p_y, 
          p_y, 
          SECP256K1_BASE_FIELD_ORDER
        ), 
        addmod(
          SECP256K1_B, 
          mulmod(
            mulmod(
              p_x, 
              p_x, 
              SECP256K1_BASE_FIELD_ORDER), 
            p_x, 
            SECP256K1_BASE_FIELD_ORDER),
          SECP256K1_BASE_FIELD_ORDER
        ))
    }
  }

  modifier isCurvePoint(uint256 p_x, uint256 p_y) {
    if (!isCurvePointInternal(p_x, p_y)) {
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
   * Takes a curve point `P = (p_x, p_y)` and a scalar and returns the Ethereum address associated
   * to the point `scalar * P` on the secp256k1 curve.
   *
   * This function is necessary due to the missing ECMUL operation in Ethereum. It misuses the
   * ECRECOVER precompile to perform the scalar multiplication in a gas-efficient way
   *
   * For more information see
   * https://ethresear.ch/t/you-can-kinda-abuse-ecrecover-to-do-ecmul-in-secp256k1-today/2384
   */
  function scalarPointMultiplication(uint256 scalar, uint256 p_x, uint256 p_y) internal pure returns (address) {
    uint8 sign;
    if (p_y % 2 == 0) {
      sign = 27;
    } else {
      sign = 28;
    }

    return
      ecrecover(
        0,
        sign,
        bytes32(p_x),
        bytes32(mulmod(scalar, p_x, SECP256K1_FIELD_ORDER))
      );
  }

  /**
   * Converts a curve point to an Ethereum address.
   *
   * This function can be used to witness the result of a scalar
   * multiplication.
   */
  function pointToAddress(uint256 p_x, uint256 p_y) public pure returns (address) {
    return address(uint160(uint256(keccak256(abi.encodePacked(p_x, p_y)))));
  }

  /**
   * Adds two elliptic curve points using the general implementation.
   *
   * This function is optimized to perform one single point addition, e.g.
   * when using in a VRF or hash_to_curve scheme.
   *
   * Optimizations:
   * - solidity assembly
   * - optimize for a single point addition
   * - inline modular inversion
   */
  function ecAdd(uint256 p_x, uint256 p_y, uint256 q_x, uint256 q_y, uint256 a) public view returns (uint256 r_x, uint256 r_y)  {
    assembly {
      if and(eq(p_x, q_x), not(eq(p_y, q_y))) {
        // Q = -P
        // which means P + Q = P - P = 0 which is not supported
        revert (0,0)
      }
      let lambda
      let toInvert
      switch and(eq(p_x, q_x), eq(p_y, q_y)) // P == Q ?
      case true {
        // Point double
        toInvert := addmod(mulmod(2, p_y, SECP256K1_BASE_FIELD_ORDER), a, SECP256K1_BASE_FIELD_ORDER) // 2 * p.y

        // Compute (2 * p.y) ^ -1
        let payload := mload(0x40)
        mstore(payload, 0x20)             // Length of Base
        mstore(add(payload, 0x20), 0x20)  // Length of Exponent
        mstore(add(payload, 0x40), 0x20)  // Length of Modulus
        mstore(add(payload, 0x60), toInvert)  // Base
        mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
        mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
        if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) { // 0x05 == expmod precompile
          revert(0, 0)
        }
        lambda := mulmod( // (3 * p.x ^ 2) * (2 * p.y) ^ -1
          mulmod( // 3 * p.x ^ 2
            3, 
            mulmod(p_x, p_x, SECP256K1_BASE_FIELD_ORDER), SECP256K1_BASE_FIELD_ORDER), mload(payload), 
            SECP256K1_BASE_FIELD_ORDER
          )
      }
      case false {
        // Point addition
        toInvert := addmod( // q.x - p.x
          q_x, // q.x 
          sub(SECP256K1_BASE_FIELD_ORDER, p_x), // - p.x
          SECP256K1_BASE_FIELD_ORDER
        )

        let payload := mload(0x40)
        mstore(payload, 0x20)             // Length of Base
        mstore(add(payload, 0x20), 0x20)  // Length of Exponent
        mstore(add(payload, 0x40), 0x20)  // Length of Modulus
        mstore(add(payload, 0x60), toInvert)  // Base
        mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
        mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
        if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) { // 0x05 == expmod precompile
          revert(0, 0)
        }

        lambda := mulmod( // (q.y - p.y) * (q.x - p.x) ^ -1
          addmod( // q.y - p.y
            q_y, // q.y
            sub(SECP256K1_BASE_FIELD_ORDER, p_y), // - p.y
            SECP256K1_BASE_FIELD_ORDER
          ), 
          mload(payload), // (q.x - p.x) ^ -1
          SECP256K1_BASE_FIELD_ORDER
        )
      }

      r_x :=
        addmod( // lambda^2 - q.x - p.x
          mulmod(lambda, lambda, SECP256K1_BASE_FIELD_ORDER), // lambda^2
          addmod( // - q.x - p.x
            sub(SECP256K1_BASE_FIELD_ORDER, q_x), // - q.x 
            sub(SECP256K1_BASE_FIELD_ORDER, p_x), // - p.x
            SECP256K1_BASE_FIELD_ORDER
          ),
          SECP256K1_BASE_FIELD_ORDER
        )

      r_y :=
        addmod( // lambda * (p.x - r.x) - p.y
          mulmod( // lambda * (p.x - r.x)
            lambda, 
            addmod( // p.x - r.x
              p_x, // p.x 
              sub(SECP256K1_BASE_FIELD_ORDER, r_x), // - r.x
              SECP256K1_BASE_FIELD_ORDER
            ), 
            SECP256K1_BASE_FIELD_ORDER
          ),
          sub(SECP256K1_BASE_FIELD_ORDER, p_y), 
          SECP256K1_BASE_FIELD_ORDER
        )
      
    }
  }

  /**
   * Consumes a byte string and returns a pseudo-random sep256k1 curvepoint.
   * 
   * Implements secp256k1_XMD:KECCAK_256_SSWU_RO_, see
   * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html
   */
  function hashToCurve(bytes memory payload, bytes memory DST) public view returns (uint256 r_x, uint256 r_y) {
    // uint256 A_Prime = 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533

    (uint256 u_0, uint256 u_1) = hash_to_field(payload, DST);

    (uint256 q_0_x, uint256 q_0_y) = map_to_curve_simple_swu(uint256(u_0)); // on isogenous curve
    (uint256 q_1_x, uint256 q_1_y) = map_to_curve_simple_swu(uint256(u_1)); // on isogenous curve

    // P + Q on isogenous curve
    (uint256 s_x, uint256 s_y) = ecAdd(q_0_x,q_0_y,q_1_x,q_1_y, 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533);

    return mapPoint(s_x, s_y);
  } 


  /**
   * Maps a curve point on E': y^2 = A'x^3 + B' to secp256k1. This function is necessary because
   * A*B = 0 for secp256k1 which is why the simplified SWU mapping is not directly applicable.
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
   * - use Solidity assembly
   */
  function mapPoint(uint256 p_x, uint256 p_y) public view returns (uint256 r_x, uint256 r_y) {
    // uint256 SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;
    assembly {
      let pxSquare := mulmod(p_x, p_x, SECP256K1_BASE_FIELD_ORDER) // p.x * p.x
      let pxCubic := mulmod(p_x, pxSquare, SECP256K1_BASE_FIELD_ORDER) // p.x * pxSquare

      // x_num = k_(1,3) * x'^3 + k_(1,2) * x'^2 + k_(1,1) * x' + k_(1,0)
      let x_num := addmod(
        addmod(
          mulmod(
            K_13, 
            pxCubic, 
            SECP256K1_BASE_FIELD_ORDER), 
          mulmod(
            K_12, 
            pxSquare, 
            SECP256K1_BASE_FIELD_ORDER), 
          SECP256K1_BASE_FIELD_ORDER),
        addmod(
          mulmod(
            K_11, 
            p_x, 
            SECP256K1_BASE_FIELD_ORDER), 
          K_10, 
          SECP256K1_BASE_FIELD_ORDER), 
        SECP256K1_BASE_FIELD_ORDER)
      
      // x_den = x'^2 + k_(2,1) * x' + k_(2,0)
      let x_den := addmod(
        addmod(
          pxSquare, 
          mulmod(
            K_21, 
            p_x, 
            SECP256K1_BASE_FIELD_ORDER), 
          SECP256K1_BASE_FIELD_ORDER), 
        K_20, 
        SECP256K1_BASE_FIELD_ORDER)

      // Invert x_den using expmod precompile
      let payload := mload(0x40)
      mstore(payload, 0x20)             // Length of Base
      mstore(add(payload, 0x20), 0x20)  // Length of Exponent
      mstore(add(payload, 0x40), 0x20)  // Length of Modulus
      mstore(add(payload, 0x60), x_den)  // Base
      mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
      mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) {
        revert(0, 0)
      }

      x_den := mload(payload)
    
      // x = x_num / x_den
      r_x := mulmod(x_num, x_den, SECP256K1_BASE_FIELD_ORDER)

      // y_num = k_(3,3) * x'^3 + k_(3,2) * x'^2 + k_(3,1) * x' + k_(3,0)
      let y_num := addmod(
        addmod(
          mulmod(
            K_33, 
            pxCubic, 
            SECP256K1_BASE_FIELD_ORDER), 
          mulmod(
            K_32, 
            pxSquare, 
            SECP256K1_BASE_FIELD_ORDER), 
          SECP256K1_BASE_FIELD_ORDER),
        addmod(
          mulmod(
            K_31, 
            p_x, 
            SECP256K1_BASE_FIELD_ORDER), 
          K_30, 
          SECP256K1_BASE_FIELD_ORDER), 
        SECP256K1_BASE_FIELD_ORDER)

      // y_den = x'^3 + k_(4,2) * x'^2 + k_(4,1) * x' + k_(4,0)
      let y_den := addmod(
        addmod(
          pxCubic, 
          mulmod(
            K_42, 
            pxSquare, 
            SECP256K1_BASE_FIELD_ORDER), 
          SECP256K1_BASE_FIELD_ORDER), 
        addmod(
          mulmod(
            K_41, 
            p_x, 
            SECP256K1_BASE_FIELD_ORDER), 
          K_40, 
          SECP256K1_BASE_FIELD_ORDER), 
        SECP256K1_BASE_FIELD_ORDER)
    
      // Invert y_den using expmod precompile
      payload := mload(0x40)
      mstore(payload, 0x20)             // Length of Base
      mstore(add(payload, 0x20), 0x20)  // Length of Exponent
      mstore(add(payload, 0x40), 0x20)  // Length of Modulus
      mstore(add(payload, 0x60), y_den)  // Base
      mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p - 1
      mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // p
      if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) { // 0x05 == expmod precompile
        revert(0, 0)
      }

      y_den := mload(payload)
      
      // y = y' * y_num / y_den
      r_y :=
        mulmod(
          mulmod(
            p_y, 
            y_num, 
            SECP256K1_BASE_FIELD_ORDER), 
          y_den, 
        SECP256K1_BASE_FIELD_ORDER)
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
   */
  function map_to_curve_simple_swu(uint256 u) public view returns (uint256 r_x, uint256 r_y) {
    assembly {
      let tv1 := mulmod(u, u, SECP256K1_BASE_FIELD_ORDER) // 1.  tv1 = u^2
      tv1 := mulmod(Z, tv1, SECP256K1_BASE_FIELD_ORDER) // 2.  tv1 = Z * tv1
      let tv2 := mulmod(tv1, tv1, SECP256K1_BASE_FIELD_ORDER) // 3.  tv2 = tv1^2
      tv2 := addmod(tv2,tv1, SECP256K1_BASE_FIELD_ORDER) // 4.  tv2 = tv2 + tv1
      let tv3 := addmod(tv2, 1, SECP256K1_BASE_FIELD_ORDER) // 5.  tv3 = tv2 + 1
      tv3 := mulmod(tv3, B_Prime, SECP256K1_BASE_FIELD_ORDER) // 6.  tv3 = B * tv3
      
      let tv4
      switch eq(tv2, 0) // 7.  tv4 = CMOV(Z, -tv2, tv2 != 0)
      case true {
        tv4 := Z
      }
      case false {
        tv4 := sub(SECP256K1_BASE_FIELD_ORDER, tv2)
      }


      tv4 := mulmod(A_Prime, tv4, SECP256K1_BASE_FIELD_ORDER) // 8.  tv4 = A * tv4
      tv2 := mulmod(tv3, tv3, SECP256K1_BASE_FIELD_ORDER) // 9.  tv2 = tv3^2
      let tv6 := mulmod(tv4, tv4, SECP256K1_BASE_FIELD_ORDER) // 10. tv6 = tv4^2
      let tv5 := mulmod(A_Prime, tv6, SECP256K1_BASE_FIELD_ORDER) // 11. tv5 = A * tv6
      tv2 := addmod(tv2, tv5, SECP256K1_BASE_FIELD_ORDER) // 12. tv2 = tv2 + tv5
      tv2 := mulmod(tv2, tv3, SECP256K1_BASE_FIELD_ORDER) // 13. tv2 = tv2 * tv3
      tv6 := mulmod(tv6, tv4, SECP256K1_BASE_FIELD_ORDER) // 14. tv6 = tv6 * tv4
      tv5 := mulmod(B_Prime, tv6, SECP256K1_BASE_FIELD_ORDER) // 15. tv5 = B * tv6
      tv2 := addmod(tv2, tv5, SECP256K1_BASE_FIELD_ORDER) // 16. tv2 = tv2 + tv5
      r_x := mulmod(tv1, tv3, SECP256K1_BASE_FIELD_ORDER) // 17.   x = tv1 * tv3

      // 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)
      let y1
      let is_square

      // sqrt_ratio_3mod4(u,v) subroutine
      // tv1 -> tv7
      // tv2 -> tv8
      // u -> tv2
      // v -> tv6
      //
      // Algorithm from:
      // https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-optimized-sqrt_ratio-for-q-
      // ===================================
      let tv7 := mulmod(tv6,tv6, SECP256K1_BASE_FIELD_ORDER) // 1. tv1 = v^2
      let tv8 := mulmod(tv2,tv6, SECP256K1_BASE_FIELD_ORDER) // 2. tv2 = u * v

      tv7 := mulmod(tv7, tv8, SECP256K1_BASE_FIELD_ORDER) // 3. tv1 = tv1 * tv2
            
      // 4. y1 = tv1^c1 (using expmod precompile)
      let p := mload(0x40)
      mstore(p, 0x20)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), tv7)   // Base
      mstore(add(p, 0x80), C_1)   // Exponent
      mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) { // 0x05 == expmod precompile
        revert(0, 0)
      }
      
      let y1_inner := mulmod(mload(p), tv8, SECP256K1_BASE_FIELD_ORDER) // 5. y1 = y1 * tv2
      let y2_inner := mulmod(y1_inner, C_2, SECP256K1_BASE_FIELD_ORDER) // 6. y2 = y1 * c2
      let tv9 := mulmod(y1_inner, y1_inner, SECP256K1_BASE_FIELD_ORDER) // 7. tv3 = y1^2
      tv9 := mulmod(tv9, tv6, SECP256K1_BASE_FIELD_ORDER) // 8. tv3 = tv3 * v

      switch eq(tv9, tv2) // 9. isQR = tv3 == u
      case true { // 10. y = CMOV(y2, y1, isQR)
        is_square := true
        y1 := y1_inner
      }
      case false {
        is_square := false
        y1 := y2_inner
      }

      // =====================================

      r_y := mulmod(tv1, u, SECP256K1_BASE_FIELD_ORDER) // 19.   y = tv1 * u
      r_y := mulmod(r_y, y1, SECP256K1_BASE_FIELD_ORDER) // 20.   y = y * y1

      if is_square {
        r_x := tv3 // 21.   x = CMOV(x, tv3, is_gx1_square)
        r_y := y1 // 22.   y = CMOV(y, y1, is_gx1_square)
      }

      // 23.  e1 = sgn0(u) == sgn0(y)
      if iszero(eq(mod(u, 2), mod(r_y, 2))) { // sgn0(x) ~= x % 2
        r_y := sub(SECP256K1_BASE_FIELD_ORDER, r_y) // 24.   y = CMOV(-y, y, e1)
      }

      // compute tv4 ^ -1
      let payload := mload(0x40)
      mstore(payload, 0x20)             // Length of Base
      mstore(add(payload, 0x20), 0x20)  // Length of Exponent
      mstore(add(payload, 0x40), 0x20)  // Length of Modulus
      mstore(add(payload, 0x60), tv4)  // Base
      mstore(add(payload, 0x80), 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2D) // p-2
      mstore(add(payload, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // p
      if iszero(staticcall(not(0), 0x05, payload, 0xC0, payload, 0x20)) { // 0x05 == expmod precompile
        revert(0, 0)
      }

      r_x := mulmod(r_x, mload(payload), SECP256K1_BASE_FIELD_ORDER) // 25.   x = x / tv4
    }
  }

  /**
   * Square root subroutine used by simplified SWU mapping.
   *
   * Returns (true, sqrt(u / v)) if (u / v) is square in the field
   * Returns (false, sqrt(Z * (u / v))) otherwise
   *
   * Algorithm from:
   * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-optimized-sqrt_ratio-for-q-
   *
   * Optimizations:
   * - mathematical optimization: reduce expmod / mulmod / addmod operations
   * - few temporary values to reduce memory expansion
   * - Solidity assembly
   */
  function sqrt_ratio_3mod4(uint256 u, uint256 v) public view returns (uint256 y, bool isSquare) {
    assembly {
      let tv1 := mulmod(v,v, SECP256K1_BASE_FIELD_ORDER) // 1. tv1 = v^2
      let tv2 := mulmod(u,v, SECP256K1_BASE_FIELD_ORDER) // 2. tv2 = u * v

      tv1 := mulmod(tv1, tv2, SECP256K1_BASE_FIELD_ORDER) // 3. tv1 = tv1 * tv2
            
      // 4. y1 = tv1^c1
      let p := mload(0x40)
      mstore(p, 0x20)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), tv1)  // Base
      mstore(add(p, 0x80), C_1)    // Exponent
      mstore(add(p, 0xa0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xC0, p, 0x20)) { // 0x05 == expmod precompile
        revert(0, 0)
      }
      
      let y1 := mulmod(mload(p), tv2, SECP256K1_BASE_FIELD_ORDER) // 5. y1 = y1 * tv2
      let y2 := mulmod(y1, C_2, SECP256K1_BASE_FIELD_ORDER) // 6. y2 = y1 * c2
      let tv3 := mulmod(y1, y1, SECP256K1_BASE_FIELD_ORDER) // 7. tv3 = y1^2
      tv3 := mulmod(tv3, v, SECP256K1_BASE_FIELD_ORDER) // 8. tv3 = tv3 * v

      switch eq(tv3, u) // 9. isQR = tv3 == u
      case true { // 10. y = CMOV(y2, y1, isQR)
        isSquare := true
        y := y1
      }
      case false {
        isSquare := false
        y := y2
      }
    }
  }

  /**
   * Takes an arbitrary byte-string and a domain seperation tag (DST) and returns
   * two elements of the field used to create the secp256k1 curve.
   */
  function hash_to_field(bytes memory message, bytes memory DST) public view returns (uint256 u_0, uint256 u_1) {
    (bytes32 b_1, bytes32 b_2, bytes32 b_3) = expand_message_xmd_keccak256(message, DST);

    // computes [...b_1[..], ...b_2[0..16]] ^ 1 mod n
    assembly {
      let p := mload(0x40) // next free memory slot
      mstore(p, 0x30)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), b_1)  // Base
      mstore(add(p, 0x80), b_2)
      mstore(add(p, 0x90), 1)    // Exponent
      mstore(add(p, 0xb0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) {
        revert(0, 0)
      }

      u_0 := mload(p)
    }

    // computes [...b_2[16..32], ...b_3[..]] ^ 1 mod n
    assembly {
      let p := mload(0x40)
      mstore(p, 0x30)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x50), b_2)
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x70), b_3)  // Base
      mstore(add(p, 0x90), 1)    // Exponent
      mstore(add(p, 0xb0), SECP256K1_BASE_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) {
        revert(0, 0)
      }

      u_1 := mload(p)
    }
  }

    /**
   * Takes an arbitrary byte-string and a domain seperation tag (DST) and returns
   * two elements of the field used to create the secp256k1 curve.
   */
  function hash_to_field_single(bytes memory message, bytes memory DST) public view returns (uint256 u) {
    (bytes32 b_1, bytes32 b_2) = expand_message_xmd_keccak256_single(message, DST);

    // computes [...b_1[0..32], ...b_2[0..16]] ^ 1 mod n
    assembly {
      let p := mload(0x40) // next free memory slot
      mstore(p, 0x30)             // Length of Base
      mstore(add(p, 0x20), 0x20)  // Length of Exponent
      mstore(add(p, 0x40), 0x20)  // Length of Modulus
      mstore(add(p, 0x60), b_1)  // Base
      mstore(add(p, 0x80), b_2)
      mstore(add(p, 0x90), 1)    // Exponent
      mstore(add(p, 0xb0), SECP256K1_FIELD_ORDER)     // Modulus
      if iszero(staticcall(not(0), 0x05, p, 0xD0, p, 0x20)) {
        revert(0, 0)
      }

      u := mload(p)
    }
  }

  /**
   * Expands an arbitrary byte-string to 96 bits using the `expand_message_xmd` method described in
   * https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html
   *
   * Note: This is not a general implementation as the output length fixed. It is tailor-made
   *       for secp256k1_XMD:KECCAK_256_SSWU_RO_ hash_to_curve implementation.
   */
  function expand_message_xmd_keccak256(bytes memory message, bytes memory DST) public pure returns (bytes32 b_1, bytes32 b_2, bytes32 b_3) {
    uint256 ell; 

    if ((message.length >> 5) << 5 == 0) {
      ell = message.length >> 5;
    } else {
      ell = message.length >> 5 + 1;
    }

    if (ell > 255 || message.length > 2040 || DST.length > 255) {
      revert MessageTooLong();
    }

    assembly {
      let b_0
      {
        // create payload for b_0 hash
        let b_0_payload := mload(0x40)

        // payload[0..KECCAK256_BLOCKSIZE] = 0

        let b_0_payload_o := KECCAK256_BLOCKSIZE // leave first block empty
        let msg_o := 0x20 // skip length prefix

        // payload[KECCAK256_BLOCKSIZE..KECCAK256_BLOCKSIZE+message.len()] = message[0..message.len()]
        for { let i := 0 } lt(i, mload(message)) { i := add(i, 0x20) } {
          mstore(add(b_0_payload, b_0_payload_o), mload(add(message, msg_o)))
          b_0_payload_o := add(b_0_payload_o, 0x20)
          msg_o := add(msg_o, 0x20)
        }

        // payload[KECCAK256_BLOCKSIZE+message.len()+1..KECCAK256_BLOCKSIZE+message.len()+2] = 96
        b_0_payload_o := add(mload(message), 137)
        mstore8(add(b_0_payload, b_0_payload_o), 0x60) // only support for 96 bytes output length

        let dst_o := 0x20
        b_0_payload_o := add(b_0_payload_o, 2)

        // // payload[KECCAK256_BLOCKSIZE+message.len()+3..KECCAK256_BLOCKSIZE+message.len()+DST.len()] = DST[0..DST.len()]
        for { let i := 0 } lt(i, mload(DST)) { i := add(i, 0x20) } {
          mstore(add(b_0_payload, b_0_payload_o), mload(add(DST, dst_o)))
          b_0_payload_o := add(b_0_payload_o, 0x20)
          dst_o := add(dst_o, 0x20)
        }

        // // payload[KECCAK256_BLOCKSIZE+message.len()+DST.len()..KECCAK256_BLOCKSIZE+message.len()+DST.len()+1] = DST.len()
        b_0_payload_o := add(add(mload(message), mload(DST)), 139)
        mstore8(add(b_0_payload, b_0_payload_o), mload(DST))

        b_0 := keccak256(b_0_payload, add(140, add(mload(DST), mload(message))))
      }

      // create payload for b_1, b_2 ... hashes
      let b_i_payload := mload(0x40)
      mstore(b_i_payload, b_0)
      // payload[32..33] = 1
      mstore8(add(b_i_payload,0x20), 1)

      let payload_o := 0x21
      let dst_o := 0x20

      // payload[33..33+DST.len()] = DST[0..DST.len()] 
      for { let i := 0 } lt(i, mload(DST)) { i := add(i, 0x20) } {
        mstore(add(b_i_payload, payload_o), mload(add(DST, dst_o)))
        payload_o := add(payload_o, 0x20)
        dst_o := add(dst_o, 0x20)
      }

      // payload[65+DST.len()..66+DST.len()] = DST.len()
      mstore8(add(b_i_payload, add(0x21, mload(DST))), mload(DST))

      b_1 := keccak256(b_i_payload, add(34, mload(DST)))
      
      // payload[0..32] = b_0 XOR b_1
      mstore(b_i_payload, xor(b_0, b_1))
      // payload[32..33] = 2
      mstore8(add(b_i_payload, 0x20), 2)

      b_2 := keccak256(b_i_payload, add(34, mload(DST)))

      // payload[0..32] = b_0 XOR b_2
      mstore(b_i_payload, xor(b_0, b_2))
      // payload[32..33] = 2
      mstore8(add(b_i_payload, 0x20), 3)

      b_3 := keccak256(b_i_payload, add(34, mload(DST)))
    }
  }

  function expand_message_xmd_keccak256_single(bytes memory message, bytes memory DST) public pure returns (bytes32 b_1, bytes32 b_2) {
    uint256 ell; 

    if ((message.length >> 5) << 5 == 0) {
      ell = message.length >> 5;
    } else {
      ell = message.length >> 5 + 1;
    }

    if (ell > 255 || message.length > 2040 || DST.length > 255) {
      revert MessageTooLong();
    }

    assembly {
      let b_0
      {
        // create payload for b_0 hash
        let b_0_payload := mload(0x40)

        // payload[0..KECCAK256_BLOCKSIZE] = 0

        let b_0_payload_o := KECCAK256_BLOCKSIZE // leave first block empty
        let msg_o := 0x20 // skip length prefix

        // payload[KECCAK256_BLOCKSIZE..KECCAK256_BLOCKSIZE+message.len()] = message[0..message.len()]
        for { let i := 0 } lt(i, mload(message)) { i := add(i, 0x20) } {
          mstore(add(b_0_payload, b_0_payload_o), mload(add(message, msg_o)))
          b_0_payload_o := add(b_0_payload_o, 0x20)
          msg_o := add(msg_o, 0x20)
        }

        // payload[KECCAK256_BLOCKSIZE+message.len()+1..KECCAK256_BLOCKSIZE+message.len()+2] = 48
        b_0_payload_o := add(mload(message), 137)
        mstore8(add(b_0_payload, b_0_payload_o), 0x30) // only support for 48 bytes output length

        let dst_o := 0x20
        b_0_payload_o := add(b_0_payload_o, 2)

        // // payload[KECCAK256_BLOCKSIZE+message.len()+3..KECCAK256_BLOCKSIZE+message.len()+DST.len()] = DST[0..DST.len()]
        for { let i := 0 } lt(i, mload(DST)) { i := add(i, 0x20) } {
          mstore(add(b_0_payload, b_0_payload_o), mload(add(DST, dst_o)))
          b_0_payload_o := add(b_0_payload_o, 0x20)
          dst_o := add(dst_o, 0x20)
        }

        // // payload[KECCAK256_BLOCKSIZE+message.len()+DST.len()..KECCAK256_BLOCKSIZE+message.len()+DST.len()+1] = DST.len()
        b_0_payload_o := add(add(mload(message), mload(DST)), 139)
        mstore8(add(b_0_payload, b_0_payload_o), mload(DST))

        b_0 := keccak256(b_0_payload, add(140, add(mload(DST), mload(message))))
      }

      // create payload for b_1, b_2 ... hashes
      let b_i_payload := mload(0x40)
      mstore(b_i_payload, b_0)
      // payload[32..33] = 1
      mstore8(add(b_i_payload,0x20), 1)

      let payload_o := 0x21
      let dst_o := 0x20

      // payload[33..33+DST.len()] = DST[0..DST.len()] 
      for { let i := 0 } lt(i, mload(DST)) { i := add(i, 0x20) } {
        mstore(add(b_i_payload, payload_o), mload(add(DST, dst_o)))
        payload_o := add(payload_o, 0x20)
        dst_o := add(dst_o, 0x20)
      }

      // payload[65+DST.len()..66+DST.len()] = DST.len()
      mstore8(add(b_i_payload, add(0x21, mload(DST))), mload(DST))

      b_1 := keccak256(b_i_payload, add(34, mload(DST)))
      
      // payload[0..32] = b_0 XOR b_1
      mstore(b_i_payload, xor(b_0, b_1))
      // payload[32..33] = 2
      mstore8(add(b_i_payload, 0x20), 2)

      b_2 := keccak256(b_i_payload, add(34, mload(DST)))
    }
  }


  struct VRF_Parameters {
    address signer;
    // we only support 32-byte messages
    uint256 message;
    uint256 v_x;
    uint256 v_y;
    uint256 s;
    uint256 h;
    uint256 sB_x;
    uint256 sB_y;
    uint256 hV_x;
    uint256 hV_y;
  }

  /**
   * Implements a VRF based on public-key cryptography using hash_to_curve primitive.
   *
   * Algorithm highly inspired by:
   * https://www.signal.org/docs/specifications/xeddsa/#vxeddsa
   */
  function vrf_verify(VRF_Parameters memory params) public view returns (bool) {
    if (params.h >= SECP256K1_FIELD_ORDER) {
      revert InvalidFieldElement();
    }

    bytes memory DST = "some DST tag";

    // we get a pseudo-random curve point
    (uint256 B_x, uint256 B_y) = hashToCurve(abi.encodePacked(params.signer, params.message), DST);

    // Mitigate missing ECMUL operation by using precomputed values and verify
    // against computed Ethereum address.
    address maybe_sBv = scalarPointMultiplication(params.s, B_x, B_y);

    if (maybe_sBv != pointToAddress(params.sB_x, params.sB_y)) {
      revert InvalidPointWitness();
    }

    address maybe_hV = scalarPointMultiplication(params.h, params.v_x, params.v_y);

    if (maybe_hV != pointToAddress(params.hV_x, params.hV_y)) {
      revert InvalidPointWitness();
    }

    // We checked the validity of precomputed sB and hV values,
    // now use them as if they were intermediate results. 

    // R = sB - hV
    (uint256 r_x, uint256 r_y) = ecAdd(params.sB_x, params.sB_y, params.hV_x, SECP256K1_BASE_FIELD_ORDER - params.hV_y, 0);

    uint256 h_check = hash_to_field_single(abi.encodePacked(params.signer, params.v_x, params.v_y, r_x, r_y, params.message), DST);

    return h_check == params.h;
  }
}
