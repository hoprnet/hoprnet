// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import '../src/NetworkRegistry.sol';
import 'forge-std/Test.sol';

contract HoprNetworkRegistryTest is Test {
  // to alter the storage
  using stdStorage for StdStorage;

  HoprNetworkRegistry public hoprNetworkRegistry;
  address public proxy;
  address public owner;
  address[] public stakingAccounts;
  address[] public nodeAddresses;
  uint256 public constant STAKING_ACCOUNTS_SIZE = 5;
  // uint256[] public allowances;

  /**
   * Manually import the errors and events
   */
  event EnabledNetworkRegistry(bool indexed isEnabled); // Global toggle of the network registry
  event RequirementUpdated(address indexed requirementImplementation); // Emit when the network registry proxy is updated
  event Registered(address indexed stakingAccount, address indexed nodeAddress); // Emit when a node is included in the registry
  event Deregistered(address indexed stakingAccount, address indexed nodeAddress); // Emit when a node is removed from the registry
  event RegisteredByManager(address indexed stakingAccount, address indexed nodeAddress); // Emit when the contract owner register a node for an account
  event DeregisteredByManager(address indexed stakingAccount, address indexed nodeAddress); // Emit when the contract owner removes a node from the registry
  event EligibilityUpdated(address indexed stakingAccount, bool indexed eligibility); // Emit when the eligibility of an account is updated

  function setUp() public {
    proxy = vm.addr(100); // make vm.addr(100) requirementImplementation
    owner = vm.addr(101); // make address(101) new owner
    hoprNetworkRegistry = new HoprNetworkRegistry(proxy, owner);

    // create some unique staking account
    stakingAccounts = new address[](STAKING_ACCOUNTS_SIZE);
    for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
      stakingAccounts[i] = vm.addr(i+1);
    }
    // allowances = new uint256[](STAKING_ACCOUNTS_SIZE);
  }

  /**
   * @dev mock the return of proxy for created staking accounts
   */
  function testFuzz_MockProxyReturn(uint256[STAKING_ACCOUNTS_SIZE] memory allowances) public {
    _helperMockProxyReturns(allowances);

    for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
      (, bytes memory returndataAllowance) = proxy.staticcall(
        abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(i+1))
      );
      uint256 allowance = abi.decode(returndataAllowance, (uint256));
      assertEq(allowance, allowances[i]);
      // check eligibility
      assertTrue(hoprNetworkRegistry.isAccountEligible(stakingAccounts[i]));
    }

    vm.clearMockedCalls();
  }
  /**
   * @dev verify that return value of canOperateFor is correct
   */
  function testFuzz_MockCanOperateFor(uint256[STAKING_ACCOUNTS_SIZE] memory allowances, uint256 accountIndex) public {
    accountIndex = bound(accountIndex, 0, STAKING_ACCOUNTS_SIZE-1);
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
          abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector, stakingAccounts[accountIndex], nodeAddresses[j])
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
    emit EnabledNetworkRegistry(false);
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
    emit EnabledNetworkRegistry(true);
    hoprNetworkRegistry.enableRegistry();
  }

  /**
   * @dev Register contract for a single time:
   * it can self-register when the requirement is fulfilled and emits true
   */
  function testFuzz_SelfRegisterAddresses(uint256[STAKING_ACCOUNTS_SIZE] memory allowances, uint256 accountIndex) public {
    accountIndex = bound(accountIndex, 0, STAKING_ACCOUNTS_SIZE - 1);
    // ensure that the allowance at the index is suffiicent
    emit log_named_uint("allowances length", allowances.length);
    allowances[accountIndex] = accountIndex;
    emit log_named_uint("accountIndex", accountIndex);
    _helperMockProxyReturns(allowances);

    // ensure the staking account is created
    address stakingAccount = stakingAccounts[accountIndex];
    address[] memory nodeAddresses = _helperCreateNodeAddresses(accountIndex);

    // // read allowance
    // (bool successRead, bytes memory returndataAllowance) = proxy.staticcall(
    //   abi.encodeWithSelector(IHoprNetworkRegistryRequirement.maxAllowedRegistrations.selector, stakingAccount)
    // );
    // // only continue when a value is returned
    // assertTrue(successRead);
    // uint256 allowance = abi.decode(returndataAllowance, (uint256));
    // // when it's possible to register
    // vm.assume(allowance > 0);

    vm.prank(stakingAccount);
    for (uint256 i = 0; i < nodeAddresses.length; i++) {
    emit log_named_uint("expect emit Registered", i);
      vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
      emit Registered(stakingAccount, nodeAddresses[i]);
    }
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(stakingAccount, true);
    hoprNetworkRegistry.selfRegister(nodeAddresses);
    vm.clearMockedCalls();
  }

  

  /**
   * @dev Register contract for a single time:
   * it cannot self-register when trying to register more than the limit
   * it cannot self-register when the requirement is not fulfilled
   */
  function testRevert_SelfRegisterTooManyAddresses(uint256[STAKING_ACCOUNTS_SIZE] memory allowances, uint256 accountIndex) public {
    accountIndex = bound(accountIndex, 0, STAKING_ACCOUNTS_SIZE - 1);
    // ensure that the allowance at the index is suffiicent
    emit log_named_uint("allowances length", allowances.length);
    vm.assume(allowances[accountIndex]  < accountIndex);
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
   * @dev Manager can register safe and node
   */
  function test_ManagerRegister() public {
    address[] memory nodeAddresses = _helperCreateNodeAddresses(STAKING_ACCOUNTS_SIZE);

    vm.prank(owner);
    for (uint256 i = 0; i < stakingAccounts.length; i++) {
      vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
      emit RegisteredByManager(stakingAccounts[i], nodeAddresses[i]);
    }
    hoprNetworkRegistry.managerRegister(stakingAccounts, nodeAddresses);
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
        startingIndex ++;
      }
    }
    hoprNetworkRegistry.managerDeregister(nodes);

    uint256 startingInd;
    for (uint256 p = 0; p < stakingAccounts.length; p++) {
      assertEq(hoprNetworkRegistry.countRegisterdNodesPerAccount(stakingAccounts[p]), 0);
      for (uint256 q = 0; q < p; q++) {
        assertEq(hoprNetworkRegistry.nodeRegisterdToAccount(nodes[startingInd]), address(0));
        startingInd ++;
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
      stdstore
        .target(address(hoprNetworkRegistry))
        .sig("countRegisterdNodesPerAccount(address)")
        .with_key(stakingAccounts[i])
        .depth(0)
        .checked_write(2);
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
      stdstore
        .target(address(hoprNetworkRegistry))
        .sig("countRegisterdNodesPerAccount(address)")
        .with_key(stakingAccounts[i])
        .depth(0)
        .checked_write(2);
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

    // mock the number of curent registered nodes
    for (uint256 i = 0; i < STAKING_ACCOUNTS_SIZE; i++) {
      eligibilities[i] = i % 2 == 0 ? true : false;
    }
     
    vm.prank(owner);
    for (uint256 j = 0; j < STAKING_ACCOUNTS_SIZE; j++) {
      vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
      emit EligibilityUpdated(stakingAccounts[j], eligibilities[j]);
    }
    hoprNetworkRegistry.managerForceSync(stakingAccounts, eligibilities);
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
   * @dev Fail to deregister an non-registered account
   */
  function testRevert_DeregisterForOtherAccount() public {
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
   * @dev Deregistering not registered nodes will not fail but no state gets updated
   */
  function test_DeregisterNonRegisteredNode() public {
    _helperRegisterAllNodeAddresses();
    address nodeAddress = vm.addr(20);

    assertEq(hoprNetworkRegistry.nodeRegisterdToAccount(nodeAddress), address(0));
    vm.prank(stakingAccounts[0]);
    hoprNetworkRegistry.selfDeregister(nodeAddresses);
    assertEq(hoprNetworkRegistry.nodeRegisterdToAccount(nodeAddress), address(0));
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

  // /**
  //  * @dev Register contract for multiple times by one
  //  * it fails to register the node address by a different account
  //  */
  // function testRevert_RegisterARegisteredNode() public {
  //   uint256 accountIndex = 1;
  //   string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

  //   vm.prank(vm.addr(accountIndex + 1));
  //   vm.expectRevert('HoprNetworkRegistry: Cannot link a registered node.');
  //   hoprNetworkRegistry.selfRegister(nodeIds);
  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev Register contract for multiple times by one
  //  * it can register an additional peer ID
  //  */
  // function test_RegisterAnotherNode() public {
  //   _helperMockProxyReturns();
  //   uint256 accountIndex = 1;

  //   vm.prank(vm.addr(accountIndex));

  //   string[] memory nodeAddresses = new string[](1);
  //   nodeAddresses[0] = HOPR_NODE_ADDRESSES[accountIndex];

  //   vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
  //   emit EligibilityUpdated(vm.addr(accountIndex), true);
  //   vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
  //   emit Registered(vm.addr(accountIndex), nodeAddresses[0]);
  //   hoprNetworkRegistry.selfRegister(nodeAddresses);

  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev Register contract for multiple times by one
  //  * it self-registered account emits true when the requirement is fulfilled, but no longer emits Registered event
  //  */
  // function test_RegisterAgainTheSameNode() public {
  //   uint256 accountIndex = 1;
  //   string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

  //   vm.prank(vm.addr(accountIndex));

  //   vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
  //   emit EligibilityUpdated(vm.addr(accountIndex), true);
  //   hoprNetworkRegistry.selfRegister(nodeIds);

  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev Register contract for multiple times by one
  //  * it fails self-registered when the requirement is not fulfilled
  //  */
  // function testRevert_WhenEligibilityChangesRegisterAnotherNode() public {
  //   uint256 accountIndex = 1;
  //   string[] memory nodeIds = _helperRegisterOneNode(accountIndex);
  //   string[] memory nodeAddresses = new string[](1);
  //   nodeAddresses[0] = HOPR_NODE_ADDRESSES[accountIndex];

  //   // update the eligibility
  //   vm.mockCall(
  //     proxy,
  //     abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(accountIndex)),
  //     abi.encode(1)
  //   );

  //   // the same account cannot register another node
  //   vm.prank(vm.addr(accountIndex));
  //   vm.expectRevert('HoprNetworkRegistry: selfRegister reaches limit, cannot register requested nodes.');
  //   hoprNetworkRegistry.selfRegister(nodeAddresses);
  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev Force emit an eligibility update
  //  * it allows owner to force emit an eligibility update
  //  */
  // function test_OwnerForceUpdateEligibility() public {
  //   _helperMockProxyReturns();

  //   address[] memory participantAddresses = new address[](3);
  //   participantAddresses[0] = vm.addr(1);
  //   participantAddresses[1] = vm.addr(3);
  //   participantAddresses[2] = vm.addr(5);
  //   bool[] memory eligibility = new bool[](3);
  //   eligibility[0] = false;
  //   eligibility[1] = true;
  //   eligibility[2] = true;

  //   vm.prank(owner);

  //   vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
  //   emit EligibilityUpdated(participantAddresses[0], eligibility[0]);
  //   vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
  //   emit EligibilityUpdated(participantAddresses[1], eligibility[1]);
  //   vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
  //   emit EligibilityUpdated(participantAddresses[2], eligibility[2]);
  //   hoprNetworkRegistry.ownerForceEligibility(participantAddresses, eligibility);

  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev Sync with when criteria change
  //  * it allows owner to sync the criteria, before criteria change
  //  * it allows anyone to check account and node eligibility
  //  */
  // function test_OwnerSyncBeforeChange() public {
  //   uint256 accountIndex = 3;
  //   string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

  //   string[] memory nodeAddresses = new string[](3);
  //   nodeAddresses[0] = HOPR_NODE_ADDRESSES[1];
  //   nodeAddresses[1] = nodeIds[0];
  //   nodeAddresses[2] = HOPR_NODE_ADDRESSES[5];

  //   vm.prank(owner);

  //   vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
  //   emit EligibilityUpdated(vm.addr(accountIndex), true);
  //   hoprNetworkRegistry.sync(nodeAddresses);

  //   // on registered eligible accounts
  //   assertTrue(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(accountIndex)));
  //   assertTrue(hoprNetworkRegistry.isNodeRegisteredAndEligible(nodeIds[0]));
  //   // on non-registered eligible accounts
  //   assertFalse(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(1)));
  //   assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(HOPR_NODE_ADDRESSES[1]));
  //   // on non-registered ineligible accounts
  //   assertFalse(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(5)));
  //   assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(HOPR_NODE_ADDRESSES[5]));

  //   vm.clearMockedCalls();
  // }

  // /**
  //  * @dev Sync with when criteria change
  //  * it allows owner to sync the criteria, after criteria change
  //  * it allows anyone to check account and node eligibility
  //  */
  // function test_OwnerSyncAfterChange() public {
  //   uint256 accountIndex = 3;
  //   string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

  //   string[] memory nodeAddresses = new string[](3);
  //   nodeAddresses[0] = HOPR_NODE_ADDRESSES[1];
  //   nodeAddresses[1] = nodeIds[0];
  //   nodeAddresses[2] = HOPR_NODE_ADDRESSES[5];

  //   // update the eligibility
  //   vm.mockCall(
  //     proxy,
  //     abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(accountIndex)),
  //     abi.encode(0)
  //   );

  //   vm.prank(owner);

  //   vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
  //   emit EligibilityUpdated(vm.addr(accountIndex), false);
  //   hoprNetworkRegistry.sync(nodeAddresses);

  //   // on registered, now ineligible accounts
  //   assertFalse(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(accountIndex)));
  //   assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(nodeIds[0]));

  //   vm.clearMockedCalls();
  // }

  /**
   *@dev Helper function to mock returns of `maxAllowedRegistrations` function on proxy contract
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
        proxy,
        abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector),
        abi.encode(false)
    );

    for (uint256 i = 0; i < allowances.length; i++) {
      vm.mockCall(
        proxy,
        abi.encodeWithSelector(IHoprNetworkRegistryRequirement.maxAllowedRegistrations.selector, stakingAccounts[i]),
        abi.encode(allowances[i])
      );

      if (i == 0) {
        continue;
      }

      for (uint256 j = 0; j < i; j++) {
        vm.mockCall(
          proxy,
          abi.encodeWithSelector(IHoprNetworkRegistryRequirement.canOperateFor.selector, stakingAccounts[i], vm.addr(i * (i - 1) / 2 + j + STAKING_ACCOUNTS_SIZE + 1)),
          abi.encode(true)
        );
      }
    }
  }

  /**
   *@dev add create node addresses
   */
  function _helperCreateNodeAddresses(uint256 accountIndex) private returns (address[] memory) {
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
        nodeStartingIndex ++;
      }
      hoprNetworkRegistry.selfRegister(nodes);
    }
  }

//   /**
//    *@dev Helper function to mock returns of `maxAllowedRegistrations` function on proxy contract
//    */
//   function _helperMockProxyReturns() internal {
//     // account vm.addr(1) and vm.addr(2) have max allowance
//     vm.mockCall(
//       proxy,
//       abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(1)),
//       abi.encode(type(uint256).max)
//     );
//     vm.mockCall(
//       proxy,
//       abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(2)),
//       abi.encode(type(uint256).max)
//     );
//     // account vm.addr(3) and vm.addr(4) have 1 allowance
//     vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(3)), abi.encode(1));
//     vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(4)), abi.encode(1));
//     // account vm.addr(5) and vm.addr(6) have 0 allowance
//     vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(5)), abi.encode(0));
//     vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(6)), abi.encode(0));
//   }

//   /**
//    * @dev helper function to reigster an account with a node. Note that
//    * accountIndex should be between 1 and 6
//    */
//   function _helperRegisterOneNode(uint256 accountIndex) internal returns (string[] memory) {
//     _helperMockProxyReturns();

//     address[] memory participantAddresses = new address[](1);
//     participantAddresses[0] = vm.addr(accountIndex);
//     string[] memory nodeAddresses = new string[](1);
//     nodeAddresses[0] = HOPR_NODE_ADDRESSES[accountIndex - 1];

//     vm.prank(participantAddresses[0]);
//     hoprNetworkRegistry.selfRegister(nodeAddresses);

//     return nodeAddresses;
//   }

// /**
//  * @dev return 
//  */
//   function _helperMockAllowedRegistrations(adress stakingAccountSize, uint256 nodeSize) private {
    
//   }
}
