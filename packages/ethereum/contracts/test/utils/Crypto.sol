// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;

import '../../src/Crypto.sol';
import 'solcrypto/SECP2561k.sol';

import 'forge-std/console2.sol';

abstract contract CryptoUtils is HoprCrypto, SECP2561k {
  uint256 constant SECP256K1_HALF_FIELD_ORDER = 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0;

  function toCompactSignature(uint8 v, bytes32 r, bytes32 s) pure internal returns (HoprCrypto.CompactSignature memory sig) {
    if (uint256(s) >= SECP256K1_HALF_FIELD_ORDER) {
      s = bytes32(HoprCrypto.SECP256K1_FIELD_ORDER - uint256(s));
    }
    sig.r = r;
    sig.vs = bytes32(uint256(v - 27) << 255) | s;
  }

  function getVRFParameters(uint256 privKey, bytes memory DST, bytes32 vrfMessage) view internal returns (HoprCrypto.VRF_Parameters memory params) {
    HoprCrypto.VRF_Payload memory payload;

    // stack height optimization, doesn't compile otherwise
    {
      address chain_addr = HoprCrypto.scalarTimesBasepoint(privKey);
      payload.message = vrfMessage;
      payload.signer = chain_addr;
      payload.DST = abi.encodePacked(DST);
    }

    // stack height optimization, doesn't compile otherwise
    {
      (uint256 b_x, uint256 b_y) = HoprCrypto.hashToCurve(abi.encodePacked(payload.signer, payload.message), payload.DST);

      {
        (uint256 v_x, uint256 v_y) = SECP2561k.ecmul(b_x, b_y, privKey);
        params.v_x = v_x;
        params.v_y = v_y;
      }

      uint256 r = HoprCrypto.hash_to_scalar(abi.encodePacked(privKey, b_x, b_y, payload.message), payload.DST);

      (uint256 b_r_x, uint256 b_r_y) = SECP2561k.ecmul(b_x,b_y, r);

      params.h = HoprCrypto.hash_to_scalar(abi.encodePacked(
        payload.signer,
        params.v_x,
        params.v_y,
        b_r_x,
        b_r_y,
        payload.message
      ), abi.encodePacked(DST));

      // s = r + h * a (mod p)
      params.s = addmod(r, mulmod(params.h, privKey, HoprCrypto.SECP256K1_FIELD_ORDER), HoprCrypto.SECP256K1_FIELD_ORDER);

      {
        (uint256 s_b_x, uint256 s_b_y) = SECP2561k.ecmul(b_x, b_y, params.s);
        params.sB_x = s_b_x;
        params.sB_y = s_b_y;

        (uint256 h_v_x, uint256 h_v_y) = SECP2561k.ecmul(params.v_x, params.v_y, params.h);
        params.hV_x = h_v_x;
        params.hV_y = h_v_y;
      }
    }
  }
}