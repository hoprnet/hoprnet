// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity >=0.6.0 <0.9.0;

import {
    IHoprNetworkRegistryRequirement, HoprNetworkRegistry, HoprNetworkRegistryEvents
} from "../src/NetworkRegistry.sol";
import { Test, stdStorage, StdStorage } from "forge-std/Test.sol";

contract HoprNetworkRegistryTest is Test, HoprNetworkRegistryEvents {
    // to alter the storage
    using stdStorage for StdStorage;

    HoprNetworkRegistry public hoprNetworkRegistry;
    address public proxy;
    address public owner;
    address[] public stakingAccounts;
    uint256 public constant STAKING_ACCOUNTS_SIZE = 5;
    // uint256[] public allowances;

    function setUp() public {
        proxy = vm.addr(100); // make vm.addr(100) requirementImplementation
        owner = vm.addr(101); // make address(101) new owner
        hoprNetworkRegistry = new HoprNetworkRegistry(proxy, owner, owner);

        // create some unique staking account
        stakingAccounts = new address[](STAKING_ACCOUNTS_SIZE);
        for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
            stakingAccounts[i] = vm.addr(i + 1);
        }
        // allowances = new uint256[](STAKING_ACCOUNTS_SIZE);
    }

    /**
     * @dev mock the return of proxy for created staking accounts
     */
    function testFuzz_MockProxyReturn(uint256[STAKING_ACCOUNTS_SIZE] memory allowances) public {
        _helperMockProxyReturns(allowances);

        for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
            (, bytes memory returndataAllowance) =
                proxy.staticcall(abi.encodeWithSignature("maxAllowedRegistrations(address)", vm.addr(i + 1)));
            uint256 allowance = abi.decode(returndataAllowance, (uint256));
            assertEq(allowance, allowances[i]);
            // check eligibility
            if (allowance > 0) {
                assertTrue(hoprNetworkRegistry.isAccountEligible(stakingAccounts[i]));
            } else {
                assertFalse(hoprNetworkRegistry.isAccountEligible(stakingAccounts[i]));
            }
        }

        vm.clearMockedCalls();
    }

    /**
     * @dev the maximum ndoes in addition that a node can register, in theory
     */
    function testFuzz_maxAdditionalRegistrations(uint256 allowance, uint256 registered) public {
        address account = vm.addr(404);
        vm.mockCall(
            proxy,
            abi.encodeWithSelector(IHoprNetworkRegistryRequirement.maxAllowedRegistrations.selector, account),
            abi.encode(allowance)
        );
        stdstore.target(address(hoprNetworkRegistry)).sig("countRegisterdNodesPerAccount(address)").with_key(account)
            .depth(0).checked_write(registered);

        if (allowance > registered) {
            assertEq(hoprNetworkRegistry.maxAdditionalRegistrations(account), allowance - registered);
        } else {
            assertEq(hoprNetworkRegistry.maxAdditionalRegistrations(account), 0);
        }
        vm.clearMockedCalls();
    }
    /**
     * @dev verify that return value of canOperateFor is correct
     */

    function testFuzz_MockCanOperateFor(
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances,
        uint256 accountIndex
    )
        public
    {
        accountIndex = bound(accountIndex, 0, STAKING_ACCOUNTS_SIZE - 1);
        _helperMockProxyReturns(allowances);

        // the add some nodes
        address[] memory nodeAddresses = _helperCreateNodeAddresses(accountIndex);

        for (uint256 i = 1; i < 5; i++) {
            emit log_named_uint("asserting i", i);
            // read allowance
            (bool successRead1, bytes memory returndataCanOperateFor) = proxy.staticcall(
                abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector, vm.addr(i))
            );
            vm.assume(successRead1);
            // only continue when a value is returned
            bool canOperateFor = abi.decode(returndataCanOperateFor, (bool));
            assertFalse(canOperateFor);
        }

        uint256 j = 0;
        if (accountIndex > 0) {
            for (j = 0; j < accountIndex; j++) {
                emit log_named_uint("asserting j", j);
                // read allowance
                (bool successRead2, bytes memory returndataCanOperateFor) = proxy.staticcall(
                    abi.encodeWithSelector(
                        IHoprNetworkRegistryRequirement.canOperateFor.selector,
                        stakingAccounts[accountIndex],
                        nodeAddresses[j]
                    )
                );
                vm.assume(successRead2);
                // only continue when a value is returned
                bool canOperateFor = abi.decode(returndataCanOperateFor, (bool));
                assertTrue(canOperateFor);
            }
        }

        for (uint256 k = j + 1; k < 15; k++) {
            // read allowance
            (bool successRead3, bytes memory returndataCanOperateFor) = proxy.staticcall(
                abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector, vm.addr(k))
            );
            vm.assume(successRead3);
            // only continue when a value is returned
            bool canOperateFor = abi.decode(returndataCanOperateFor, (bool));
            assertFalse(canOperateFor);
        }

        vm.clearMockedCalls();
    }

    /**
     * @dev Owner can update important parameters of the contract:
     * it is enabled globally
     */
    function test_IsEnabled() public {
        assertTrue(hoprNetworkRegistry.enabled());
    }

    /**
     * @dev Owner can update important parameters of the contract:
     * it allows owner to update the registry
     */
    function testFuzz_OwnerUpdateRegistry(address newImp) public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
        emit RequirementUpdated(newImp);
        hoprNetworkRegistry.updateRequirementImplementation(newImp);
    }

    /**
     * @dev Owner can update important parameters of the contract:
     * it fails to enable an enabled registry by its owner
     */
    function testRevert_WhenEnablingAnEnabledRegistry() public {
        vm.prank(owner);
        vm.expectRevert(HoprNetworkRegistry.GloballyEnabledRegistry.selector);
        hoprNetworkRegistry.enableRegistry();
    }

    /**
     * @dev Owner can update important parameters of the contract:
     * it allows owner to disable the registry
     */
    function test_OwnerDisableRegistry() public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
        emit NetworkRegistryStatusUpdated(false);
        hoprNetworkRegistry.disableRegistry();
    }

    /**
     * @dev Owner can update important parameters of the contract:
     * it fails to enable an enabled registry by its owner
     */
    function testRevert_WhenDisablingADisnabledRegistry() public {
        vm.startPrank(owner);
        hoprNetworkRegistry.disableRegistry();
        vm.expectRevert(HoprNetworkRegistry.GloballyDisabledRegistry.selector);
        hoprNetworkRegistry.disableRegistry();
    }

    /**
     * @dev Owner can update important parameters of the contract:
     * it allows owner to enable an enabled registry
     */
    function test_OwnerEnsableRegistry() public {
        vm.startPrank(owner);
        hoprNetworkRegistry.disableRegistry();

        vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
        emit NetworkRegistryStatusUpdated(true);
        hoprNetworkRegistry.enableRegistry();
    }

    /**
     * @dev Register contract for a single time:
     * it can self-register when the requirement is fulfilled and emits true
     */
    function testFuzz_SelfRegisterAddresses(
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances,
        uint256 accountIndex
    )
        public
    {
        accountIndex = bound(accountIndex, 0, STAKING_ACCOUNTS_SIZE - 1);
        // ensure that the allowance at the index is suffiicent
        emit log_named_uint("allowances length", allowances.length);
        allowances[accountIndex] = accountIndex;
        emit log_named_uint("accountIndex", accountIndex);
        _helperMockProxyReturns(allowances);

        // ensure the staking account is created
        address stakingAccount = stakingAccounts[accountIndex];
        address[] memory nodeAddresses = _helperCreateNodeAddresses(accountIndex);

        vm.prank(stakingAccount);
        for (uint256 i = 0; i < nodeAddresses.length; i++) {
            emit log_named_uint("expect emit Registered", i);
            vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
            emit Registered(stakingAccount, nodeAddresses[i]);
        }
        if (nodeAddresses.length > 0) {
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit EligibilityUpdated(stakingAccount, true);
            hoprNetworkRegistry.selfRegister(nodeAddresses);
        }

        // registered nodes are eligible
        for (uint256 j = 0; j < nodeAddresses.length; j++) {
            assertTrue(hoprNetworkRegistry.isNodeRegisteredAndEligible(nodeAddresses[j]));
            assertTrue(hoprNetworkRegistry.isNodeRegisteredByAccount(nodeAddresses[j], stakingAccount));
        }
        vm.clearMockedCalls();
    }

    /**
     * @dev Fail to register nodes exceeding its allowance
     */
    function testRevert_SelfRegisterTooManyAddresses(
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances,
        uint256 accountIndex
    )
        public
    {
        accountIndex = bound(accountIndex, 0, STAKING_ACCOUNTS_SIZE - 1);
        // ensure that the allowance at the index is suffiicent
        emit log_named_uint("allowances length", allowances.length);
        vm.assume(allowances[accountIndex] < accountIndex);
        emit log_named_uint("accountIndex", accountIndex);
        _helperMockProxyReturns(allowances);

        // ensure the staking account is created
        address stakingAccount = stakingAccounts[accountIndex];
        address[] memory nodeAddresses = _helperCreateNodeAddresses(accountIndex);

        vm.prank(stakingAccount);
        vm.expectRevert(HoprNetworkRegistry.NotEnoughAllowanceToRegisterNode.selector);
        hoprNetworkRegistry.selfRegister(nodeAddresses);
        vm.clearMockedCalls();
    }

    /**
     * @dev fails to register by the manager when array lengths not match
     */
    function testRevert_ManagerRegisterWithWrongArrayLengths() public {
        address[] memory nodeAddresses = _helperCreateNodeAddresses(STAKING_ACCOUNTS_SIZE + 1);

        vm.prank(owner);
        vm.expectRevert(HoprNetworkRegistry.ArrayLengthNotMatch.selector);
        hoprNetworkRegistry.managerRegister(stakingAccounts, nodeAddresses);
        vm.clearMockedCalls();
    }

    /**
     * @dev fails to register by the manager when array lengths not match
     */
    function testRevert_ManagerRegisterRegisteredNode() public {
        address[] memory oldAccounts = new address[](1);
        oldAccounts[0] = stakingAccounts[1];
        address[] memory accounts = new address[](1);
        accounts[0] = stakingAccounts[0];
        address[] memory nodeAddresses = _helperCreateNodeAddresses(1);

        vm.startPrank(owner);
        hoprNetworkRegistry.managerRegister(oldAccounts, nodeAddresses);
        vm.expectRevert(abi.encodeWithSelector(HoprNetworkRegistry.NodeAlreadyRegisterd.selector, nodeAddresses[0]));
        hoprNetworkRegistry.managerRegister(accounts, nodeAddresses);
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Manager can register safe and node
     */
    function test_ManagerRegister() public {
        // mock the entire function that not acount can register any node
        vm.mockCall(
            proxy,
            abi.encodeWithSelector(IHoprNetworkRegistryRequirement.maxAllowedRegistrations.selector),
            abi.encode(0)
        );

        address[] memory nodeAddresses = _helperCreateNodeAddresses(STAKING_ACCOUNTS_SIZE);

        vm.prank(owner);
        for (uint256 i = 0; i < stakingAccounts.length; i++) {
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit RegisteredByManager(stakingAccounts[i], nodeAddresses[i]);
        }
        hoprNetworkRegistry.managerRegister(stakingAccounts, nodeAddresses);

        // registerd nodes are beyond their eligibility
        for (uint256 j = 0; j < nodeAddresses.length; j++) {
            assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(nodeAddresses[j]));
            assertTrue(hoprNetworkRegistry.isNodeRegisteredByAccount(nodeAddresses[j], stakingAccounts[j]));
        }
        vm.clearMockedCalls();
    }

    /**
     * @dev Manager cannot deregister non registered nodes
     */
    function testRevert_ManagerDeregisterAnNonRegisteredNode() public {
        address[] memory nodeAddresses = _helperCreateNodeAddresses(STAKING_ACCOUNTS_SIZE);

        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(HoprNetworkRegistry.NodeNotYetRegisterd.selector, nodeAddresses[0]));
        hoprNetworkRegistry.managerDeregister(nodeAddresses);
        vm.clearMockedCalls();
    }

    /**
     * @dev registered nodes can be deregistered by a manager
     */
    function test_ManagerDeregisterRegisteredNodes() public {
        _helperRegisterAllNodeAddresses();
        address[] memory nodes = new address[](10);
        for (uint256 i = 0; i < nodes.length; i++) {
            nodes[i] = vm.addr(STAKING_ACCOUNTS_SIZE + 1 + i);
        }

        vm.startPrank(owner);
        uint256 startingIndex;
        for (uint256 j = 1; j < stakingAccounts.length; j++) {
            for (uint256 k = 0; k < j; k++) {
                vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
                emit DeregisteredByManager(stakingAccounts[j], nodes[startingIndex]);
                startingIndex++;
            }
        }
        hoprNetworkRegistry.managerDeregister(nodes);

        uint256 startingInd;
        for (uint256 p = 0; p < stakingAccounts.length; p++) {
            assertEq(hoprNetworkRegistry.countRegisterdNodesPerAccount(stakingAccounts[p]), 0);
            for (uint256 q = 0; q < p; q++) {
                assertEq(hoprNetworkRegistry.nodeRegisterdToAccount(nodes[startingInd]), address(0));
                startingInd++;
            }
        }
        vm.clearMockedCalls();
    }

    /**
     * @dev Manager sync nodes based on their latest eligibility
     */
    function test_ManagerSyncStakingAccounts() public {
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances = [uint256(0), uint256(1), uint256(2), uint256(3), uint256(4)];
        // mock requirement function returns
        _helperMockProxyReturns(allowances);

        // mock the number of curent registered nodes
        for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
            stdstore.target(address(hoprNetworkRegistry)).sig("countRegisterdNodesPerAccount(address)").with_key(
                stakingAccounts[i]
            ).depth(0).checked_write(2);
        }

        vm.prank(owner);
        for (uint256 j = 0; j < STAKING_ACCOUNTS_SIZE; j++) {
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit EligibilityUpdated(stakingAccounts[j], allowances[j] >= 2);
        }
        hoprNetworkRegistry.managerSync(stakingAccounts);
        vm.clearMockedCalls();
    }

    /**
     * @dev sync nodes based on their latest eligibility
     */
    function test_SelfSyncStakingAccounts() public {
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances = [uint256(0), uint256(1), uint256(2), uint256(3), uint256(4)];
        // mock requirement function returns
        _helperMockProxyReturns(allowances);

        // mock the number of curent registered nodes
        for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
            stdstore.target(address(hoprNetworkRegistry)).sig("countRegisterdNodesPerAccount(address)").with_key(
                stakingAccounts[i]
            ).depth(0).checked_write(2);
        }

        for (uint256 j = 0; j < STAKING_ACCOUNTS_SIZE; j++) {
            vm.prank(stakingAccounts[j]);
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit EligibilityUpdated(stakingAccounts[j], allowances[j] >= 2);
            hoprNetworkRegistry.selfSync();
        }
        vm.clearMockedCalls();
    }

    /**
     * @dev Manager sync nodes based on their latest eligibility
     */
    function test_ManagerForceSyncStakingAccounts() public {
        bool[] memory eligibilities = new bool[](STAKING_ACCOUNTS_SIZE);
        bool[] memory allTrue = new bool[](STAKING_ACCOUNTS_SIZE);
        bool[] memory allFalse = new bool[](STAKING_ACCOUNTS_SIZE);

        // mock the number of curent registered nodes
        for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
            eligibilities[i] = i % 2 == 0 ? true : false;
            allTrue[i] = true;
            allFalse[i] = false;
        }

        vm.startPrank(owner);
        for (uint256 j = 0; j < STAKING_ACCOUNTS_SIZE; j++) {
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit EligibilityUpdated(stakingAccounts[j], eligibilities[j]);
        }
        hoprNetworkRegistry.managerForceSync(stakingAccounts, eligibilities);

        for (uint256 k = 0; k < STAKING_ACCOUNTS_SIZE; k++) {
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit EligibilityUpdated(stakingAccounts[k], true);
        }
        hoprNetworkRegistry.managerForceSync(stakingAccounts, allTrue);

        for (uint256 m = 0; m < STAKING_ACCOUNTS_SIZE; m++) {
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit EligibilityUpdated(stakingAccounts[m], false);
        }
        hoprNetworkRegistry.managerForceSync(stakingAccounts, allFalse);
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Manager sync nodes based on their latest eligibility
     */
    function testRevert_ManagerForceSyncStakingAccountsWrongLength() public {
        bool[] memory eligibilities = new bool[](STAKING_ACCOUNTS_SIZE + 1);

        // mock the number of curent registered nodes
        for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE + 1; i++) {
            eligibilities[i] = i % 2 == 0 ? true : false;
        }

        vm.prank(owner);
        vm.expectRevert(HoprNetworkRegistry.ArrayLengthNotMatch.selector);
        hoprNetworkRegistry.managerForceSync(stakingAccounts, eligibilities);
        vm.clearMockedCalls();
    }

    /**
     * @dev Fail to deregister an non-registered node
     */
    function testRevert_DeregisterANonRegisteredNode() public {
        _helperRegisterAllNodeAddresses();
        address stakingAccount = stakingAccounts[3];
        address[] memory nodeAddresses = _helperCreateNodeAddresses(4);

        for (uint256 i = 0; i < nodeAddresses.length; i++) {
            assertNotEq(hoprNetworkRegistry.nodeRegisterdToAccount(nodeAddresses[i]), stakingAccount);
        }
        vm.prank(stakingAccount);
        vm.expectRevert(abi.encodeWithSelector(HoprNetworkRegistry.CannotOperateForNode.selector, nodeAddresses[0]));
        hoprNetworkRegistry.selfDeregister(nodeAddresses);
        vm.clearMockedCalls();
    }

    /**
     * @dev Fail to deregister an account registered under another account
     */
    function testRevert_DeregisterForOtherAccount() public {
        address[] memory oldAccounts = new address[](1);
        oldAccounts[0] = stakingAccounts[1];
        address[] memory accounts = new address[](1);
        accounts[0] = stakingAccounts[2];
        address[] memory nodeAddresses = _helperCreateNodeAddresses(1);

        // account is allowed to operate for node
        vm.mockCall(
            proxy, abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector), abi.encode(true)
        );

        vm.prank(owner);
        hoprNetworkRegistry.managerRegister(oldAccounts, nodeAddresses);

        vm.prank(accounts[0]);
        vm.expectRevert(
            abi.encodeWithSelector(HoprNetworkRegistry.NodeRegisterdToOtherAccount.selector, nodeAddresses[0])
        );
        hoprNetworkRegistry.selfDeregister(nodeAddresses);
        vm.clearMockedCalls();
    }

    /**
     * @dev Deregistering not registered nodes will not fail but no state gets updated
     */
    function test_DeregisterNonRegisteredNode() public {
        _helperRegisterAllNodeAddresses();
        address[] memory nodeAddresses = new address[](1);
        nodeAddresses[0] = vm.addr(20);

        // account is allowed to operate for node
        vm.mockCall(
            proxy, abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector), abi.encode(true)
        );
        assertEq(hoprNetworkRegistry.nodeRegisterdToAccount(nodeAddresses[0]), address(0));
        vm.prank(stakingAccounts[0]);
        hoprNetworkRegistry.selfDeregister(nodeAddresses);
        assertEq(hoprNetworkRegistry.nodeRegisterdToAccount(nodeAddresses[0]), address(0));
        vm.clearMockedCalls();
    }

    /**
     * @dev Deregister nodes by the staking account.
     */
    function testFuzz_DeregisterByItself(uint256 accountIndex) public {
        accountIndex = bound(accountIndex, 1, STAKING_ACCOUNTS_SIZE - 1);
        _helperRegisterAllNodeAddresses();

        address stakingAccount = stakingAccounts[accountIndex];
        address[] memory nodeAddresses = _helperCreateNodeAddresses(accountIndex);
        vm.prank(stakingAccount);

        for (uint256 index = 0; index < nodeAddresses.length; index++) {
            vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
            emit Deregistered(stakingAccount, nodeAddresses[index]);
        }
        vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
        emit EligibilityUpdated(stakingAccount, true);
        hoprNetworkRegistry.selfDeregister(nodeAddresses);

        vm.clearMockedCalls();
    }

    /**
     * @dev Register a node that has been registered to the caller. No revert but also no state update
     */
    function test_RegisterASelfRegisteredNode() public {
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances = [uint256(0), uint256(1), uint256(2), uint256(3), uint256(4)];
        // mock requirement function returns
        _helperMockProxyReturns(allowances);

        uint256 accountIndex = 4;
        address[] memory nodeAddresses = _helperCreateNodeAddresses(accountIndex);

        vm.startPrank(stakingAccounts[accountIndex]);
        hoprNetworkRegistry.selfRegister(nodeAddresses);
        // register again
        hoprNetworkRegistry.selfRegister(nodeAddresses);
        vm.stopPrank();
        vm.clearMockedCalls();
    }

    /**
     * @dev Register a node that has been registered to the caller
     */
    function testRevert_RegisterNodesRegisteredToOtherAccounts() public {
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances = [uint256(0), uint256(1), uint256(2), uint256(3), uint256(4)];
        // mock requirement function returns
        _helperMockProxyReturns(allowances);

        uint256 accountIndex = 1;
        address[] memory nodeAddresses = _helperCreateNodeAddresses(accountIndex);
        address[] memory oldAccounts = new address[](accountIndex);
        oldAccounts[0] = stakingAccounts[4];
        address[] memory accounts = new address[](accountIndex);
        accounts[0] = stakingAccounts[accountIndex];

        vm.prank(owner);
        hoprNetworkRegistry.managerRegister(oldAccounts, nodeAddresses);
        vm.prank(accounts[0]);
        vm.expectRevert(abi.encodeWithSelector(HoprNetworkRegistry.NodeAlreadyRegisterd.selector, nodeAddresses[0]));
        hoprNetworkRegistry.selfRegister(nodeAddresses);
        vm.clearMockedCalls();
    }

    /**
     * @dev check if an addres is registered and eligible
     */
    function testFuzz_NonRegisteredNodeisNeitherNodeRegisteredNorEligible(address nodeAddress) public {
        // registerd nodes are beyond their eligibility
        assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(nodeAddress));
    }
    /**
     * @dev Helper function to mock returns of `maxAllowedRegistrations` function on proxy contract
     * stakingAccounts[i] where i = 0..STAKING_ACCOUNTS_SIZE-1 has some random allowance value
     * for each stakingAccounts[i] where i = 1..STAKING_ACCOUNTS_SIZE-1, there are i nodes ranging
     * from vm.addr(i(i-1)/2+5) to vm.addr(i(i+1)/2+5) that can be operated by the staking accounts
     */

    function _helperMockProxyReturns(uint256[STAKING_ACCOUNTS_SIZE] memory allowances) internal {
        // mock the entire function
        vm.mockCall(
            proxy,
            abi.encodeWithSelector(IHoprNetworkRegistryRequirement.maxAllowedRegistrations.selector),
            abi.encode(0)
        );

        vm.mockCall(
            proxy, abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector), abi.encode(false)
        );

        for (uint256 i = 0; i < allowances.length; i++) {
            vm.mockCall(
                proxy,
                abi.encodeWithSelector(
                    IHoprNetworkRegistryRequirement.maxAllowedRegistrations.selector, stakingAccounts[i]
                ),
                abi.encode(allowances[i])
            );

            if (i == 0) {
                continue;
            }

            for (uint256 j = 0; j < i; j++) {
                vm.mockCall(
                    proxy,
                    abi.encodeWithSelector(
                        IHoprNetworkRegistryRequirement.canOperateFor.selector,
                        stakingAccounts[i],
                        vm.addr(i * (i - 1) / 2 + j + STAKING_ACCOUNTS_SIZE + 1)
                    ),
                    abi.encode(true)
                );
            }
        }
    }

    /**
     * @dev add create node addresses
     */
    function _helperCreateNodeAddresses(uint256 accountIndex) private pure returns (address[] memory) {
        address[] memory nodeAddresses = new address[](accountIndex);
        if (accountIndex > 0) {
            uint256 nodeStartingIndex = (accountIndex - 1) * accountIndex / 2 + STAKING_ACCOUNTS_SIZE + 1;

            for (uint256 index = 0; index < accountIndex; index++) {
                nodeAddresses[index] = vm.addr(nodeStartingIndex + index);
            }
        }
        return nodeAddresses;
    }

    /**
     * @dev register node addresses ranging from vm.addr(6) to vm.addr(15) to stakingAccounts
     */
    function _helperRegisterAllNodeAddresses() public {
        uint256[STAKING_ACCOUNTS_SIZE] memory allowances = [uint256(0), uint256(1), uint256(2), uint256(3), uint256(4)];
        // mock requirement function returns
        _helperMockProxyReturns(allowances);

        uint256 nodeStartingIndex = 1 + STAKING_ACCOUNTS_SIZE;
        for (uint256 j = 1; j < allowances.length; j++) {
            vm.prank(stakingAccounts[j]);
            address[] memory nodes = new address[](j);
            for (uint256 k = 0; k < j; k++) {
                nodes[k] = vm.addr(nodeStartingIndex);
                nodeStartingIndex++;
            }
            hoprNetworkRegistry.selfRegister(nodes);
        }
    }
}
