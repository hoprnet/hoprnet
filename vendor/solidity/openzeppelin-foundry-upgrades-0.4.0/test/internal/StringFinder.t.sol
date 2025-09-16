// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {StringFinder} from "openzeppelin-foundry-upgrades/internal/StringFinder.sol";

/**
 * @dev Tests the StringFinder internal library.
 */
contract StringFinderTest is Test {
    using StringFinder for string;

    function testContains() public {
        string memory str = "hello world";
        assertTrue(str.contains("ello"));
        assertFalse(str.contains("Ello"));
    }

    function testStartsWith() public pure {
        string memory str = "hello world";
        assertTrue(str.startsWith("hello"));
        assertFalse(str.startsWith("ello"));
        assertFalse(str.startsWith("Hello"));
        assertTrue(str.startsWith(""));

        string memory empty = "";
        assertFalse(empty.startsWith("a"));
    }

    function testEndsWith() public pure {
        string memory str = "hello world";
        assertTrue(str.endsWith("world"));
        assertFalse(str.endsWith("worl"));
        assertFalse(str.endsWith("World"));
        assertTrue(str.endsWith(""));

        string memory empty = "";
        assertFalse(empty.endsWith("a"));
    }

    function testCount() public pure {
        string memory str = "hello world";
        assertEq(str.count("l"), 3);
        assertEq(str.count("ll"), 1);
        assertEq(str.count("a"), 0);
        assertEq(str.count(""), 12);

        string memory overlap = "aaa";
        assertEq(overlap.count("aa"), 1); // does not count overlapping occurrences

        string memory empty = "";
        assertEq(empty.count("a"), 0);
        assertEq(empty.count(""), 1);
    }
}
