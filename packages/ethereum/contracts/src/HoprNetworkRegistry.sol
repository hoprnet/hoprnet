// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import 'openzeppelin-contracts-4.8.3/access/AccessControlEnumerable.sol';
import './IHoprNetworkRegistryRequirement.sol';

/**
 * @title HoprNetworkRegistry
 * @dev Smart contract that maintains a list of hopr node address that are allowed
 * to enter HOPR network. 
 * Eligibility of whether a node address can be registered is checked through
 * `IHoprNetworkRegistryRequirement`.
 *
 * When reaching its limits, accounts can remove registered node addresses (`deregister`)
 * before adding more.
 *
 * A node address can only be registered if it's not registered by another account.
 *
 * This network registry can be globally enabled/disabled by the owner
 *
 * Implementation of `IHoprNetworkRegistryRequirement` can also be dynamically updated by the
 * owner. Some sample implementations can be found under../proxy/ folder
 *
 * Owner has the power to overwrite the registration
 */
contract HoprNetworkRegistry is AccessControlEnumerable {
  bytes32 public constant MANAGER_ROLE = keccak256('MANAGER_ROLE');

  IHoprNetworkRegistryRequirement public requirementImplementation; // Implementation of network registry proxy
  mapping(address => uint256) public countRegisterdNodesPerAccount; // counter for registered nodes per staking account
  mapping(address => address) public nodeToAccount; // map the node address to its staking account
  bool public enabled;

  error GloballyDisabledRegistry();
  error GloballyEnabledRegistry();
  error NodeAlreadyRegisterd();
  error NotEnoughAllowanceToRegisterNode(uint256 maxAllowance, uint256 expectedAllowance);
  error NotEligibleToRegisterNode(uint256 eligibleNodeCount);

  event EnabledNetworkRegistry(bool indexed isEnabled); // Global toggle of the network registry
  event RequirementUpdated(address indexed requirementImplementation); // Emit when the network registry proxy is updated
  event Registered(address indexed account, address indexed nodeAddress); // Emit when a node is included in the registry
  event Deregistered(address indexed account, address indexed nodeAddress); // Emit when a node is removed from the registry
  event RegisteredByManager(address indexed account, address indexed nodeAddress); // Emit when the contract manager includes a node to the registry
  event DeregisteredByManager(address indexed account, address indexed nodeAddress); // Emit when the contract owner removes a node from the registry
  event EligibilityUpdated(address indexed account, bool indexed eligibility); // Emit when the eligibility of an account is updated

  /**
   * @dev Network registry can be globally toggled. If `enabled === true`, only nodes registered
   * in this contract with an eligible account associated can join HOPR network; If `!enabled`,
   * all the nodes can join HOPR network.
   */
  modifier mustBeEnabled() {
    if (!enabled) {
      revert GloballyDisabledRegistry();
    }
    _;
  }

  /**
   * @dev Specify NetworkRegistry logic implementation and set up manager role
   * enable the network registry on deployment.
   * @param _requirementImplementation address of the network registry logic implementation
   * @param _newOwner address of the contract owner
   */
  constructor(
    address _requirementImplementation, 
    address _newOwner
  ) {
    _setupRole(DEFAULT_ADMIN_ROLE, _newOwner);
    _setupRole(MANAGER_ROLE, _newOwner);
    
    requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
    enabled = true;
    emit RequirementUpdated(_requirementImplementation);
    emit EnabledNetworkRegistry(true);
  }

  /**
   * Specify NetworkRegistry logic implementation by a manager
   * @param _requirementImplementation address of the network registry logic implementation
   */
  function updateRequirementImplementation(address _requirementImplementation) external onlyRole(MANAGER_ROLE) {
    requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
    emit RequirementUpdated(_requirementImplementation);
  }

  /**
   * Enable globally the network registry by a manager
   */
  function enableRegistry() external onlyRole(MANAGER_ROLE) {
    if (enabled) {
      revert GloballyEnabledRegistry();
    }
    enabled = true;
    emit EnabledNetworkRegistry(true);
  }

  /**
   * Disable globally the network registry by a manager
   */
  function disableRegistry() external onlyRole(MANAGER_ROLE) mustBeEnabled {
    enabled = false;
    emit EnabledNetworkRegistry(false);
  }

  /**
   * @dev Register new nodes by its staking account
   * @notice Transaction will fail, if a node has already been registered
   * registered nodes are ignored by this function
   * Function can only be called when the registry is enabled.
   * @param nodeAddresses addresses nodes to be registered
   */
  function selfRegister(address[] calldata nodeAddresses) external mustBeEnabled {
    for (uint256 i = 0; i < nodeAddresses.length; i++) {
      address nodeAddress = nodeAddresses[i];
      address stakingAccount = nodeToAccount[nodeAddress];

      if (stakingAccount == address(0)) {
        // if node is not registered, update the counter
        countRegisterdNodesPerAccount[msg.sender]++;
        emit Registered(nodeAddress);
      } else if (stakingAccount != msg.sender) {
        // when the node is not registered under the caller, revert
        revert NodeAlreadyRegisterd();
      }
    }
    // TODO: delete me
    // require(
    //   _checkEligibility(msg.sender, nodeAddresses),
    //   'HoprNetworkRegistry: selfRegister reaches limit, cannot register requested nodes.'
    // );

    // check eligibility of the sender staking account
    (uint256 allowancePerAccount, uint256 eligibleNodeNum) = requirementImplementation.maxAllowedRegistrations(msg.sender, nodeAddresses);
    if (countRegisterdNodesPerAccount[account] > allowancePerAccount) {
      revert NotEnoughAllowanceToRegisterNode({maxAllowance: allowancePerAccount, expectedAllowance: nodeAddresses.length});
    } else if (nodeAddresses.length > eligibleNodeCount) {
      revert NotEligibleToRegisterNode({eligibleNodeCount: eligibleNodeNum});
    } else {
      emit EligibilityUpdated(msg.sender, true);
    }
  }

  /**
   * @dev Allows the staking account to deregister a registered node address
   * Function can only be called when the registry is enabled.
   *
   * @param nodeAddresses addresses nodes to be registered
   */
  function selfDeregister(address[] calldata nodeAddresses) external mustBeEnabled {
    // check eligibility of the sender staking account
    (uint256 allowancePerAccount, uint256 eligibleNodeNum) = requirementImplementation.maxAllowedRegistrations(msg.sender, nodeAddresses);


    // update the counter
    countRegisterdNodesPerAccount[msg.sender] -= hoprPeerIds.length;

    // check sender eligibility
    if (_checkEligibility(msg.sender)) {
      // account becomes eligible
      emit EligibilityUpdated(msg.sender, true);
    } else {
      emit EligibilityUpdated(msg.sender, false);
    }

    for (uint256 i = 0; i < hoprPeerIds.length; i++) {
      string memory hoprPeerId = hoprPeerIds[i];
      require(
        nodePeerIdToAccount[hoprPeerId] == msg.sender,
        'HoprNetworkRegistry: Cannot delete an entry not associated with the caller.'
      );
      delete nodePeerIdToAccount[hoprPeerId];
      emit Deregistered(msg.sender, hoprPeerId);
    }
  }

  /**
   * @dev Owner adds Ethereum addresses and HOPR node ids to the registration.
   * Function can be called at any time.
   * Allows owner to register arbitrary HOPR peer ids even if accounts do not fulfill registration requirements.
   * HOPR node peer id validation should be done off-chain.
   * @notice It allows owner to overwrite exisitng entries.
   * @param accounts Array of Ethereum accounts, e.g. [0xf6A8b267f43998B890857f8d1C9AabC68F8556ee]
   * @param hoprPeerIds Array of hopr nodes id. e.g. [16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1]
   */
  function ownerRegister(address[] calldata accounts, string[] calldata hoprPeerIds) external onlyRole(MANAGER_ROLE) {
    require(hoprPeerIds.length == accounts.length, 'HoprNetworkRegistry: hoprPeerIdes and accounts lengths mismatch');
    for (uint256 i = 0; i < accounts.length; i++) {
      // validate peer the length and prefix of peer Ids. If invalid, skip.
      if (bytes(hoprPeerIds[i]).length == 53 && bytes32(bytes(hoprPeerIds[i])[0:8]) == '16Uiu2HA') {
        string memory hoprPeerId = hoprPeerIds[i];
        address account = accounts[i];
        // link the account with peer id.
        nodePeerIdToAccount[hoprPeerId] = account;
        // update the counter
        countRegisterdNodesPerAccount[account] += 1;
        emit RegisteredByOwner(account, hoprPeerId);
      }
    }
  }

  /**
   * @dev Owner removes previously owner-added Ethereum addresses and HOPR node ids from the registration.
   * Function can be called at any time.
   * @notice Owner can even remove self-declared entries.
   * @param hoprPeerIds Array of hopr nodes id. e.g. [16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1]
   */
  function ownerDeregister(string[] calldata hoprPeerIds) external onlyRole(MANAGER_ROLE) {
    for (uint256 i = 0; i < hoprPeerIds.length; i++) {
      string memory hoprPeerId = hoprPeerIds[i];
      address account = nodePeerIdToAccount[hoprPeerId];
      if (account != address(0)) {
        delete nodePeerIdToAccount[hoprPeerId];
        countRegisterdNodesPerAccount[account] -= 1;
        // Eligibility update should have a logindex strictly smaller
        // than the deregister event to make sure it always gets processed
        // before the deregister event
        emit DeregisteredByOwner(account, hoprPeerId);
      }
    }
  }

  /**
   * @dev Force emit eligibility update by the owner.
   * @notice This does not change the result returned from the proxy, so if `sync` is called on those accounts,
   * it may return a different result.
   * @param accounts Array of Ethereum accounts, e.g. [0xf6A8b267f43998B890857f8d1C9AabC68F8556ee]
   * @param eligibility Array of account eligibility, e.g. [true]
   */
  function ownerForceEligibility(address[] calldata accounts, bool[] calldata eligibility) external onlyRole(MANAGER_ROLE) {
    require(accounts.length == eligibility.length, 'HoprNetworkRegistry: accounts and eligibility lengths mismatch');
    for (uint256 i = 0; i < accounts.length; i++) {
      emit EligibilityUpdated(accounts[i], eligibility[i]);
    }
  }

  /**
   * @dev Owner syncs a list of peer Ids with based on the latest criteria.
   * Function can only be called when the registry is enabled.
   * @notice If a peer id hasn't been registered, its eligibility is not going to be updated
   * @param hoprPeerIds Array of hopr nodes id. e.g. [16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1]
   */
  function sync(string[] calldata hoprPeerIds) external onlyRole(MANAGER_ROLE) mustBeEnabled {
    for (uint256 i = 0; i < hoprPeerIds.length; i++) {
      string memory hoprPeerId = hoprPeerIds[i];
      address account = nodePeerIdToAccount[hoprPeerId];

      if (account == address(0)) {
        // if the account does not have any registered address
        continue;
      }
      if (_checkEligibility(account)) {
        emit EligibilityUpdated(account, true);
      } else {
        // if the account is no longer eligible
        emit EligibilityUpdated(account, false);
      }
    }
  }

  /**
   * @dev Returns if a hopr address is registered and its associated account is eligible or not.
   * @param hoprPeerId hopr node peer id
   */
  function isNodeRegisteredAndEligible(string calldata hoprPeerId) public view returns (bool) {
    // check if peer id is registered
    address account = nodePeerIdToAccount[hoprPeerId];
    if (account == address(0)) {
      // this address has never been registered
      return false;
    }
    return _checkEligibility(account);
  }

  /**
   * @dev Returns if an account address is eligible according to the criteria defined in the proxy implementation
   * It also checks if a node peer id is associated with the account.
   * @param account account address that runs hopr node
   */
  function isAccountRegisteredAndEligible(address account) public view returns (bool) {
    return countRegisterdNodesPerAccount[account] > 0 && _checkEligibility(account);
  }

  /**
   * @dev given the current registry, check if an account has the number of registered nodes within the limit,
   * which is the eligibility of an account.
   * @notice If an account has registerd more peers than it's currently allowed, the account become ineligible
   * @param account address to check its eligibility
   */
  function _checkEligibility(address account) private view returns (bool) {
    uint256 maxAllowedRegistration = requirementImplementation.maxAllowedRegistrations(account);
    if (countRegisterdNodesPerAccount[account] <= maxAllowedRegistration) {
      return true;
    } else {
      return false;
    }
  }
}
