// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import './utils/Accounts.sol';

import '../src/Crypto.sol';

contract Crypto is Test, AccountsFixtureTest, HoprCrypto {
  function modifierIsCurvePoint(CurvePoint calldata p) isCurvePoint(p) public {}

  function setUp() public {
  }

  function testInvMod(uint256 el) public {
    el = bound(el, 2 , HoprCrypto.SECP256K1_BASE_FIELD_ORDER - 1);

    uint256 invEl = HoprCrypto.invMod(el);

    assertEq(mulmod(el, invEl, HoprCrypto.SECP256K1_BASE_FIELD_ORDER), 1);
  }

  function testPointToAddress() public {
    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(HoprCrypto.pointToAddress.selector, 0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75, 0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5));

    assertTrue(success);
    assertEq(address(uint160(uint256(bytes32(returnValue)))), accountA.accountAddr);
  }

  function testIsCurvePoint() public {
    // .selector is not yet supported here
    bytes4 isCurvePointSelector = bytes4(0x9a82d40c);
    
    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(isCurvePointSelector, 0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75, 0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5));

    assertEq(bytes32(0), bytes32(returnValue));
    assertTrue(success);
  }

  function testScalarTimeBasepoint() public {
    assertEq(HoprCrypto.scalarTimesBasepoint(accountA.privateKey), accountA.accountAddr);
  }

  function testEcAdd() public {
    uint256 x1 = 0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75;
    uint256 x2 = 0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5;
    uint256 y1 = 0xba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0;
    uint256 y2 = 0xd67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4;

    uint256 r1 = 0x551c7c46a964dec7edd8a5cedc557ebce43cc3f70ff481bdcfbd4e86d435c2ba;
    uint256 r2 = 0x16883c2c7e2527800aa21a8420f8af48eafb2594d00e0f7e9e7d11a938b9a168;

    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(HoprCrypto.ecAdd.selector, x1, x2, y1, y2));

    HoprCrypto.CurvePoint memory p = abi.decode(returnValue, (HoprCrypto.CurvePoint));
    assertTrue(success);
    assertEq(r1, p.x);
    assertEq(r2, p.y);
  }

  function testSqrt() public {
    uint256 x = 11;
    
    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(HoprCrypto.sqrtMod.selector, x));
    uint256 result = abi.decode(returnValue, (uint256));

    assertTrue(success);
    assertEq(mulmod(result, result, SECP256K1_BASE_FIELD_ORDER), x);
    console.log(result);

    console.logBytes32(bytes32((SECP256K1_BASE_FIELD_ORDER - 3) / 4));
  }

  function testIsSquare() public {
    uint256 SQUARE = 11;
    uint256 NON_SQUARE = SECP256K1_BASE_FIELD_ORDER - 11;

    assertTrue(HoprCrypto.isSquare(SQUARE));
    assertFalse(HoprCrypto.isSquare(NON_SQUARE));
  }

  // function testMapPoint() public {

  // }

  function testMapToCurve() public {
    uint256 u_0 = 0x6b0f9910dd2ba71c78f2ee9f04d73b5f4c5f7fc773a701abea1e573cab002fb3;

    CurvePoint memory r = map_to_curve_simple_swu(u_0);
    CurvePoint memory mapped = mapPoint(r);
    // console.logBytes32(bytes32(SECP256K1_BASE_FIELD_ORDER-B_Prime / A_Prime));
    console.logBytes32(bytes32(mapped.x));
  }
}