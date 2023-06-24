// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.19;

error InvalidFieldElement();
error InvalidCurvePoint();

contract HoprCrypto {
  // secp256k1: y^2 = x^3 + ax + b (mod F_p)

  bytes32 constant SECP256K1_B = 0x0000000000000000000000000000000000000000000000000000000000000007;
  // Field order created by secp256k1 curve
  bytes32 constant SECP256K1_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

  // 
  bytes32 constant SECP256K1_BASE_FIELD_ORDER = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F;
  // x-component of base point of secp256k1 curve
  bytes32 constant SECP256K1_BASE_POINT_X_COMPONENT = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798;

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
    bytes32 x;
    bytes32 y;
  }

  /**
   * Checks whether given value is an element of the secp256k1 field
   */
  modifier isFieldElement(bytes32 el) {
    if (0 == uint256(el) || uint256(el) >= uint256(SECP256K1_FIELD_ORDER)) {
      revert InvalidFieldElement();
    }
    _;
  }

  /**
   * Checks whether given coordinates fulfill the secp256k1 curve equation
   */
  modifier isCurvePoint(CurvePoint calldata p)  {
    if ( 
      uint256(p.x) == 0 || uint256(p.x) >= uint256(SECP256K1_FIELD_ORDER) ||
      uint256(p.y) == 0 || uint256(p.y) >= uint256(SECP256K1_FIELD_ORDER) ||
      mulmod(uint256(p.x), uint256(p.x), uint256(SECP256K1_FIELD_ORDER)) != 
      (mulmod(mulmod(uint256(p.y), uint256(p.y), uint256(SECP256K1_FIELD_ORDER)), uint256(p.y), uint256(SECP256K1_FIELD_ORDER)) + 7) % uint256(SECP256K1_FIELD_ORDER)
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
  function scalarTimesBasepoint(bytes32 scalar) public pure returns (address) {
    return
      ecrecover(
        0,
        SECP256K1_BASE_POINT_Y_COMPONENT_SIGN,
        bytes32(SECP256K1_BASE_POINT_X_COMPONENT),
        bytes32(mulmod(uint256(scalar), uint256(SECP256K1_BASE_POINT_X_COMPONENT), uint256(SECP256K1_FIELD_ORDER)))
      );
  }

  /**
   * Computes multiplicative inverse of secp256k1 field element.
   *
   * Uses little Fermat because expmod precompile is more gas-efficient
   * than extended euclidian algorithm in Solidity.
   */
  function invMod(bytes32 el) public view returns (bytes32 o) {
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

  /**
   * Converts a curve point to an Ethereum address
   */
  function pointToAddress(CurvePoint calldata p) public pure returns (address) {
    return address(uint160(uint256(keccak256(abi.encodePacked(p.x, p.y)))));
  }
}