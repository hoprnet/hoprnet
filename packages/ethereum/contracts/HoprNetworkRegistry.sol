// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import '@openzeppelin/contracts/access/Ownable.sol';
import './IHoprNetworkRegistryRequirement.sol';

/**
 * @title HoprNetworkRegistry
 * @dev Smart contract that maintains a list of hopr node addresses that are allowed
 * to enter HOPR network. Each node address is linked with an Ethereum account. Only Ethereum
 * accounts that are eligible according to `IHoprNetworkRegistryRequirement` can register a
 * HOPR node address. If an account wants to change its registerd HOPR node address, it must
 * firstly deregister itself before registering new node.
 *
 * Note that HOPR node address refers to `PeerId.toB58String()`
 *
 * This network registry can be globally enabled/disabled by the owner
 *
 * Implementation of `IHoprNetworkRegistryRequirement` can also be dynamically updated by the
 * owner. Some sample implementations can be found under../proxy/ folder
 *
 * Owner has the power to overwrite the registration
 */
contract HoprNetworkRegistry is Ownable {
  IHoprNetworkRegistryRequirement public requirementImplementation; // Implementation of network registry proxy
  mapping(address => string) public accountToNodeMultiaddr; // mapping the account to the hopr node address in bytes
  mapping(string => address) public nodeMultiaddrToAccount; // mapping the hopr node address in bytes to account
  bool public enabled;

  event EnabledNetworkRegistry(bool indexed isEnabled); // Global toggle of the network registry
  event RequirementUpdated(address indexed requirementImplementation); // Emit when the network registry proxy is updated
  event Registered(address indexed account, string HoprMultiaddr); // Emit when an account register a node address for itself
  event Deregistered(address indexed account); // Emit when an account deregister a node address for itself
  event RegisteredByOwner(address indexed account, string HoprMultiaddr); // Emit when the contract owner register a node address for an account
  event DeregisteredByOwner(address indexed account); // Emit when the contract owner deregister a node address for an account
  event EligibilityUpdated(address indexed account, bool indexed eligibility); // Emit when the eligibility of an account is updated

  /**
   * @dev Network registry can be globally toggled. If `enabled === true`, only nodes registered
   * in this contract with an eligible account associated can join HOPR network; If `!enabled`,
   * all the nodes can join HOPR network regardless the eligibility of the associated account.
   */
  modifier mustBeEnabled() {
    require(enabled, 'HoprNetworkRegistry: Registry is disabled');
    _;
  }

  /**
   * Specify NetworkRegistry logic implementation and transfer the ownership
   * enable the network registry on deployment.
   * @param _requirementImplementation address of the network registry logic implementation
   * @param _newOwner address of the contract owner
   */
  constructor(address _requirementImplementation, address _newOwner) {
    requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
    enabled = true;
    _transferOwnership(_newOwner);
    emit RequirementUpdated(_requirementImplementation);
    emit EnabledNetworkRegistry(true);
  }

  /**
   * Specify NetworkRegistry logic implementation
   * @param _requirementImplementation address of the network registry logic implementation
   */
  function updateRequirementImplementation(address _requirementImplementation) external onlyOwner {
    requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
    emit RequirementUpdated(_requirementImplementation);
  }

  /**
   * Enable globally the network registry by the owner
   */
  function enableRegistry() external onlyOwner {
    require(!enabled, 'HoprNetworkRegistry: Registry is enabled');
    enabled = true;
    emit EnabledNetworkRegistry(true);
  }

  /**
   * Disanable globally the network registry by the owner
   */
  function disableRegistry() external onlyOwner mustBeEnabled {
    enabled = false;
    emit EnabledNetworkRegistry(false);
  }

  /**
   * @dev Checks if the msg.sender fulfills registration requirement at the calling time, if so,
   * register the EOA with HOPR node address. Account can also update its registration status
   * with this function.
   * @notice It allows msg.sender to update registered node address.
   * @param hoprAddress Hopr nodes id in bytes. e.g. 16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1
   * hopr node address should always start with '16Uiu2HA' (0x3136556975324841) and be of length 53
   */
  function selfRegister(string calldata hoprAddress) external mustBeEnabled returns (bool) {
    require(
      bytes(hoprAddress).length == 53 && bytes32(bytes(hoprAddress)[0:8]) == '16Uiu2HA',
      'HoprNetworkRegistry: HOPR node address must be valid'
    );
    // get account associated with the given hopr node address, if any
    address registeredAccount = nodeMultiaddrToAccount[hoprAddress];
    // if the hopr node address was linked to a different account, revert.
    // To change a nodes' linked account, it must be deregistered by the previously linked account
    // first before registering by the new account, to prevent hostile takeover of others' node address
    require(
      registeredAccount == msg.sender || registeredAccount == address(0),
      'HoprNetworkRegistry: Cannot link a registered node to a different account'
    );

    // get multi address associated with the caller, if any
    bytes memory registeredNodeMultiaddrInBytes = bytes(accountToNodeMultiaddr[msg.sender]);
    require(
      registeredNodeMultiaddrInBytes.length == 0 ||
        keccak256(registeredNodeMultiaddrInBytes) == keccak256(bytes(hoprAddress)),
      'HoprNetworkRegistry: Cannot link an account to a different node. Please remove the registered node'
    );

    if (requirementImplementation.isRequirementFulfilled(msg.sender)) {
      // only update the list when no record previously exists
      if (registeredNodeMultiaddrInBytes.length == 0) {
        accountToNodeMultiaddr[msg.sender] = hoprAddress;
        nodeMultiaddrToAccount[hoprAddress] = msg.sender;
        emit Registered(msg.sender, hoprAddress);
      }
      emit EligibilityUpdated(msg.sender, true);
      return true;
    }

    emit EligibilityUpdated(msg.sender, false);
    return false;
  }

  /**
   * @dev Allows when there's already a multi address associated with the caller account, remove the link by deregistering
   */
  function selfDeregister() external mustBeEnabled returns (bool) {
    string memory registeredNodeMultiaddr = accountToNodeMultiaddr[msg.sender];
    require(bytes(registeredNodeMultiaddr).length > 0, 'HoprNetworkRegistry: Cannot delete an empty entry');
    delete accountToNodeMultiaddr[msg.sender];
    delete nodeMultiaddrToAccount[registeredNodeMultiaddr];
    emit Deregistered(msg.sender);
    return true;
  }

  /**
   * @dev Owner adds Ethereum addresses and HOPR node ids to the registration.
   * Allows owner to register arbitrary HOPR Addresses even if accounts do not fulfill registration requirements.
   * HOPR node address validation should be done off-chain.
   * @notice It allows owner to overwrite exisitng entries.
   * @param accounts Array of Ethereum accounts, e.g. [0xf6A8b267f43998B890857f8d1C9AabC68F8556ee]
   * @param hoprAddresses Array of hopr nodes id. e.g. [16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1]
   */
  function ownerRegister(address[] calldata accounts, string[] calldata hoprAddresses)
    external
    onlyOwner
    mustBeEnabled
  {
    require(
      hoprAddresses.length == accounts.length,
      'HoprNetworkRegistry: hoprAddresses and accounts lengths mismatch'
    );
    for (uint256 i = 0; i < accounts.length; i++) {
      string memory hoprAddress = hoprAddresses[i];
      address account = accounts[i];
      accountToNodeMultiaddr[account] = hoprAddress;
      nodeMultiaddrToAccount[hoprAddress] = account;
      emit RegisteredByOwner(account, hoprAddress);
      emit EligibilityUpdated(account, true);
    }
  }

  /**
   * @dev Owner removes previously owner-added Ethereum addresses and HOPR node ids from the registration.
   * @notice Owner can even remove self-declared entries.
   * @param accounts Array of Ethereum accounts, e.g. 0xf6A8b267f43998B890857f8d1C9AabC68F8556ee
   */
  function ownerDeregister(address[] calldata accounts) external onlyOwner mustBeEnabled {
    for (uint256 i = 0; i < accounts.length; i++) {
      address account = accounts[i];
      string memory hoprAddress = accountToNodeMultiaddr[account];
      delete accountToNodeMultiaddr[account];
      delete nodeMultiaddrToAccount[hoprAddress];
      emit DeregisteredByOwner(account);
      emit EligibilityUpdated(account, false);
    }
  }

  /**
   * @dev Owner syncs a list of addresses with based on the latest criteria.
   * @notice If an account hasn't been registered, its eligibility is not going to be updated
   * @param accounts Array of Ethereum accounts, e.g. [0xf6A8b267f43998B890857f8d1C9AabC68F8556ee]
   */
  function sync(address[] calldata accounts) external onlyOwner mustBeEnabled {
    for (uint256 i = 0; i < accounts.length; i++) {
      address account = accounts[i];
      if (bytes(accountToNodeMultiaddr[account]).length == 0) {
        // if the account does not have any registered address
        continue;
      }
      if (!requirementImplementation.isRequirementFulfilled(account)) {
        // if the account is no longer eligible
        emit EligibilityUpdated(account, false);
      } else {
        emit EligibilityUpdated(account, true);
      }
    }
  }

  /**
   * @dev Returns if a hopr address is registered and its associated account is eligible or not.
   * @param hoprAddress hopr node address
   */
  function isNodeRegisteredAndEligible(string calldata hoprAddress) public view returns (bool) {
    address account = nodeMultiaddrToAccount[hoprAddress];
    if (account == address(0)) {
      // this address has never been registered
      return false;
    }
    return requirementImplementation.isRequirementFulfilled(account);
  }

  /**
   * @dev Returns if an account address is eligible according to the criteria defined in the implementation
   * It also checks if a node address is associated with the account.
   * @param account account address that runs hopr node
   */
  function isAccountRegisteredAndEligible(address account) public view returns (bool) {
    return
      bytes(accountToNodeMultiaddr[account]).length != 0 && requirementImplementation.isRequirementFulfilled(account);
  }
}
