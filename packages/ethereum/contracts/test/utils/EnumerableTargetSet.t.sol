// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "../mocks/EnumerableTargetSetMock.sol";
import "../../src/utils/TargetUtils.sol";

contract EnumerableTargetSetTest is Test {
    using stdStorage for StdStorage;
    using TargetUtils for Target;

    EnumerableTargetSetMock public enumerableTargetSetMock;

    /**
     * @dev create mock for test
     */
    function setUp() public {
        enumerableTargetSetMock = new EnumerableTargetSetMock();
    }

    /**
     * @dev fuzz test oln add, length and contains
     */
    function testFuzz_AddLengthContains(uint256 targetVal) public {
        Target target = Target.wrap(targetVal);
        assertTrue(enumerableTargetSetMock.add(target));
        // check target is indeed added
        // check the length of the TargetSet._values
        Target[] memory firstValues = enumerableTargetSetMock.values();
        assertEq(firstValues.length, 1);
        assertEq(enumerableTargetSetMock.length(), 1);
        assertEq(Target.unwrap(firstValues[0]), targetVal);
        // check adding the same target for another time
        assertFalse(enumerableTargetSetMock.add(target));
        Target[] memory secondValues = enumerableTargetSetMock.values();
        assertEq(secondValues.length, 1);
        assertEq(enumerableTargetSetMock.length(), 1);
        assertEq(Target.unwrap(secondValues[0]), targetVal);
        // check set indeed contains the targetAddress
        address targetAddress = target.getTargetAddress();
        assertTrue(enumerableTargetSetMock.contains(targetAddress));
    }

    /**
     * @dev fuzz test on remove a random value from the set
     */
    function testFuzz_RemoveRandom(uint256[] memory targetVals, uint256 index) public {
        vm.assume(targetVals.length > 0);
        // add values to target
        _helperCreateTargetSet(targetVals);
        uint256 length = enumerableTargetSetMock.length();
        index = bound(index, 0, length - 1);

        address targetAddress = enumerableTargetSetMock.values()[index].getTargetAddress();

        if (enumerableTargetSetMock.contains(targetAddress)) {
            // able to remove
            assertTrue(enumerableTargetSetMock.remove(targetAddress));
            assertEq(enumerableTargetSetMock.length(), length - 1);
            // targeAddress is not longer in the set
            assertFalse(enumerableTargetSetMock.contains(targetAddress));
        }
    }

    /**
     * @dev fuzz test on removing the last (and the first if applicable) from the set
     */
    function testFuzz_RemoveTheFirstAndLast(uint256[] memory targetVals) public {
        vm.assume(targetVals.length > 0);
        // add values to target
        _helperCreateTargetSet(targetVals);
        uint256 length = enumerableTargetSetMock.length();

        // remove the last one
        uint256 lastIndex = length - 1;
        address lastTargetAddress = enumerableTargetSetMock.values()[lastIndex].getTargetAddress();
        assertTrue(enumerableTargetSetMock.remove(lastTargetAddress));
        assertEq(enumerableTargetSetMock.length(), length - 1);
        assertFalse(enumerableTargetSetMock.contains(lastTargetAddress));

        // proceed with removing the first if there's still element
        if (enumerableTargetSetMock.length() > 0) {
            address firstTargetAddress = enumerableTargetSetMock.values()[0].getTargetAddress();
            assertTrue(enumerableTargetSetMock.remove(firstTargetAddress));
            assertEq(enumerableTargetSetMock.length(), length - 2);
            assertFalse(enumerableTargetSetMock.contains(firstTargetAddress));
        }
    }

    /**
     * @dev fuzz test on removing a non existing target address
     */
    function testFuzz_RemoveNonExisting(uint256[] memory targetVals, address targetAddress) public {
        // add values to target
        _helperCreateTargetSet(targetVals);
        uint256 length = enumerableTargetSetMock.length();
        Target[] memory values = enumerableTargetSetMock.values();

        vm.assume(!enumerableTargetSetMock.contains(targetAddress));

        // skip removal
        assertFalse(enumerableTargetSetMock.remove(targetAddress));
        assertEq(enumerableTargetSetMock.length(), length);
        for (uint256 j = 0; j < length; j++) {
            // no element has been moved
            assertEq(Target.unwrap(enumerableTargetSetMock.at(j)), Target.unwrap(values[j]));
        }
    }

    /**
     * @dev test values
     */
    function test_Values() public {
        // check default values
        Target[] memory values = enumerableTargetSetMock.values();
        assertEq(values.length, 0);
    }

    /**
     * @dev fuzz test at
     */
    function testFuzz_At(uint256[] memory targetVals) public {
        // add values to target
        _helperCreateTargetSet(targetVals);
        Target[] memory values = enumerableTargetSetMock.values();
        assertEq(values.length, enumerableTargetSetMock.length());

        for (uint256 i = 0; i < values.length; i++) {
            assertEq(Target.unwrap(enumerableTargetSetMock.at(i)), Target.unwrap(values[i]));
        }
    }

    /**
     * @dev fuzz test get and tryGet methods
     */
    function testFuzz_GetAndTryGetWithInArray(uint256[] memory targetVals) public {
        // at least one item can be found from the array
        vm.assume(targetVals.length > 0);

        // add values to target
        _helperCreateTargetSet(targetVals);

        for (uint256 i = 0; i < targetVals.length; i++) {
            address targetAddress = Target.wrap(targetVals[i]).getTargetAddress();
            if (targetAddress != address(0)) {
                // address zero is not a valid target address
                Target target = enumerableTargetSetMock.get(targetAddress);
                assertEq(target.getTargetAddress(), targetAddress);

                (bool tryResult, Target tryTarget) = enumerableTargetSetMock.tryGet(targetAddress);
                assertEq(tryTarget.getTargetAddress(), targetAddress);
                assertTrue(tryResult);
            }
        }
    }

    /**
     * @dev test revert condition of get, namely when the address does not exist
     */
    function testRevert_Get(uint256[] memory targetVals, address targetAddress) public {
        // add values to target
        _helperCreateTargetSet(targetVals);

        bool tryResult;
        if (!enumerableTargetSetMock.contains(targetAddress)) {
            (tryResult,) = enumerableTargetSetMock.tryGet(targetAddress);
            assertFalse(tryResult);

            vm.expectRevert(EnumerableTargetSet.NonExistentKey.selector);
            enumerableTargetSetMock.get(targetAddress);
        } else {
            (tryResult,) = enumerableTargetSetMock.tryGet(targetAddress);
            assertTrue(tryResult);
        }
    }

    /**
     * @dev helper function to create a set with an array of target values in fuzz testing
     */
    function _helperCreateTargetSet(uint256[] memory targetVals) private {
        for (uint256 i = 0; i < targetVals.length; i++) {
            enumerableTargetSetMock.add(Target.wrap(targetVals[i]));
        }
    }
}
