// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import { AccountsFixtureTest } from "./utils/Accounts.sol";
import { Test } from "forge-std/Test.sol";

contract AccountTest is Test, AccountsFixtureTest {
    function setUp() public { }

    function testAccountAIsCorrect() public {
        assertTrue(address(uint160(uint256(keccak256(accountA.publicKey)))) == accountA.accountAddr);
        assertTrue(vm.addr(accountA.privateKey) == accountA.accountAddr);
    }

    function testAccountBIsCorrect() public {
        assertTrue(address(uint160(uint256(keccak256(accountB.publicKey)))) == accountB.accountAddr);
        assertTrue(vm.addr(accountB.privateKey) == accountB.accountAddr);
    }

    function testAccountCIsCorrect() public {
        assertTrue(address(uint160(uint256(keccak256(accountC.publicKey)))) == accountC.accountAddr);
        assertTrue(vm.addr(accountC.privateKey) == accountC.accountAddr);
    }
}
