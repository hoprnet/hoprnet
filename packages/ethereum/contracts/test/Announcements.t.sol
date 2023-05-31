// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import './utils/Accounts.sol';
import 'forge-std/Test.sol';
import '../src/Announcements.sol';

address constant odd_addr = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
bytes32 constant odd_secp256k1_x = 0x8318535b54105d4a7aae60c08fc45f9687181b4fdfc625bd1a753fa7397fed75;
bytes32 constant odd_secp256k1_y = 0x3547f11ca8696646f2f3acb08e31016afac23e630c5d11f59f61fef57b0d2aa5;

address constant even_addr = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;
bytes32 constant even_secp256k1_x = 0xba5734d8f7091719471e7f7ed6b9df170dc70cc661ca05e688601ad984f068b0;
bytes32 constant even_secp256k1_y = 0xd67351e5f06073092499336ab0839ef8a521afd334e53807205fa2f08eec74f4;
// Dummy since there is no verification happening on-chain
bytes32 constant ed25519_sig_0 = 0x0000000000000000000000000000000000000000000000000000000000025519;
bytes32 constant ed25519_sig_1 = 0x1000000000000000000000000000000000000000000000000000000000025519;

bytes4 constant ipv4 = 0x10000001; // 10.0.0.1
bytes16 constant ipv6 = 0x20010db8000000000000ff0000428329; // 2001:0db8:0000:0000:0000:ff00:0042:8329
bytes2 constant port = 0xffff; // port 65535

bytes32 constant ed25519_pub_key = 0x3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c;

contract AccountTest is Test {
  HoprAnnouncements public announcements;

  event KeyBindingOdd(bytes32 secp256k1_x, bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key);
  event KeyBindingEven(bytes32 secp256k1_x, bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key);

  function setUp() public {
    announcements = new HoprAnnouncements();
  }

  function testKeyBinding() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBindingOdd(odd_secp256k1_x, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

    vm.prank(odd_addr);
    announcements.bindKeys(odd_secp256k1_x, odd_secp256k1_y, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBindingEven(even_secp256k1_x, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

    vm.prank(even_addr);
    announcements.bindKeys(even_secp256k1_x, even_secp256k1_y, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
  }

  function testKeyBindingError() public {
    vm.expectRevert(
      abi.encodeWithSelector(HoprAnnouncements.PublicKeyDoesNotMatchSender.selector, odd_addr, even_addr)
    );
    vm.prank(even_addr);
    announcements.bindKeys(odd_secp256k1_x, odd_secp256k1_y, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
  }

  event AddressAnnouncement4(address node, bytes4 ip4, bytes2 port);
  event AddressAnnouncement6(address node, bytes16 ip6, bytes2 port);

  function testIpPortAnnouncements() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(vm.addr(1), ipv4, port);

    vm.prank(vm.addr(1));
    announcements.announce4(ipv4, port);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(vm.addr(1), ipv6, port);

    vm.prank(vm.addr(1));
    announcements.announce6(ipv6, port);
  }

  event RevokeAnnouncement(address node);

  function testAddressRevocation() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit RevokeAnnouncement(vm.addr(1));

    vm.prank(vm.addr(1));
    announcements.revoke();
  }

  function testAllInOneAnnouncement() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBindingOdd(odd_secp256k1_x, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(odd_addr, ipv4, port);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(odd_addr, ipv6, port);

    bytes[] memory calls = new bytes[](3);

    calls[0] = abi.encodeCall(
      announcements.bindKeys,
      (odd_secp256k1_x, odd_secp256k1_y, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key)
    );

    calls[1] = abi.encodeCall(announcements.announce4, (ipv4, port));

    calls[2] = abi.encodeCall(announcements.announce6, (ipv6, port));

    vm.prank(odd_addr);
    announcements.multicall(calls);
  }

  function testBindKeyAnnounce4() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBindingOdd(odd_secp256k1_x, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(odd_addr, ipv4, port);

    vm.prank(odd_addr);
    announcements.bindKeysAnnounce4(
      odd_secp256k1_x,
      odd_secp256k1_y,
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv4,
      port
    );

    vm.expectRevert();
    vm.prank(even_addr);
    announcements.bindKeysAnnounce4(
      odd_secp256k1_x,
      odd_secp256k1_y,
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv4,
      port
    );
  }

  function testBindKeyAnnounce6() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBindingOdd(odd_secp256k1_x, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(odd_addr, ipv6, port);

    vm.prank(odd_addr);
    announcements.bindKeysAnnounce6(
      odd_secp256k1_x,
      odd_secp256k1_y,
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv6,
      port
    );

    vm.expectRevert();
    vm.prank(even_addr);
    announcements.bindKeysAnnounce6(
      odd_secp256k1_x,
      odd_secp256k1_y,
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv4,
      port
    );
  }
}
