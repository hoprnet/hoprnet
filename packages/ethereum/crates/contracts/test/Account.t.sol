// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./utils/Accounts.sol";
import "forge-std/Test.sol";

contract AccountTest is Test, AccountsFixture {
    function setUp() public {}

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
