// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "forge-std/Test.sol";

import { ERC777SnapshotMock } from "../mocks/ERC777SnapshotMock.sol";
import { ERC1820RegistryFixtureTest } from "../utils/ERC1820Registry.sol";

contract ERC777SnapshotTest is Test, ERC1820RegistryFixtureTest {
    // to alter the storage
    using stdStorage for StdStorage;

    ERC777SnapshotMock public erc777SnapshotMock;
    string public NAME = "ERC 777 Token";
    string public SYMBOL = "ERC777";
    address public INITIAL_HOLDER;
    uint256 public INITIAL_BALANCE = 10 ether;
    uint128 public INITIAL_MINT_BLOCK;
    address public DEFAULT_RECIPIENT;
    address public OTHER_RECIPIENT;

    function setUp() public virtual override {
        super.setUp();
        INITIAL_HOLDER = vm.addr(100); // make address(100) default operator
        DEFAULT_RECIPIENT = vm.addr(101); // make address(101) default recipient
        OTHER_RECIPIENT = vm.addr(102); // make address(102) a recipient other than the default recipient
        erc777SnapshotMock = new ERC777SnapshotMock('ERC 777 Token', 'ERC777', INITIAL_HOLDER, INITIAL_BALANCE);
        INITIAL_MINT_BLOCK = uint128(block.number);
    }

    /**
     * @dev it should revert when trying to snapshot unsupported amount
     */
    function testRevert_UpdateUnsupportedAmount() public {
        // vm.prank(owner);
        vm.expectRevert("casting overflow");
        erc777SnapshotMock.updateValueAtNowAccount(INITIAL_HOLDER, type(uint256).max);
    }

    /**
     * @dev valueAt,
     * it should return account balance 0 at block 0
     * it should return total supply balance 0 at block 0
     */
    function test_ValueAtInitialHolder() public {
        // it should return account balance 0 at block 0
        uint256 balanceAtBlockZero = erc777SnapshotMock.getAccountValueAt(INITIAL_HOLDER, 0);
        assertEq(balanceAtBlockZero, 0);
        // it should return total supply balance 0 at block 0
        uint256 totalSupplyAtBlockZero = erc777SnapshotMock.getTotalSupplyValueAt(0);
        assertEq(totalSupplyAtBlockZero, 0);
    }

    /**
     * @dev valueAt,
     * it should return unknown account balance 0 at block 0
     */
    function testFuzz_ValueAtOtherHolder(address otherAddress) public {
        vm.assume(otherAddress != INITIAL_HOLDER);
        // it should return unknown account balance 0 at block 0
        uint256 balanceAtBlockZero = erc777SnapshotMock.getAccountValueAt(otherAddress, 0);
        assertEq(balanceAtBlockZero, 0);
    }

    /**
     * @dev it should return account balance at block
     */
    function test_CheckAccountBalance() public {
        vm.startPrank(INITIAL_HOLDER);
        uint128 currentBlockNumber = uint128(block.number);

        // transfer 20 tokens every 10 blocks
        for (uint128 i = 0; i < 10; i++) {
            vm.roll(currentBlockNumber + 10 * i);
            erc777SnapshotMock.transfer(DEFAULT_RECIPIENT, 20);
        }

        for (uint128 j = 0; j < 10; j++) {
            uint256 totalSupplyAtBlockZero = erc777SnapshotMock.balanceOfAt(DEFAULT_RECIPIENT, 10 * j);
            assertEq(totalSupplyAtBlockZero, 20 * j);
        }
    }

    /**
     * @dev totalSupplyAt, with no supply changes after the snapshot
     * it should return 0 at block 0
     * it should return latest totalSupply at block number after creation
     * it should return latest totalSupply at a not-yet-created block number
     */
    function test_TotalSupplyAt() public {
        assertEq(INITIAL_MINT_BLOCK, 1);
        assertEq(erc777SnapshotMock.totalSupplyAt(0), 0);
        assertEq(erc777SnapshotMock.totalSupplyAt(INITIAL_MINT_BLOCK), INITIAL_BALANCE);
        assertEq(erc777SnapshotMock.totalSupplyAt(uint128(block.number + 1)), INITIAL_BALANCE);
    }

    /**
     * @dev totalSupplyAt, with supply changes after the snapshot
     * it returns the total supply before the changes
     * snapshots return the supply before and after the changes
     * all posterior snapshots return the supply after the changes
     */
    function test_TotalSupplyAtWithSupplyChanges() public {
        vm.roll(5);
        erc777SnapshotMock.mint(OTHER_RECIPIENT, 5 ether, hex"00", hex"00");
        vm.roll(6);
        erc777SnapshotMock.burn(INITIAL_HOLDER, 2 ether, hex"00", hex"00");
        assertEq(INITIAL_MINT_BLOCK, 1);
        assertEq(erc777SnapshotMock.totalSupplyAt(0), 0);

        // snapshots return the supply before and after the changes
        assertEq(erc777SnapshotMock.totalSupplyAt(INITIAL_MINT_BLOCK), INITIAL_BALANCE);
        assertEq(erc777SnapshotMock.totalSupplyAt(5), INITIAL_BALANCE + 5 ether);
        assertEq(erc777SnapshotMock.totalSupplyAt(6), INITIAL_BALANCE + 3 ether);

        // all posterior snapshots return the supply after the changes
        for (uint128 i = 0; i < 10; i++) {
            assertEq(erc777SnapshotMock.totalSupplyAt(6 + i), INITIAL_BALANCE + 3 ether);
        }
    }

    /**
     * @dev balanceOfAt
     */
    function test_BalanceOfAtWithSupplyChanges() public {
        // it should return 0 at block 0
        assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, 0), 0);
        // it should return latest balance at block number after creation
        assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, INITIAL_MINT_BLOCK), INITIAL_BALANCE);
        // it should return latest balance at a not-yet-created block number
        assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, 5), INITIAL_BALANCE);
        // with no balance changes after the snapshot, it returns the current balance for all accounts
        vm.roll(5);
        assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, 6), INITIAL_BALANCE);
        // with balance changes after the snapshot
        vm.roll(6);
        vm.prank(INITIAL_HOLDER);
        erc777SnapshotMock.transfer(DEFAULT_RECIPIENT, 1 ether);
        vm.roll(7);
        erc777SnapshotMock.mint(DEFAULT_RECIPIENT, 5 ether, hex"00", hex"00");
        vm.roll(8);
        erc777SnapshotMock.burn(INITIAL_HOLDER, 2 ether, hex"00", hex"00");

        // snapshots return the supply before and after the changes
        assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, 6), INITIAL_BALANCE - 1 ether);
        assertEq(erc777SnapshotMock.balanceOfAt(DEFAULT_RECIPIENT, 6), 1 ether);
        assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, 7), INITIAL_BALANCE - 1 ether);
        assertEq(erc777SnapshotMock.balanceOfAt(DEFAULT_RECIPIENT, 7), 6 ether);
        assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, 8), INITIAL_BALANCE - 3 ether);
        assertEq(erc777SnapshotMock.balanceOfAt(DEFAULT_RECIPIENT, 8), 6 ether);

        // all posterior snapshots return the supply after the changes
        for (uint128 i = 0; i < 10; i++) {
            assertEq(erc777SnapshotMock.balanceOfAt(INITIAL_HOLDER, 8), INITIAL_BALANCE - 3 ether);
            assertEq(erc777SnapshotMock.balanceOfAt(DEFAULT_RECIPIENT, 8), 6 ether);
        }
    }
}
