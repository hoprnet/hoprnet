// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import './utils/Accounts.sol';

import '../src/Crypto.sol';

contract Crypto is Test, AccountsFixtureTest, HoprCrypto {
    HoprCrypto crypto;

  function setUp() public {
    crypto = new HoprCrypto();
  }

  function testInvMod(uint256 el) public {
    // uint256 el = 2;
    el = bound(el, 2 , uint256(HoprCrypto.SECP256K1_BASE_FIELD_ORDER) - 1);

    bytes32 invEl = crypto.invMod(bytes32(el));

    assertEq(mulmod(el, uint256(invEl), uint256(HoprCrypto.SECP256K1_BASE_FIELD_ORDER)), 1);
  }
}