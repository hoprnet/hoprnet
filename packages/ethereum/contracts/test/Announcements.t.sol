// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import 'forge-std/Test.sol';
import '../src/Announcements.sol';
import '../src/node-stake/NodeSafeRegistry.sol';

// Dummy since there is no verification happening on-chain
bytes32 constant ed25519_sig_0 = 0x000000000000000000000000000000000000000000000000000000000ed25519;
bytes32 constant ed25519_sig_1 = 0x100000000000000000000000000000000000000000000000000000000ed25519;

bytes4 constant ipv4 = 0x10000001; // 10.0.0.1
bytes16 constant ipv6 = 0x20010db8000000000000ff0000428329; // 2001:0db8:0000:0000:0000:ff00:0042:8329
bytes2 constant port = 0xffff; // port 65535

bytes32 constant ed25519_pub_key = 0x3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c;

contract AccountTest is Test {
  HoprNodeSafeRegistry safeRegistry;
  HoprAnnouncements announcements;

  event KeyBinding(bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key, address chain_key);

  function setUp() public {
    safeRegistry = new HoprNodeSafeRegistry();
    announcements = new HoprAnnouncements(safeRegistry);
  }

  function testKeyBinding(address caller) public {
    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(0))
    );

    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, caller);

    vm.prank(caller);
    announcements.bindKeys(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

    vm.clearMockedCalls();
  }

  event AddressAnnouncement4(address node, bytes4 ip4, bytes2 port);
  event AddressAnnouncement6(address node, bytes16 ip6, bytes2 port);

  function testIpPortAnnouncements(address caller) public {
    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(0))
    );

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(caller, ipv4, port);

    vm.prank(caller);
    announcements.announce4(ipv4, port);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(caller, ipv6, port);

    vm.prank(caller);
    announcements.announce6(ipv6, port);

    vm.clearMockedCalls();
  }

  event RevokeAnnouncement(address node);

  function testAddressRevocation(address caller) public {
    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(0))
    );

    vm.expectEmit(true, false, false, false, address(announcements));
    emit RevokeAnnouncement(caller);

    vm.prank(caller);
    announcements.revoke();

    vm.clearMockedCalls();
  }

  function testAllInOneAnnouncement(address caller) public {
    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(0))
    );

    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, caller);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(caller, ipv4, port);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(caller, ipv6, port);

    bytes[] memory calls = new bytes[](3);

    calls[0] = abi.encodeCall(
      announcements.bindKeys,
      (ed25519_sig_0, ed25519_sig_1, ed25519_pub_key)
    );

    calls[1] = abi.encodeCall(announcements.announce4, (ipv4, port));

    calls[2] = abi.encodeCall(announcements.announce6, (ipv6, port));

    vm.prank(caller);
    announcements.multicall(calls);

    vm.clearMockedCalls();
  }

  function testBindKeyAnnounce4(address caller) public {
    vm.mockCall(
      address(safeRegistry),
      abi.encodeWithSignature('nodeToSafe(address)', caller),
      abi.encode(address(0))
    );

    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, caller);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement4(caller, ipv4, port);

    vm.prank(caller);
    announcements.bindKeysAnnounce4(
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv4,
      port
    );

    vm.clearMockedCalls();
  }

  function testBindKeyAnnounce6(address caller) public {
    vm.expectEmit(true, false, false, false, address(announcements));
    emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, caller);

    vm.expectEmit(true, false, false, false, address(announcements));
    emit AddressAnnouncement6(caller, ipv6, port);

    vm.prank(caller);
    announcements.bindKeysAnnounce6(
      ed25519_sig_0,
      ed25519_sig_1,
      ed25519_pub_key,
      ipv6,
      port
    );

    vm.clearMockedCalls();
  }
}
