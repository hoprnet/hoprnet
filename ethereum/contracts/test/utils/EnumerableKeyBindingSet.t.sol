// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.0;

import { Test, stdStorage, StdStorage } from "forge-std/Test.sol";
import { KeyBindingWithSignature, EnumerableKeyBindingSet, EnumerableKeyBindingSetMock } from "../mocks/EnumerableKeyBindingSetMock.sol";
import { MAX_KEY_ID } from "../../src/utils/EnumerableKeyBindingSet.sol";

contract EnumerableKeyBindingSetTest is Test {
    using stdStorage for StdStorage;
    using EnumerableKeyBindingSet for KeyBindingWithSignature;

    EnumerableKeyBindingSetMock public enumerableKeyBindingSetMock;

    /**
     * @dev create mock for test
     */
    function setUp() public {
        enumerableKeyBindingSetMock = new EnumerableKeyBindingSetMock();
    }

    /**
     * @dev modifier to create a new instance of the mock contract before each test
     */
    modifier beforeEach() {
        enumerableKeyBindingSetMock = new EnumerableKeyBindingSetMock();
        _;
    }

    modifier respectCurveRangeSingle(bytes32 privKey) {
        // Seckp256k1 curve order
        vm.assume(uint256(privKey) < SECP256K1_ORDER);
        vm.assume(uint256(privKey) != 0);
        _;
    }

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

    /**
     * @dev fuzz test oln add, length and contains
     */
    function testFuzz_AddLengthContains(
        bytes32 pubkey1,
        bytes32 pubkey2
    ) public beforeEach respectCurveRangeSingle(pubkey1) respectCurveRangeSingle(pubkey2) {
        vm.assume(pubkey1 != pubkey2);
        // vm.assume(pubkey1 != pubkey2 && vm.addr(uint256(pubkey1)) != vm.addr(uint256(pubkey2)) );
        KeyBindingWithSignature memory keyBinding = KeyBindingWithSignature(pubkey1, pubkey1, pubkey1, vm.addr(uint256(pubkey1)));
        KeyBindingWithSignature memory keyBinding2 = KeyBindingWithSignature(pubkey2, pubkey2, pubkey2, vm.addr(uint256(pubkey2)));

        // Include the first item
        assertEq(enumerableKeyBindingSetMock.add(keyBinding), 0);
        // check keyBinding is indeed added
        KeyBindingWithSignature[] memory firstValues = enumerableKeyBindingSetMock.values();
        assertEq(firstValues.length, 1);
        assertEq(enumerableKeyBindingSetMock.length(), 1);
        assertTrue(_compareKeyBinding(firstValues[0], keyBinding));

        // check adding another keyBinding
        assertEq(enumerableKeyBindingSetMock.add(keyBinding2), 1);
        KeyBindingWithSignature[] memory secondValues = enumerableKeyBindingSetMock.values();
        assertEq(secondValues.length, 2);
        assertEq(enumerableKeyBindingSetMock.length(), 2);
        assertTrue(_compareKeyBinding(secondValues[1], keyBinding2));
        // check set indeed contains the targetAddress
        assertTrue(enumerableKeyBindingSetMock.contains(pubkey1));
        assertTrue(enumerableKeyBindingSetMock.contains(pubkey2));
    }

    function testRevert_AddExistingKeyBinding(bytes32 pubkey) public beforeEach respectCurveRangeSingle(pubkey) {
        KeyBindingWithSignature memory keyBinding = KeyBindingWithSignature(pubkey, pubkey, pubkey, vm.addr(uint256(pubkey)));
        assertEq(enumerableKeyBindingSetMock.add(keyBinding), 0);
        // check adding the same keyBinding again reverts
        vm.expectRevert(EnumerableKeyBindingSet.ExistingKeyBinding.selector);
        enumerableKeyBindingSetMock.add(keyBinding);
    }

    /**
     * @dev test values
     */
    function test_Values() public beforeEach {
        // check default values
        KeyBindingWithSignature[] memory values = enumerableKeyBindingSetMock.values();
        assertEq(values.length, 0);
    }

    /**
     * @dev fuzz test at
     */
    function testFuzz_At(bytes32[] memory bytes32Vals) public beforeEach respectCurveRange(bytes32Vals) {
        // add unique values to target
        uint256 addedCount = _helperCreateKeyBindingSet(bytes32Vals);
        KeyBindingWithSignature[] memory values = enumerableKeyBindingSetMock.values();
        assertEq(addedCount, enumerableKeyBindingSetMock.length());

        for (uint256 i = 0; i < values.length; i++) {
            // compare each value from at() with values()
            assertTrue(_compareKeyBinding(values[i], enumerableKeyBindingSetMock.at(i)));
        }
    }

    /**
     * @dev fuzz test get and tryGet methods
     */
    function testFuzz_GetAndTryGetWithInArray(bytes32[] memory bytes32Vals) public beforeEach respectCurveRange(bytes32Vals) {
        // at least one item can be found from the array
        vm.assume(bytes32Vals.length > 0);

        // add values to target
        _helperCreateKeyBindingSet(bytes32Vals);

        for (uint256 i = 0; i < bytes32Vals.length; i++) {
            (bool tryResult, uint256 index, KeyBindingWithSignature memory tryBinding) = enumerableKeyBindingSetMock.tryGet(bytes32Vals[i]);
            assertEq(index, i);
            assertTrue(_compareKeyBinding(tryBinding, enumerableKeyBindingSetMock.at(i)));
            assertTrue(tryResult);
        }
    }

    /**
     * @dev test revert condition of get, namely when the address does not exist
     */
    function testRevert_Get(bytes32[] memory bytes32Vals) public beforeEach respectCurveRange(bytes32Vals) {
        // add values to target
        _helperCreateKeyBindingSet(bytes32Vals);

        // bytes32(0) is not going to be added to the set
        bytes32 nonExistentKey = bytes32(0);

        (bool tryResult, uint256 index, KeyBindingWithSignature memory tryBinding) = enumerableKeyBindingSetMock.tryGet(nonExistentKey);
        vm.expectRevert(EnumerableKeyBindingSet.NonExistentKey.selector);
        enumerableKeyBindingSetMock.get(nonExistentKey);
    
        assertFalse(tryResult);
        assertEq(index, 0);
        assertTrue(_compareKeyBinding(tryBinding, KeyBindingWithSignature(bytes32(0), bytes32(0), bytes32(0), address(0))));
    }

    /**
     * @dev test positive condition of get
     */
    function testFuzz_Get(bytes32[] memory bytes32Vals) public beforeEach respectCurveRange(bytes32Vals) {
        // add values to target
        _helperCreateKeyBindingSet(bytes32Vals);

        if (bytes32Vals.length == 0) {
            return;
        } else {
            (bool tryResult, uint256 index, KeyBindingWithSignature memory tryBinding) = enumerableKeyBindingSetMock.tryGet(bytes32Vals[0]);
            assertTrue(tryResult);
            assertEq(index, 0);
            assertTrue(_compareKeyBinding(tryBinding, enumerableKeyBindingSetMock.at(0)));
        }
    }

    /**
     * @dev test revert condition of add, namely when the set is full
     */
    function testRevert_IndexOutOfRange(bytes32 pubkey) public beforeEach respectCurveRangeSingle(pubkey) {
        // assume there are already 0xFFFFFFFF items in the set
        stdstore
            .target(address(enumerableKeyBindingSetMock))
            .sig("length()")
            .checked_write(uint256(MAX_KEY_ID));
    
        // check revert when index is out of range
        vm.expectRevert(EnumerableKeyBindingSet.KeyIdOutOfRange.selector);
        enumerableKeyBindingSetMock.add(KeyBindingWithSignature(pubkey, pubkey, pubkey, vm.addr(uint256(pubkey))));
    }

    /**
     * @dev helper function to create a set for fuzz testing
            chain_key (address) is derived from the uint256 value, non-zero address only
     * @param bytes32Vals array of ed25519_pub_key values to be added to the set
     */
    function _helperCreateKeyBindingSet(bytes32[] memory bytes32Vals) private returns (uint256) {
        uint256 counter = 0;
        for (uint256 i = 0; i < bytes32Vals.length; i++) {
            // only add unique non-existing ed25519_pub_key
            if (!enumerableKeyBindingSetMock.contains(bytes32Vals[i])) {
                enumerableKeyBindingSetMock.add(KeyBindingWithSignature(
                    bytes32Vals[i],
                    bytes32Vals[i],
                    bytes32Vals[i],
                    vm.addr(uint256(bytes32Vals[i]))
                ));
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