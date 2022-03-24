// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

import '@openzeppelin/contracts/access/Ownable.sol';
import './IHoprNetworkRegistryRequirement.sol';

/**
 * @dev Allowlist addresses and hopr nodes
 */
contract HoprNetworkRegistry is Ownable {
  IHoprNetworkRegistryRequirement public requirementImplementation;
  mapping(address => string) public accountToNodeMultiaddr;
  mapping(string => address) public nodeMultiaddrToAccount;

  event RequirementUpdated(address indexed requirementImplementation);
  event Registered(address indexed account, string HoprMultiaddr);
  event RegisteredByOwner(address indexed account, string HoprMultiaddr);
  event DeregisteredByOwner(address indexed account);
  event EligibilityUpdated(address indexed account, bool indexed eligibility);

  /**
   * Specify NetworkRegistry logic implemntation and transfer the ownership
   * _requirementImplementation address of the network registry logic implementation
   * _newOwner address of the contract owner
   */
  constructor(address _requirementImplementation, address _newOwner) {
    requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
    _transferOwnership(_newOwner);
    emit RequirementUpdated(_requirementImplementation);
  }

  /**
   * Specify NetworkRegistry logic implemntation
   * _requirementImplementation address of the network registry logic implementation
   */
  function updateRequirementImplementation(address _requirementImplementation) external onlyOwner {
    requirementImplementation = IHoprNetworkRegistryRequirement(_requirementImplementation);
    emit RequirementUpdated(_requirementImplementation);
  }

  /**
   * @dev Checks if the msg.sender fulfills registration requirement at the calling time, if so,
   * register the EOA with HOPR node address. Account can also update its registration status
   * with this function.
   * @notice It allows msg.sender to update registered node address.
   * @param hoprAddress Hopr nodes id. e.g. 16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1
   */
  function selfRegister(string calldata hoprAddress) external returns (bool) {
    if (requirementImplementation.isRequirementFulfilled(msg.sender)) {
      accountToNodeMultiaddr[msg.sender] = hoprAddress;
      nodeMultiaddrToAccount[hoprAddress] = msg.sender;
      emit Registered(msg.sender, hoprAddress);
      emit EligibilityUpdated(msg.sender, true);
      return true;
    }

    if (bytes(accountToNodeMultiaddr[msg.sender]).length != 0) {
      // if the account has a registration entry, keep the entry but update the eligibility
      emit EligibilityUpdated(msg.sender, false);
    }
    return false;
  }

  /**
   * @dev Owner adds Ethereum addresses and HOPR node ids to the registration.
   * Allows owner to register arbitrary HOPR Addresses even if accounts do not fulfill registration requirements.
   * HOPR node address validation should be done off-chain.
   * @notice It allows owner to overwrite exisitng entries.
   * @param accounts Array of Ethereum accounts, e.g. [0xf6A8b267f43998B890857f8d1C9AabC68F8556ee]
   * @param hoprAddresses Array of hopr nodes id. e.g. [16Uiu2HAmHsB2c2puugVuuErRzLm9NZfceainZpkxqJMR6qGsf1x1]
   */
  function ownerRegister(address[] calldata accounts, string[] calldata hoprAddresses) external onlyOwner {
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
  function ownerDeregister(address[] calldata accounts) external onlyOwner {
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
  function sync(address[] calldata accounts) external onlyOwner {
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
