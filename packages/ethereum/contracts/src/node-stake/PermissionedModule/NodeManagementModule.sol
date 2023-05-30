// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

/**
 * This contract follows the principle of `zodiac/core/Module.sol`
 * but implement differently in order to overwrite functionalities
 */
import "./SimplifiedModule.sol";
import "./CapabilityPermissions.sol";
import "../../IHoprChannels.sol";

// address is 0x00
error AddressIsZero();
// when the contract has already been initialized
error AlreadyInitialized();
// when a node is a member of the role
error WithMembership();

error CannotChangeOwner();
error NotAllowedNode();
error NotHoprChannelsContract();

/**
 * @title Permissioned capability-based module for HOPR nodes operations
 *
 * @dev Drawing inspiration from the `zodiac-modifier-roles-v1` `Roles.sol` contract, 
 * this module removes target attribute and is dedicated for managing HOPR nodes
 * with capability based permissions.
 * Only those addresses that are added by the Safe can call execTransactionFromModule
 * A deployed Multisend contract address is included in the contract
 * Module can only execute DELEGATECALL to the Multisend contract 
 * Module can execute CALLs to HoprChannels contracts
 * Module can execute CALLs to HoprToken contracts
 */
contract HoprNodeManagementModule is SimplifiedModule {
  // // Node addresses that can execute transaction for target
  // mapping(address=>bool) public nodes;
  // // HoprChannel addresses on which 
  // mapping(address=>bool) public channels;

  // address to send delegated multisend calls to 
  address public multisend;
  // from HoprCapabilityPermissions. This module is a Role where members are NODE_CHAIN_KEYs
  Role internal role;

  modifier nodeOnly() {
    if (!role.members[_msgSender()]) {
      revert HoprCapabilityPermissions.NoMembership();
    }
    _;
  }

  event SetMultisendAddress(address multisendAddress);
  event NodeAdded(address indexed node);
  event NodeRemoved(address indexed node);

  // set values to be zero for the singleton
  constructor() {
    avatar = address(0);
    multisend = address(0);
  }

  function setUp(bytes memory initParams) public override {
    (address _safe, address _multisend) = abi.decode(
        initParams,
        (address, address)
    );
    __Ownable_init();

    // cannot accept a zero address as Safe or multisend contract
    if (_safe == address(0) || _multisend == address(0)) {
      revert AddressIsZero();
    }

    // cannot setup again if it's been set up
    if (address(avatar) != address(0) || _multisend != address(0)) {
      revert AlreadyInitialized();
    }

    // internally setAvatar and setTarget
    avatar = _safe;
    multisend = _multisend;
    // transfer ownership
    _transferOwnership(_safe);
    emit AvatarSet(address(0), avatar);
    emit SetMultisendAddress(_multisend);
  }

  /**
   * @dev Add a node to be able to execute this module, to the target
   */
  function addNode(address nodeAddress) external onlyOwner {
    // cannot add a node that's added
    if (role.members[nodeAddress]) {
      revert WithMembership();
    }
    role.members[nodeAddress] = true;
    emit NodeAdded(nodeAddress);
  }

  /**
   * @dev Remove a node from being able to execute this module, to the target
   */
  function removeNode(address nodeAddress) external onlyOwner {
    // cannot move a node that's not added
    if (!role.members[nodeAddress]) {
      revert HoprCapabilityPermissions.NoMembership();
    }
    role.members[nodeAddress] = false;
    emit NodeRemoved(nodeAddress);
  }

  /**
   * @dev Override {transferOwnership} so the owner cannot be changed once created
   */
  function transferOwnership(address /*newOwner*/) public override(OwnableUpgradeable) onlyOwner {
    revert CannotChangeOwner();
  }

  /// @dev Passes a transaction to the modifier.
  /// @param to Destination address of module transaction
  /// @param value Ether value of module transaction
  /// @param data Data payload of module transaction
  /// @param operation Operation type of module transaction
  /// @notice Can only be called by enabled modules
  function execTransactionFromModule(
      address to,
      uint256 value,
      bytes calldata data,
      Enum.Operation operation
  ) public nodeOnly returns (bool success) {
      HoprCapabilityPermissions.check(
          role,
          multisend,
          to,
          value,
          data,
          operation
      );
      return exec(to, value, data, operation);
  }

  /// @dev Passes a transaction to the modifier, expects return data.
  /// @param to Destination address of module transaction
  /// @param value Ether value of module transaction
  /// @param data Data payload of module transaction
  /// @param operation Operation type of module transaction
  /// @notice Can only be called by enabled modules
  function execTransactionFromModuleReturnData(
      address to,
      uint256 value,
      bytes calldata data,
      Enum.Operation operation
  ) public nodeOnly returns (bool, bytes memory) {
      HoprCapabilityPermissions.check(
          role,
          multisend,
          to,
          value,
          data,
          operation
      );
      return execAndReturnData(to, value, data, operation);
  }

  /**
   * @dev FIXME: add guard of who can call this function
   */
  function _addHoprChannelsAsTarget(address hoprChannelsAddress) internal {
    if (!IHoprChannels(hoprChannelsAddress).IS_HOPR_CHANNELS()) {
      // not a channel contract
      revert NotHoprChannelsContract();
    }
    // get tokens contract FIXME:
    // address hoprTokenAddress = IHoprChannels(hoprChannelsAddress).token();

    // add default scope for hoprChannelsAddress
    // SimplifiedPermissions.scopeTarget(role, hoprChannelsAddress);
    // FIXME:
    // SimplifiedPermissions.scopeTarget(role, hoprTokenAddress);
    // add default scopr for hoprTokenAddress
  }
}