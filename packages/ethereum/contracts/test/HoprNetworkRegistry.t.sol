// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.6.0 <0.9.0;

import '../src/HoprNetworkRegistry.sol';
import 'forge-std/Test.sol';

contract HoprNetworkRegistryTest is Test {
  // to alter the storage
  using stdStorage for StdStorage;

  HoprNetworkRegistry public hoprNetworkRegistry;
  address public proxy;
  address public owner;
  string[] public HOPR_NODE_ADDRESSES = [
    '16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x0',
    '16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1',
    '16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x2',
    '16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x3',
    '16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x4',
    '16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x5'
  ];

  /**
   * Manually import the errors and events
   */
  error InvalidPeerId(string peerId);

  event EnabledNetworkRegistry(bool indexed isEnabled); // Global toggle of the network registry
  event RequirementUpdated(address indexed requirementImplementation); // Emit when the network registry proxy is updated
  event Registered(address indexed account, string hoprPeerId); // Emit when an account register a node peer id for itself
  event Deregistered(address indexed account, string hoprPeerId); // Emit when an account deregister a node peer id for itself
  event RegisteredByOwner(address indexed account, string hoprPeerId); // Emit when the contract owner register a node peer id for an account
  event DeregisteredByOwner(address indexed account, string hoprPeerId); // Emit when the contract owner deregister a node peer id for an account
  event EligibilityUpdated(address indexed account, bool indexed eligibility); // Emit when the eligibility of an account is updated

  function setUp() public {
    proxy = vm.addr(100); // make vm.addr(100) requirementImplementation
    owner = vm.addr(101); // make address(101) new owner
    hoprNetworkRegistry = new HoprNetworkRegistry(proxy, owner);
  }

  function testFuzz_MockProxyReturn() public {
    _helperMockProxyReturns();
    (, bytes memory returndataAllowance0) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(1))
    );
    (, bytes memory returndataAllowance1) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(2))
    );
    (, bytes memory returndataAllowance2) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(3))
    );
    (, bytes memory returndataAllowance3) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(4))
    );
    (, bytes memory returndataAllowance4) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(5))
    );
    (, bytes memory returndataAllowance5) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(6))
    );
    uint256 allowance0 = abi.decode(returndataAllowance0, (uint256));
    uint256 allowance1 = abi.decode(returndataAllowance1, (uint256));
    uint256 allowance2 = abi.decode(returndataAllowance2, (uint256));
    uint256 allowance3 = abi.decode(returndataAllowance3, (uint256));
    uint256 allowance4 = abi.decode(returndataAllowance4, (uint256));
    uint256 allowance5 = abi.decode(returndataAllowance5, (uint256));
    assertEq(allowance0, type(uint256).max);
    assertEq(allowance1, type(uint256).max);
    assertEq(allowance2, 1);
    assertEq(allowance3, 1);
    assertEq(allowance4, 0);
    assertEq(allowance5, 0);
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it is enabled globally
   */
  function test_IsEnabled() public {
    _helperMockProxyReturns();
    assertTrue(hoprNetworkRegistry.enabled());
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it allows owner to update the registry
   */
  function testFuzz_OwnerUpdateRegistry(address newImp) public {
    _helperMockProxyReturns();
    vm.prank(owner);
    vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
    emit RequirementUpdated(newImp);
    hoprNetworkRegistry.updateRequirementImplementation(newImp);
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to update the registry by a non-owner account
   */
  function testRevert_WhenNotOwnerUpdateRegistry(address caller) public {
    vm.assume(caller != owner);
    _helperMockProxyReturns();
    vm.prank(caller);
    vm.expectRevert('Ownable: caller is not the owner');
    hoprNetworkRegistry.updateRequirementImplementation(address(0));
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to enable the registry by a non-owner account
   */
  function testRevert_WhenNotOwnerEnableRegistry(address caller) public {
    vm.assume(caller != owner);
    _helperMockProxyReturns();
    vm.prank(caller);
    vm.expectRevert('Ownable: caller is not the owner');
    hoprNetworkRegistry.enableRegistry();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to enable an enabled registry by its owner
   */
  function testRevert_WhenEnablingAnEnabledRegistry() public {
    _helperMockProxyReturns();
    vm.prank(owner);
    vm.expectRevert('HoprNetworkRegistry: Registry is enabled');
    hoprNetworkRegistry.enableRegistry();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it allows owner to disable the registry
   */
  function test_OwnerDisableRegistry() public {
    _helperMockProxyReturns();
    vm.prank(owner);
    vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
    emit EnabledNetworkRegistry(false);
    hoprNetworkRegistry.disableRegistry();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it fails to enable an enabled registry by its owner
   */
  function testRevert_WhenDisablingADisnabledRegistry() public {
    _helperMockProxyReturns();
    vm.startPrank(owner);
    hoprNetworkRegistry.disableRegistry();

    vm.expectRevert('HoprNetworkRegistry: Registry is disabled');
    hoprNetworkRegistry.disableRegistry();

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner can update important parameters of the contract:
   * it allows owner to enable an enabled registry
   */
  function test_OwnerEnsableRegistry() public {
    _helperMockProxyReturns();
    vm.startPrank(owner);
    hoprNetworkRegistry.disableRegistry();

    vm.expectEmit(true, false, false, false, address(hoprNetworkRegistry));
    emit EnabledNetworkRegistry(true);
    hoprNetworkRegistry.enableRegistry();

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for a single time:
   * it can self-register when the requirement is fulfilled and emits true
   */
  function test_SelfRegisterAddresses(
    uint256 accountIndex,
    uint256 addressIndex1,
    uint256 addressIndex2
  ) public {
    _helperMockProxyReturns();

    // ensure register accounts' allowance is above 1
    accountIndex = bound(accountIndex, 1, 2);
    vm.assume(accountIndex > 0);
    // read allowance
    (bool successRead, bytes memory returndataAllowance) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(accountIndex))
    );
    // only continue when a value is returned
    vm.assume(successRead);
    uint256 allowance = abi.decode(returndataAllowance, (uint256));
    // when it's possible to register
    vm.assume(allowance > 1);

    addressIndex1 = bound(addressIndex1, 0, 5);
    addressIndex2 = bound(addressIndex2, 0, 5);
    vm.assume(addressIndex1 != addressIndex2);
    string[] memory nodeAddresses = new string[](2);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[addressIndex1];
    nodeAddresses[1] = HOPR_NODE_ADDRESSES[addressIndex2];

    vm.prank(vm.addr(accountIndex));
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(vm.addr(accountIndex), true);
    vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
    emit Registered(vm.addr(accountIndex), nodeAddresses[0]);
    vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
    emit Registered(vm.addr(accountIndex), nodeAddresses[1]);
    hoprNetworkRegistry.selfRegister(nodeAddresses);
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for a single time:
   * it cannot self-register when trying to register more than the limit
   * it cannot self-register when the requirement is not fulfilled
   */
  function testRevert_SelfRegisterTooManyAddresses(uint256 accountIndex) public {
    _helperMockProxyReturns();

    // read registration allowance
    accountIndex = bound(accountIndex, 1, 6);
    (bool successRead, bytes memory returndataAllowance) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(accountIndex))
    );
    vm.assume(successRead);
    uint256 allowance = abi.decode(returndataAllowance, (uint256));

    // ensure that the allowance is below the amount in the registration attempt
    vm.assume(allowance < 3);
    string[] memory nodeAddresses = new string[](3);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[0];
    nodeAddresses[1] = HOPR_NODE_ADDRESSES[1];
    nodeAddresses[2] = HOPR_NODE_ADDRESSES[2];

    vm.prank(vm.addr(accountIndex));
    vm.expectRevert('HoprNetworkRegistry: selfRegister reaches limit, cannot register requested nodes.');
    hoprNetworkRegistry.selfRegister(nodeAddresses);
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for a single time:
   * it fails to register when hopr node address is empty
   * it fails to register when hopr node address of wrong length
   * it fails to register when hopr node address is of the right length but with wrong prefix
   */
  function testRevert_SelfRegisterWithInvalidNodeAddresses(uint256 accountIndex) public {
    _helperMockProxyReturns();

    // read registration allowance
    accountIndex = bound(accountIndex, 1, 6);
    (bool successRead, bytes memory returndataAllowance) = proxy.staticcall(
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(accountIndex))
    );
    vm.assume(successRead);
    uint256 allowance = abi.decode(returndataAllowance, (uint256));

    // ensure that the allowance is above zero
    vm.assume(allowance > 0);
    string[] memory nodeAddresses = new string[](1);

    vm.startPrank(vm.addr(accountIndex));
    uint256 snapshotBeforeRegister = vm.snapshot();

    // node address is empty
    nodeAddresses[0] = '';
    vm.expectRevert(abi.encodeWithSelector(HoprNetworkRegistry.InvalidPeerId.selector, nodeAddresses[0]));
    hoprNetworkRegistry.selfRegister(nodeAddresses);

    // node address has wrong length
    vm.revertTo(snapshotBeforeRegister);
    nodeAddresses[0] = '16Uiu2HA';
    vm.expectRevert(abi.encodeWithSelector(HoprNetworkRegistry.InvalidPeerId.selector, nodeAddresses[0]));
    hoprNetworkRegistry.selfRegister(nodeAddresses);

    // node address has wrong prefix
    vm.revertTo(snapshotBeforeRegister);
    nodeAddresses[0] = '0xUiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x0';
    vm.expectRevert(abi.encodeWithSelector(HoprNetworkRegistry.InvalidPeerId.selector, nodeAddresses[0]));
    hoprNetworkRegistry.selfRegister(nodeAddresses);

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for a single time:
   * it fails to register by the owner when array length does not match
   */
  function testRevert_OwnerRegisterWithWrongArrayLengths() public {
    _helperMockProxyReturns();

    vm.startPrank(owner);

    address[] memory participantAddresses = new address[](1);
    participantAddresses[0] = vm.addr(10);
    string[] memory nodeAddresses = new string[](2);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[0];
    nodeAddresses[1] = HOPR_NODE_ADDRESSES[1];

    // node address is empty
    vm.expectRevert('HoprNetworkRegistry: hoprPeerIdes and accounts lengths mismatch');
    hoprNetworkRegistry.ownerRegister(participantAddresses, nodeAddresses);

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for a single time:
   * it can register by the owner and emit RegisteredByOwner
   */
  function test_OwnerRegister(uint256 index1, uint256 index2) public {
    _helperMockProxyReturns();
    index1 = bound(index1, 1, 6);
    index2 = bound(index2, 1, 6);

    vm.startPrank(owner);

    address[] memory participantAddresses = new address[](2);
    participantAddresses[0] = vm.addr(index1);
    participantAddresses[1] = vm.addr(index2);
    string[] memory nodeAddresses = new string[](2);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[index1 - 1];
    nodeAddresses[1] = HOPR_NODE_ADDRESSES[index2 - 1];

    vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
    emit RegisteredByOwner(participantAddresses[0], nodeAddresses[0]);
    vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
    emit RegisteredByOwner(participantAddresses[1], nodeAddresses[1]);
    hoprNetworkRegistry.ownerRegister(participantAddresses, nodeAddresses);

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for a single time:
   * it can be deregistered by the owner when a node was not registered. Nothing gets emitted
   */
  function test_OwnerDeregisterAnNonRegisteredNode(uint256 nodeIndex) public {
    _helperMockProxyReturns();
    nodeIndex = bound(nodeIndex, 0, 5);

    assertEq(hoprNetworkRegistry.nodePeerIdToAccount(HOPR_NODE_ADDRESSES[nodeIndex]), address(0));

    string[] memory nodesToDeregister = new string[](1);
    nodesToDeregister[0] = HOPR_NODE_ADDRESSES[nodeIndex];

    vm.prank(owner);
    hoprNetworkRegistry.ownerDeregister(nodesToDeregister);
    vm.clearMockedCalls();

    assertEq(hoprNetworkRegistry.nodePeerIdToAccount(HOPR_NODE_ADDRESSES[nodeIndex]), address(0));
  }

  /**
   * @dev Register contract for a single time:
   * it can be deregistered by the owner when an address was registered
   */
  function test_OwnerDeregisterAnRegisteredNode(uint256 index) public {
    _helperMockProxyReturns();
    index = bound(index, 1, 6);

    address[] memory participantAddresses = new address[](1);
    participantAddresses[0] = vm.addr(index);
    string[] memory nodeAddresses = new string[](1);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[index - 1];

    vm.startPrank(owner);
    hoprNetworkRegistry.ownerRegister(participantAddresses, nodeAddresses);

    assertEq(hoprNetworkRegistry.nodePeerIdToAccount(nodeAddresses[0]), participantAddresses[0]);

    vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
    emit DeregisteredByOwner(participantAddresses[0], nodeAddresses[0]);
    hoprNetworkRegistry.ownerDeregister(nodeAddresses);
    assertEq(hoprNetworkRegistry.nodePeerIdToAccount(nodeAddresses[0]), address(0));

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner force update eligibility: (with addr(1) and addr(5) registered)
   * it can force update eligibility of an account independantly (true), and sync back to its actual eligibility (false)
   * it can force update eligibility of an account independantly (false), and sync back to its actual eligibility (false)
   */
  function test_OwnerUpdateAndSyncIneligibleAccount() public {
    _helperMockProxyReturns();
    uint256 index = 5;
    uint256 snapshot;

    address[] memory participantAddresses = new address[](1);
    participantAddresses[0] = vm.addr(index);
    string[] memory nodeAddresses = new string[](1);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[index - 1];
    bool[] memory trueEligibility = new bool[](1);
    trueEligibility[0] = true;
    bool[] memory falseEligibility = new bool[](1);
    falseEligibility[0] = false;

    vm.startPrank(owner);
    hoprNetworkRegistry.ownerRegister(participantAddresses, nodeAddresses);
    snapshot = vm.snapshot();

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], true);
    hoprNetworkRegistry.ownerForceEligibility(participantAddresses, trueEligibility);
    vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], false);
    hoprNetworkRegistry.sync(nodeAddresses);

    vm.revertTo(snapshot);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], false);
    hoprNetworkRegistry.ownerForceEligibility(participantAddresses, falseEligibility);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], false);
    hoprNetworkRegistry.sync(nodeAddresses);

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Owner force update eligibility: (with addr(1) and addr(5) registered)
   * it can force update eligibility of an account independantly (true), and sync back to its actual eligibility (true)
   * it can force update eligibility of an account independantly (false), and sync back to its actual eligibility (true)
   */
  function test_OwnerUpdateAndSyncEligibleAccount() public {
    _helperMockProxyReturns();
    uint256 index = 1;
    uint256 snapshot;

    address[] memory participantAddresses = new address[](1);
    participantAddresses[0] = vm.addr(index);
    string[] memory nodeAddresses = new string[](1);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[index - 1];
    bool[] memory trueEligibility = new bool[](1);
    trueEligibility[0] = true;
    bool[] memory falseEligibility = new bool[](1);
    falseEligibility[0] = false;

    vm.startPrank(owner);
    hoprNetworkRegistry.ownerRegister(participantAddresses, nodeAddresses);
    snapshot = vm.snapshot();

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], true);
    hoprNetworkRegistry.ownerForceEligibility(participantAddresses, trueEligibility);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], true);
    hoprNetworkRegistry.sync(nodeAddresses);

    vm.revertTo(snapshot);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], false);
    hoprNetworkRegistry.ownerForceEligibility(participantAddresses, falseEligibility);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], true);
    hoprNetworkRegistry.sync(nodeAddresses);

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for multiple times by one
   * it fails to deregister an non-registered account. Panic when the deregister originial account has zero registered node
   */
  function testFail_DeregisterForOtherAccount() public {
    uint256 accountIndex = 1;
    string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

    vm.prank(vm.addr(accountIndex + 1));
    hoprNetworkRegistry.selfDeregister(nodeIds);
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for multiple times by one
   * it fails to deregister an non-registered node.
   */
  function testRevert_DeregisterOtherNode() public {
    _helperMockProxyReturns();
    uint256 accountIndex = 1;
    string[] memory nodeAddresses = new string[](1);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[accountIndex];

    vm.startPrank(vm.addr(accountIndex));
    // when countRegisterdNodesPerAccount(caller) is smaller than the nodeAddresses array length
    vm.expectRevert(stdError.arithmeticError);
    hoprNetworkRegistry.selfDeregister(nodeAddresses);

    // when there are enough registered nodes but none of them matches with the provided nodes
    vm.store(
      address(hoprNetworkRegistry),
      bytes32(
        stdstore
          .target(address(hoprNetworkRegistry))
          .sig('countRegisterdNodesPerAccount(address)')
          .with_key(vm.addr(accountIndex))
          .find()
      ),
      bytes32(abi.encode(1))
    );
    vm.expectRevert('HoprNetworkRegistry: Cannot delete an entry not associated with the caller.');
    hoprNetworkRegistry.selfDeregister(nodeAddresses);
    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for multiple times by one
   * it can deregister by itself.
   */
  function testRevert_DeregisterByItself() public {
    uint256 accountIndex = 1;
    string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

    vm.prank(vm.addr(accountIndex));

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(vm.addr(accountIndex), true);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit Deregistered(vm.addr(accountIndex), nodeIds[0]);
    hoprNetworkRegistry.selfDeregister(nodeIds);

    vm.stopPrank();
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for multiple times by one
   * it fails to register the node address by a different account
   */
  function testRevert_RegisterARegisteredNode() public {
    uint256 accountIndex = 1;
    string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

    vm.prank(vm.addr(accountIndex + 1));
    vm.expectRevert('HoprNetworkRegistry: Cannot link a registered node.');
    hoprNetworkRegistry.selfRegister(nodeIds);
    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for multiple times by one
   * it can register an additional peer ID
   */
  function test_RegisterAnotherNode() public {
    _helperMockProxyReturns();
    uint256 accountIndex = 1;

    vm.prank(vm.addr(accountIndex));

    string[] memory nodeAddresses = new string[](1);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[accountIndex];

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(vm.addr(accountIndex), true);
    vm.expectEmit(true, false, false, true, address(hoprNetworkRegistry));
    emit Registered(vm.addr(accountIndex), nodeAddresses[0]);
    hoprNetworkRegistry.selfRegister(nodeAddresses);

    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for multiple times by one
   * it self-registered account emits true when the requirement is fulfilled, but no longer emits Registered event
   */
  function test_RegisterAgainTheSameNode() public {
    uint256 accountIndex = 1;
    string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

    vm.prank(vm.addr(accountIndex));

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(vm.addr(accountIndex), true);
    hoprNetworkRegistry.selfRegister(nodeIds);

    vm.clearMockedCalls();
  }

  /**
   * @dev Register contract for multiple times by one
   * it fails self-registered when the requirement is not fulfilled
   */
  function testRevert_WhenEligibilityChangesRegisterAnotherNode() public {
    uint256 accountIndex = 1;
    string[] memory nodeIds = _helperRegisterOneNode(accountIndex);
    string[] memory nodeAddresses = new string[](1);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[accountIndex];

    // update the eligibility
    vm.mockCall(
      proxy,
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(accountIndex)),
      abi.encode(1)
    );

    // the same account cannot register another node
    vm.prank(vm.addr(accountIndex));
    vm.expectRevert('HoprNetworkRegistry: selfRegister reaches limit, cannot register requested nodes.');
    hoprNetworkRegistry.selfRegister(nodeAddresses);
    vm.clearMockedCalls();
  }

  /**
   * @dev Force emit an eligibility update
   * it allows owner to force emit an eligibility update
   */
  function test_OwnerForceUpdateEligibility() public {
    _helperMockProxyReturns();

    address[] memory participantAddresses = new address[](3);
    participantAddresses[0] = vm.addr(1);
    participantAddresses[1] = vm.addr(3);
    participantAddresses[2] = vm.addr(5);
    bool[] memory eligibility = new bool[](3);
    eligibility[0] = false;
    eligibility[1] = true;
    eligibility[2] = true;

    vm.prank(owner);

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[0], eligibility[0]);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[1], eligibility[1]);
    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(participantAddresses[2], eligibility[2]);
    hoprNetworkRegistry.ownerForceEligibility(participantAddresses, eligibility);

    vm.clearMockedCalls();
  }

  /**
   * @dev Sync with when criteria change
   * it allows owner to sync the criteria, before criteria change
   * it allows anyone to check account and node eligibility
   */
  function test_OwnerSyncBeforeChange() public {
    uint256 accountIndex = 3;
    string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

    string[] memory nodeAddresses = new string[](3);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[1];
    nodeAddresses[1] = nodeIds[0];
    nodeAddresses[2] = HOPR_NODE_ADDRESSES[5];

    vm.prank(owner);

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(vm.addr(accountIndex), true);
    hoprNetworkRegistry.sync(nodeAddresses);

    // on registered eligible accounts
    assertTrue(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(accountIndex)));
    assertTrue(hoprNetworkRegistry.isNodeRegisteredAndEligible(nodeIds[0]));
    // on non-registered eligible accounts
    assertFalse(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(1)));
    assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(HOPR_NODE_ADDRESSES[1]));
    // on non-registered ineligible accounts
    assertFalse(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(5)));
    assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(HOPR_NODE_ADDRESSES[5]));

    vm.clearMockedCalls();
  }

  /**
   * @dev Sync with when criteria change
   * it allows owner to sync the criteria, after criteria change
   * it allows anyone to check account and node eligibility
   */
  function test_OwnerSyncAfterChange() public {
    uint256 accountIndex = 3;
    string[] memory nodeIds = _helperRegisterOneNode(accountIndex);

    string[] memory nodeAddresses = new string[](3);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[1];
    nodeAddresses[1] = nodeIds[0];
    nodeAddresses[2] = HOPR_NODE_ADDRESSES[5];

    // update the eligibility
    vm.mockCall(
      proxy,
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(accountIndex)),
      abi.encode(0)
    );

    vm.prank(owner);

    vm.expectEmit(true, true, false, false, address(hoprNetworkRegistry));
    emit EligibilityUpdated(vm.addr(accountIndex), false);
    hoprNetworkRegistry.sync(nodeAddresses);

    // on registered, now ineligible accounts
    assertFalse(hoprNetworkRegistry.isAccountRegisteredAndEligible(vm.addr(accountIndex)));
    assertFalse(hoprNetworkRegistry.isNodeRegisteredAndEligible(nodeIds[0]));

    vm.clearMockedCalls();
  }

  /**
   *@dev Helper function to mock returns of `maxAllowedRegistrations` function on proxy contract
   */
  function _helperMockProxyReturns() internal {
    // account vm.addr(1) and vm.addr(2) have max allowance
    vm.mockCall(
      proxy,
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(1)),
      abi.encode(type(uint256).max)
    );
    vm.mockCall(
      proxy,
      abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(2)),
      abi.encode(type(uint256).max)
    );
    // account vm.addr(3) and vm.addr(4) have 1 allowance
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(3)), abi.encode(1));
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(4)), abi.encode(1));
    // account vm.addr(5) and vm.addr(6) have 0 allowance
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(5)), abi.encode(0));
    vm.mockCall(proxy, abi.encodeWithSignature('maxAllowedRegistrations(address)', vm.addr(6)), abi.encode(0));
  }

  /**
   * @dev helper function to reigster an account with a node. Note that
   * accountIndex should be between 1 and 6
   */
  function _helperRegisterOneNode(uint256 accountIndex) internal returns (string[] memory) {
    _helperMockProxyReturns();

    address[] memory participantAddresses = new address[](1);
    participantAddresses[0] = vm.addr(accountIndex);
    string[] memory nodeAddresses = new string[](1);
    nodeAddresses[0] = HOPR_NODE_ADDRESSES[accountIndex - 1];

    vm.prank(participantAddresses[0]);
    hoprNetworkRegistry.selfRegister(nodeAddresses);

    return nodeAddresses;
  }
}
