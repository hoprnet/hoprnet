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
    el = bound(el, 2 , uint256(HoprCrypto.SECP256K1_BASE_FIELD_ORDER) - 1);

    bytes32 invEl = HoprCrypto.invMod(bytes32(el));

    assertEq(mulmod(el, uint256(invEl), uint256(HoprCrypto.SECP256K1_BASE_FIELD_ORDER)), 1);
  }

  function testPointToAddress() public {
    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(HoprCrypto.pointToAddress.selector, bytes32(0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75), bytes32(0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5)));

    assertTrue(success);
    assertEq(address(uint160(uint256(bytes32(returnValue)))), accountA.accountAddr);
  }

  function testIsCurvePoint() public {
    // .selector is not yet supported here
    bytes4 isCurvePointSelector = bytes4(0x9a82d40c);
    
    (bool success, bytes memory returnValue) = address(this).staticcall(abi.encodeWithSelector(isCurvePointSelector, bytes32(0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75), bytes32(0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5)));

    assertEq(bytes32(0), bytes32(returnValue));
    assertTrue(success);
  }

  function testScalarTimeBasepoint() public {
    assertEq(HoprCrypto.scalarTimesBasepoint(bytes32(accountA.privateKey)), accountA.accountAddr);
  }
}