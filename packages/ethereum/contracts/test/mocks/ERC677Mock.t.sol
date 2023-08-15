pragma solidity ^0.8.0;

import "../../src/static/stake/mocks/ERC677Mock.sol";
import "forge-std/Test.sol";

/**
 * @title Simplified tests for ERC677 mock
 */
contract ERC677MockTest is Test {
    using stdStorage for StdStorage;

    ERC677Mock public erc677Mock;
    address[] public recipients = new address[](1);

    function setUp() public {
        recipients[0] = vm.addr(101);

        erc677Mock = new ERC677Mock();
    }

    function testFuzz_BatchMintInternal(uint256 amount) public {
        erc677Mock.batchMintInternal(recipients, amount);
        assertEq(erc677Mock.balanceOf(recipients[0]), amount);
    }

    function testRevert_TransferFromDueToHook(address sender, uint256 amount) public {
        address msgSender = vm.addr(200);
        vm.assume(sender != address(0));
        vm.assume(sender != msgSender);
        vm.assume(amount > 0);

        stdstore.target(address(erc677Mock)).sig(erc677Mock.balanceOf.selector).with_key(sender).checked_write(amount);
        assertEq(erc677Mock.balanceOf(sender), amount);

        stdstore.target(address(erc677Mock)).sig(erc677Mock.allowance.selector).with_key(msgSender).with_key(sender)
            .checked_write(amount);
        assertEq(erc677Mock.allowance(msgSender, sender), amount);

        vm.prank(msgSender);
        vm.expectRevert();
        erc677Mock.transferFrom(sender, address(erc677Mock), 0);
        vm.clearMockedCalls();
    }

    function testRevert_TransferDueToHook(address sender, uint256 amount) public {
        address msgSender = vm.addr(200);
        vm.assume(sender != address(0));
        vm.assume(sender != msgSender);
        vm.assume(amount > 0);

        stdstore.target(address(erc677Mock)).sig(erc677Mock.balanceOf.selector).with_key(sender).checked_write(amount);
        assertEq(erc677Mock.balanceOf(sender), amount);

        stdstore.target(address(erc677Mock)).sig(erc677Mock.allowance.selector).with_key(msgSender).with_key(sender)
            .checked_write(amount);
        assertEq(erc677Mock.allowance(msgSender, sender), amount);

        vm.prank(msgSender);
        vm.expectRevert();
        erc677Mock.transfer(address(erc677Mock), 0);
        vm.clearMockedCalls();
    }
}
