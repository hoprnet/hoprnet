// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import "../../src/proxy/DummyProxyForNetworkRegistry.sol";
import "forge-std/Test.sol";

contract HoprDummyProxyForNetworkRegistryTest is Test {
    HoprDummyProxyForNetworkRegistry public hoprDummyProxyForNetworkRegistry;
    address public owner;

    /**
     * Manually import events
     */
    event AccountRegistered(address indexed account);
    event AccountDeregistered(address indexed account);
    event AllowAllAccountsEligible(bool isAllowed);

    function setUp() public virtual {
        owner = vm.addr(101); // make address(101) new owner
        hoprDummyProxyForNetworkRegistry = new HoprDummyProxyForNetworkRegistry(owner);
    }

    /**
     * @dev Add account(s)
     * it fails to add account by non-owner
     */
    function testRevert_WhenNotOwnerAddAccount(address caller, address account) public {
        vm.assume(caller != address(0));
        vm.assume(caller != owner);
        vm.prank(caller);

        vm.expectRevert("Ownable: caller is not the owner");
        hoprDummyProxyForNetworkRegistry.ownerAddAccount(account);

        vm.clearMockedCalls();
    }

    /**
     * @dev Add account(s)
     * it can add an account by owner and it becomes eligible
     */
    function testFuzz_OwnerAddAccount(address account) public {
        vm.prank(owner);

        vm.expectEmit(true, false, false, false, address(hoprDummyProxyForNetworkRegistry));
        emit AccountRegistered(account);
        hoprDummyProxyForNetworkRegistry.ownerAddAccount(account);

        // max allowed registration is greater than current registered amount 1
        assertGe(hoprDummyProxyForNetworkRegistry.maxAllowedRegistrations(account), 1);

        vm.clearMockedCalls();
    }

    /**
     * @dev when no account is added, maxAllowedRegistration is zero
     */
    function testFuzz_MaxAllowedRegistrations(address account) public {
        vm.prank(owner);
        assertEq(hoprDummyProxyForNetworkRegistry.maxAllowedRegistrations(account), 0);
    }

    /**
     * @dev canOperateFor is always true
     */
    function testFuzz_MaxAllowedRegistrations(address account, address nodeAddress) public {
        assertTrue(hoprDummyProxyForNetworkRegistry.canOperateFor(account, nodeAddress));
        vm.prank(owner);
    }

    /**
     * @dev Owner can always update the allow all
     */
    function testFuzz_OwnerUpdateAllowAll() public {
        vm.startPrank(owner);
        bool currentAllowAll = hoprDummyProxyForNetworkRegistry.isAllAllowed();
        // update with the same value
        hoprDummyProxyForNetworkRegistry.updateAllowAll(currentAllowAll);
        assertEq(hoprDummyProxyForNetworkRegistry.isAllAllowed(), currentAllowAll);
        // update to the opposite value.
        vm.expectEmit(true, false, false, false, address(hoprDummyProxyForNetworkRegistry));
        emit AllowAllAccountsEligible(!currentAllowAll);
        hoprDummyProxyForNetworkRegistry.updateAllowAll(!currentAllowAll);
        assertEq(hoprDummyProxyForNetworkRegistry.isAllAllowed(), !currentAllowAll);
    }

    /**
     * @dev Add account(s)
     * it can add accounts by owner in batch
     */
    function testFuzz_OwnerBatchAddAccount(address account1, address account2) public {
        vm.prank(owner);
        vm.assume(account1 != account2);

        address[] memory accounts = new address[](2);
        accounts[0] = account1;
        accounts[1] = account2;

        vm.expectEmit(true, false, false, false, address(hoprDummyProxyForNetworkRegistry));
        emit AccountRegistered(account1);
        vm.expectEmit(true, false, false, false, address(hoprDummyProxyForNetworkRegistry));
        emit AccountRegistered(account2);
        hoprDummyProxyForNetworkRegistry.ownerBatchAddAccounts(accounts);

        vm.clearMockedCalls();
    }

    /**
     * @dev Remove account
     * it fails to remove account by non-owner
     */
    function testRevert_WhenNotOwnerRemoveAccount(address caller, address account) public {
        vm.assume(caller != owner);
        vm.prank(owner);
        hoprDummyProxyForNetworkRegistry.ownerAddAccount(account);

        vm.prank(caller);
        vm.expectRevert("Ownable: caller is not the owner");
        hoprDummyProxyForNetworkRegistry.ownerRemoveAccount(account);

        vm.clearMockedCalls();
    }

    /**
     * @dev Remove account
     * it removes account by non-owner
     */
    function testFuzz_OwnerRemoveAccount(address account) public {
        vm.startPrank(owner);
        hoprDummyProxyForNetworkRegistry.ownerAddAccount(account);

        vm.expectEmit(true, false, false, false, address(hoprDummyProxyForNetworkRegistry));
        emit AccountDeregistered(account);
        hoprDummyProxyForNetworkRegistry.ownerRemoveAccount(account);

        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Remove account(s)
     * it can remove accounts by owner in batch
     */
    function testFuzz_OwnerBatchRemoveAccount(address account1, address account2) public {
        vm.startPrank(owner);
        hoprDummyProxyForNetworkRegistry.ownerAddAccount(account1);

        address[] memory accounts = new address[](2);
        accounts[0] = account1;
        accounts[1] = account2;

        vm.expectEmit(true, false, false, false, address(hoprDummyProxyForNetworkRegistry));
        emit AccountDeregistered(account1);
        hoprDummyProxyForNetworkRegistry.ownerBatchRemoveAccounts(accounts);

        vm.clearMockedCalls();
    }
}
