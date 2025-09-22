// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { Test } from "forge-std/Test.sol";
import { HoprAnnouncements, ZeroAddress, EmptyMultiaddr, KeyBindingWithSignature, KeyId } from "../src/Announcements.sol";
import { HoprNodeSafeRegistry } from "../src/node-stake/NodeSafeRegistry.sol";

// Dummy since there is no verification happening on-chain
bytes32 constant ED25519_SIG_0 = 0x000000000000000000000000000000000000000000000000000000000ed25519;
bytes32 constant ED25519_SIG_1 = 0x100000000000000000000000000000000000000000000000000000000ed25519;
bytes32 constant ED25519_PUB_KEY = 0x3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c;

string constant MULTIADDRESS = "/ip6/2604:1380:2000:7a00::1/udp/4001/quic";

/// forge-lint:disable-next-item(mixed-case-variable)
contract AnnouncementsTest is Test {
    HoprNodeSafeRegistry safeRegistry;
    HoprAnnouncements announcements;

    event KeyBinding(bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key, address chain_key);

    modifier respectCurveRange(bytes32[] memory keys) {
        // Seckp256k1 curve order
        for (uint256 i = 0; i < keys.length; i++) {
            // private keys do not exceed the curve order
            vm.assume(uint256(keys[i]) < SECP256K1_ORDER);
            // private key cannot be zero
            vm.assume(uint256(keys[i]) != 0);
            // private keys are not leading to the same address
            for (uint256 j = 0; j < i; j++) {
                vm.assume(keys[i] != keys[j]);
            }
        }
        _;
    }

    function setUp() public {
        safeRegistry = new HoprNodeSafeRegistry();
        announcements = new HoprAnnouncements(safeRegistry);
    }

    function testRevert_ZeroAddressOnDeployment() public {
        vm.expectRevert(abi.encodeWithSelector(ZeroAddress.selector, "safeRegistry must not be empty"));
        announcements = new HoprAnnouncements(HoprNodeSafeRegistry(address(0)));
    }

    function testKeyBinding(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit KeyBinding(ED25519_SIG_0, ED25519_SIG_1, ED25519_PUB_KEY, caller);

        vm.prank(caller);
        announcements.bindKeys(ED25519_SIG_0, ED25519_SIG_1, ED25519_PUB_KEY);

        vm.clearMockedCalls();
    }

    event AddressAnnouncement(address node, string baseMultiaddr);

    function testAnnouncements(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(caller, MULTIADDRESS);

        vm.prank(caller);
        announcements.announce(MULTIADDRESS);

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
        emit KeyBinding(ED25519_SIG_0, ED25519_SIG_1, ED25519_PUB_KEY, caller);

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(caller, MULTIADDRESS);

        bytes[] memory calls = new bytes[](2);

        calls[0] = abi.encodeCall(announcements.bindKeys, (ED25519_SIG_0, ED25519_SIG_1, ED25519_PUB_KEY));

        calls[1] = abi.encodeCall(announcements.announce, (MULTIADDRESS));

        vm.prank(caller);
        announcements.multicall(calls);

        vm.clearMockedCalls();
    }

    function testBindKeyAnnounce(address caller) public {
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit KeyBinding(ED25519_SIG_0, ED25519_SIG_1, ED25519_PUB_KEY, caller);

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(caller, MULTIADDRESS);

        vm.prank(caller);
        announcements.bindKeysAnnounce(ED25519_SIG_0, ED25519_SIG_1, ED25519_PUB_KEY, MULTIADDRESS);

        vm.clearMockedCalls();
    }

    function testFuzz_BindKeysSafe(address nodeAddress, bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key) public {
        address safeAddress = vm.addr(888);
        vm.assume(nodeAddress != address(0));
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", nodeAddress), abi.encode(safeAddress)
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, nodeAddress);

        vm.prank(safeAddress);
        announcements.bindKeysSafe(nodeAddress, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key);

        vm.clearMockedCalls();
    }

    function testFuzz_BindKeysAnnounceSafe(address nodeAddress, bytes32 ed25519_sig_0, bytes32 ed25519_sig_1, bytes32 ed25519_pub_key) public {
        address safeAddress = vm.addr(888);
    
        vm.assume(nodeAddress != address(0));
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", nodeAddress), abi.encode(safeAddress)
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit KeyBinding(ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, nodeAddress);

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(nodeAddress, MULTIADDRESS);

        vm.prank(safeAddress);
        announcements.bindKeysAnnounceSafe(nodeAddress, ed25519_sig_0, ed25519_sig_1, ed25519_pub_key, MULTIADDRESS);

        vm.clearMockedCalls();
    }

    function testFuzz_AnnounceSafe(address nodeAddress) public {
        address safeAddress = vm.addr(888);
        vm.assume(nodeAddress != address(0));
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", nodeAddress), abi.encode(safeAddress)
        );

        vm.expectEmit(true, false, false, false, address(announcements));
        emit AddressAnnouncement(nodeAddress, MULTIADDRESS);

        vm.prank(safeAddress);
        announcements.announceSafe(nodeAddress, MULTIADDRESS);

        vm.clearMockedCalls();
    }

    function testFuzz_RevokeSafe(address nodeAddress) public {
        address safeAddress = vm.addr(888);
        vm.assume(nodeAddress != address(0));
        vm.mockCall(
            address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", nodeAddress), abi.encode(safeAddress)
        );
        vm.expectEmit(true, false, false, false, address(announcements));
        emit RevokeAnnouncement(nodeAddress);
        vm.prank(safeAddress);
        announcements.revokeSafe(nodeAddress);
        vm.clearMockedCalls();
    }

    function testRevert_EmptyMultiAddr() public {
        vm.expectRevert(EmptyMultiaddr.selector);
        announcements.announce("");
    }

    function testFuzz_GetKeyBinding(bytes32[] memory bytes32Vals) public respectCurveRange(bytes32Vals) {
        uint256 createdCount = _helperCreateKeyBindingSet(bytes32Vals);

        KeyBindingWithSignature[] memory results = announcements.getAllKeyBindings();

        assertEq(results.length, createdCount);
        assertEq(results.length, announcements.getKeyBindingCount());

        for (uint256 i = 0; i < createdCount; i++) {
            KeyBindingWithSignature memory result_i = announcements.getKeyBindingWithKeyId(KeyId.wrap(uint32(i)));
            assertTrue(announcements.isOffchainKeyBound(result_i.ed25519_pub_key));

            (bool success, KeyId index, KeyBindingWithSignature memory tryBinding_i) = announcements.tryGetKeyBinding(result_i.ed25519_pub_key);
            assertTrue(success);
            assertEq(KeyId.unwrap(index), uint32(i));
            assertTrue(_compareKeyBinding(tryBinding_i, result_i));

            (bool success2, KeyId keyId) = announcements.getKeyIdWithOffchainKey(result_i.ed25519_pub_key);
            assertTrue(success2);
            assertEq(uint32(KeyId.unwrap(keyId)), i);

            KeyBindingWithSignature memory at_i = announcements.getKeyBindingWithKeyId(KeyId.wrap(uint32(i)));
            assertTrue(_compareKeyBinding(at_i, result_i)); 

            bytes32 pubkey_i = announcements.getOffchainKeyWithKeyId(KeyId.wrap(uint32(i)));
            assertEq(pubkey_i, result_i.ed25519_pub_key);
        }

        vm.clearMockedCalls();
    }

    function test_GetKeyIdRange() public {
        (uint32 minKeyId, uint32 maxKeyId) = announcements.getKeyIdRange();
        assertEq(minKeyId, 0);
        assertEq(maxKeyId, type(uint32).max);
    }

    /**
    * @dev helper function to create a set for fuzz testing
     */
    function _helperCreateKeyBindingSet(bytes32[] memory bytes32Vals) private returns (uint256) {
        uint256 counter = 0;
        for (uint256 i = 0; i < bytes32Vals.length; i++) {
            address caller = vm.addr(uint256(bytes32Vals[i]));
            vm.mockCall(
                address(safeRegistry), abi.encodeWithSignature("nodeToSafe(address)", caller), abi.encode(address(0))
            );
            vm.prank(caller);
            // only add unique non-existing ed25519_pub_key
            if (!announcements.isOffchainKeyBound(bytes32Vals[i])) {
                announcements.bindKeys(
                    bytes32Vals[i],
                    bytes32Vals[i],
                    bytes32Vals[i]
                );
                counter++;
            }
        }
        return counter;
    }

    function _compareKeyBinding(KeyBindingWithSignature memory a, KeyBindingWithSignature memory b) private pure returns (bool) {
        return (a.ed25519_sig_0 == b.ed25519_sig_0 &&
                a.ed25519_sig_1 == b.ed25519_sig_1 &&
                a.ed25519_pub_key == b.ed25519_pub_key &&
                a.chain_key == b.chain_key);
    }
}
