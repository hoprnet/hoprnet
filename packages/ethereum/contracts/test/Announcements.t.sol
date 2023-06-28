// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import './utils/Accounts.sol';
import 'forge-std/Test.sol';
import '../src/Announcements.sol';

// Dummy since there is no verification happening on-chain
bytes32 constant ed25519_sig_0 = 0x000000000000000000000000000000000000000000000000000000000ed25519;
bytes32 constant ed25519_sig_1 = 0x100000000000000000000000000000000000000000000000000000000ed25519;

bytes4 constant ipv4 = 0x10000001; // 10.0.0.1
bytes16 constant ipv6 = 0x20010db8000000000000ff0000428329; // 2001:0db8:0000:0000:0000:ff00:0042:8329
bytes2 constant port = 0xffff; // port 65535

bytes32 constant ed25519_pub_key = 0x3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c;

contract AccountTest is Test, AccountsFixtureTest {
  HoprAnnouncements public announcements;

  event KeyBinding(bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key, address chain_key);

  function setUp() public {
    announcements = new HoprAnnouncements();
  }

  function testKeyBinding() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, accountA.accountAddr);

    vm.prank(accountA.accountAddr);
    announcements.bindKeys(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);
  }

  event AddressAnnouncement4(address node, bytes4 ip4, bytes2 port);
  event AddressAnnouncement6(address node, bytes16 ip6, bytes2 port);

  function testIpPortAnnouncements() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(accountA.accountAddr, ipv4, port);

    vm.prank(accountA.accountAddr);
    announcements.announce4(ipv4, port);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(accountA.accountAddr, ipv6, port);

    vm.prank(accountA.accountAddr);
    announcements.announce6(ipv6, port);
  }

  event RevokeAnnouncement(address node);

  function testAddressRevocation() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit RevokeAnnouncement(accountA.accountAddr);

    vm.prank(accountA.accountAddr);
    announcements.revoke();
  }

  function testAllInOneAnnouncement() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, accountA.accountAddr);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(accountA.accountAddr, ipv4, port);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(accountA.accountAddr, ipv6, port);

    bytes[] memory calls = new bytes[](3);

    calls[0] = abi.encodeCall(
      announcements.bindKeys,
      (ed25519_sig_0, ed25519_sig_1, ed25519_pub_key)
    );

    calls[1] = abi.encodeCall(announcements.announce4, (ipv4, port));

    calls[2] = abi.encodeCall(announcements.announce6, (ipv6, port));

    vm.prank(accountA.accountAddr);
    announcements.multicall(calls);
  }

  function testBindKeyAnnounce4() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, accountA.accountAddr);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(accountA.accountAddr, ipv4, port);

    vm.prank(accountB.accountAddr);
    announcements.bindKeysAnnounce4(
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv4,
      port
    );
  }

  function testBindKeyAnnounce6() public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, accountA.accountAddr);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(accountA.accountAddr, ipv6, port);

    vm.prank(accountA.accountAddr);
    announcements.bindKeysAnnounce6(
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv6,
      port
    );
  }
}
