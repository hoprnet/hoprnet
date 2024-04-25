// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test } from "forge-std/Test.sol";
import { HoprAnnouncements, HoprAnnouncementsEvents } from "../src/Announcements.sol";
import { HoprNodeSafeRegistry } from "../src/node-stake/NodeSafeRegistry.sol";

// Dummy since there is no verification happening on-chain
bytes32 constant ed25519_sig_0 = 0x000000000000000000000000000000000000000000000000000000000ed25519;
bytes32 constant ed25519_sig_1 = 0x100000000000000000000000000000000000000000000000000000000ed25519;

string constant multiaddress = "/ip6/2604:1380:2000:7a00::1/udp/4001/quic";

bytes32 constant ed25519_pub_key = 0x3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c;

contract AnnouncementsTest is Test {
    HoprNodeSafeRegistry safeRegistry;
    HoprAnnouncements announcements;

    event KeyBinding(bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key, address chain_key);

    function setUp() public {
        safeRegistry = new HoprNodeSafeRegistry();
        announcements = new HoprAnnouncements(safeRegistry);
    }

    function testKeyBinding(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, caller);

        vm.prank(caller);
        announcements.bindKeys(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

        vm.clearMockedCalls();
    }

    event AddressAnnouncement(address node, string baseMultiaddr);

    function testAnnouncements(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(caller, multiaddress);

        vm.prank(caller);
        announcements.announce(multiaddress);

        vm.clearMockedCalls();
    }

    event RevokeAnnouncement(address node);

    function testAddressRevocation(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit RevokeAnnouncement(caller);

        vm.prank(caller);
        announcements.revoke();

        vm.clearMockedCalls();
    }

    function testAllInOneAnnouncement(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, caller);

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(caller, multiaddress);

        bytes[] memory calls = new bytes[](2);

        calls[0] = abi.encodeCall(announcements.bindKeys, (ed25519_sig_0, ed25519_sig_1, ed25519_pub_key));

        calls[1] = abi.encodeCall(announcements.announce, (multiaddress));

        vm.prank(caller);
        announcements.multicall(calls);

        vm.clearMockedCalls();
    }

    function testBindKeyAnnounce(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, caller);

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(caller, multiaddress);

        vm.prank(caller);
        announcements.bindKeysAnnounce(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, multiaddress);

        vm.clearMockedCalls();
    }
}
