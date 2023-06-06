// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.7.0 <0.9.0;

/**
 * This contract follows the principle of `zodiac/core/Module.sol`
 * but implement differently in order to overwrite functionalities
 */
import "./SimplifiedModule.sol";
import "./CapabilityPermissions.sol";
import "../../interfaces/IHoprChannels.sol";
import "../../interfaces/INodeManagementModule.sol";

// when the contract has already been initialized
error AlreadyInitialized();
// when a node is a member of the role
error WithMembership();
// Once module gets created, the ownership cannot be transferred
error CannotChangeOwner();

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
contract HoprNodeManagementModule is SimplifiedModule, IHoprNodeManagementModule {
  bool public constant isHoprNodeManagementModule = true;
  // address to send delegated multisend calls to 
  address public multisend;
  // from HoprCapabilityPermissions. This module is a Role where members are NODE_CHAIN_KEYs
  Role internal role;

  event SetMultisendAddress(address multisendAddress);
  event NodeAdded(address indexed node);
  event NodeRemoved(address indexed node);

  modifier nodeOnly() {
    if (!role.members[_msgSender()]) {
      revert HoprCapabilityPermissions.NoMembership();
    }
    _;
  }

  /// @custom:oz-upgrades-unsafe-allow constructor
  constructor() {
    _disableInitializers();
  }

  function initialize(bytes memory initParams) public initializer {
    (address _safe, address _multisend) = abi.decode(
        initParams,
        (address, address)
    );

    // cannot accept a zero address as Safe or multisend contract
    if (_safe == address(0) || _multisend == address(0)) {
      revert HoprCapabilityPermissions.AddressIsZero();
    }

    // cannot setup again if it's been set up
    if (address(avatar) != address(0) || multisend != address(0)) {
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
   * @dev check if an address has been included as a member (NODE_CHAIN_KEY)
   * @param nodeAddress address to be checked
   */
  function isNode(address nodeAddress) external view returns (bool) {
    return role.members[nodeAddress];
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

  /// @dev Set the address of the expected multisend library
  /// @notice Only callable by owner.
  /// @param _multisend address of the multisend library contract
  function setMultisend(address _multisend) external onlyOwner {
      multisend = _multisend;
      emit SetMultisendAddress(multisend);
  }
  
  /**
   * @dev Allows the target address to be scoped as a HoprChannels target
   * and its token as a HoprToken target. HoprToken address is obtained from 
   * HoprChannels contract
   * @param hoprChannelsAddress address of HoprChannels contract to be added to scope
   */
  function addChannelsAndTokenTarget(address hoprChannelsAddress) external onlyOwner {
    // get tokens contract
    address hoprTokenAddress = IHoprChannels(hoprChannelsAddress).token();

    // add default scope for Channels TargetType
    HoprCapabilityPermissions.scopeTargetChannels(role, hoprChannelsAddress);
    // add default scope for Token TargetType
    HoprCapabilityPermissions.scopeTargetToken(role, hoprTokenAddress);
  }

  /**
   * @dev Scopes the target address as a HoprChannels target
   * @param hoprChannelsAddress address of HoprChannels contract to be added to scope
   */
  function scopeTargetChannels(address hoprChannelsAddress) external onlyOwner {
    HoprCapabilityPermissions.scopeTargetChannels(role, hoprChannelsAddress);
  }

  /**
   * @dev Scopes the target address as a HoprToken target
   * @param hoprTokenAddress address of HoprToken contract to be added to scope
   */
  function scopeTargetToken(address hoprTokenAddress) external onlyOwner {
    HoprCapabilityPermissions.scopeTargetToken(role, hoprTokenAddress);
  }

  /**
   * @dev Scopes the target address as a Send target, so native tokens can be 
   * transferred from the avatar to the target.
   * @notice Only member is allowed to be a beneficiary
   * @param beneficiaryAddress address that can receive native tokens
   */
  function scopeTargetSend(address beneficiaryAddress) external onlyOwner {
    if (!role.members[beneficiaryAddress]) {
      revert HoprCapabilityPermissions.NoMembership();
    }
    HoprCapabilityPermissions.scopeTargetSend(role, beneficiaryAddress);
  }

  /**
   * @dev Revokes the target address from the scope
   * @param targetAddress The address of the target to be revoked.
   */
  function revokeTarget(address targetAddress) external onlyOwner {
    HoprCapabilityPermissions.revokeTarget(role, targetAddress);
  }

  /**
   * @dev Sets the permission for a specific function on a scoped HoprChannels target.
   * @param targetAddress The address of the scoped HoprChannels target.
   * @param functionSig The function signature of the specific function.
   * @param channelId The channelId of the scoped HoprChannels target.
   * @param permission The permission to be set for the specific function.
   */
  function scopeChannelsCapability(
    address targetAddress,
    bytes4 functionSig,
    bytes32 channelId,
    HoprChannelsPermission permission
  ) external onlyOwner {
    HoprCapabilityPermissions.scopeChannelsCapability(role, targetAddress, functionSig, channelId, permission);
  }

  /**
   * @dev Sets the permissions for functions on a scoped HoprChannels target for a given channel
   * @notice it can batch maxinum 7 capabilities. 
   * Encoding of function signatures is right-padded in Big-Eidian format
   * Encoding of permissions is left-padded in Little-Eidian format
   * @param targetAddress The address of the scoped HoprChannels target.
   * @param encodedSigsPermissions The encoded function signatures and permissions
   * @param channelId The channelId of the scoped HoprChannels target.
   */
  function batch7ScopeChannelsCapability(
    address targetAddress,
    bytes32 encodedSigsPermissions,
    bytes32 channelId
  ) external onlyOwner {
    (bytes4[] memory functionSigs, uint256[] memory permissions) = HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedSigsPermissions, 7);

    for (uint256 i = 0; i < 7; i++) {
      if (functionSigs[i] != bytes4(0)) {
        HoprCapabilityPermissions.scopeChannelsCapability(role, targetAddress, functionSigs[i], channelId, HoprChannelsPermission(permissions[i]));
      }
    }
  }

  /**
   * @dev Sets the permission for a specific function on a scoped HoprToken target.
   * @param targetAddress The address of the scoped HoprToken target.
   * @param functionSig The function signature of the specific function.
   * @param beneficiary The beneficiary address for the scoped HoprToken target.
   * @param permission The permission to be set for the specific function.
   */
  function scopeTokenCapability(
    address targetAddress,
    bytes4 functionSig,
    address beneficiary,
    HoprTokenPermission permission
  ) external onlyOwner {
    HoprCapabilityPermissions.scopeTokenCapability(role, targetAddress, functionSig, beneficiary, permission);
  }

  /**
   * @dev Sets the permissions for functions on a scoped HoprToken target for different beneficiaries
   * @notice it can batch maxinum 7 capabilities. 
   * Encoding of function signatures is right-padded in Big-Eidian format
   * Encoding of permissions is left-padded in Little-Eidian format
   * @param targetAddress The address of the scoped HoprToken target.
   * @param encodedSigsPermissions The encoded function signatures and permissions
   * @param beneficiaries Array of beneficiary addresses for the scoped HoprToken target.
   */
  function batch7ScopeTokenCapability(
    address targetAddress,
    bytes32 encodedSigsPermissions,
    address[] memory beneficiaries
  ) external onlyOwner {
    uint256 len = beneficiaries.length;
    if (len > 7) {
      revert HoprCapabilityPermissions.ArrayTooLong();
    }
    (bytes4[] memory functionSigs, uint256[] memory permissions) = HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedSigsPermissions, len);

    for (uint256 i = 0; i < len; i++) {
      if (functionSigs[i] != bytes4(0)) {
        HoprCapabilityPermissions.scopeTokenCapability(role, targetAddress, functionSigs[i], beneficiaries[i], HoprTokenPermission(permissions[i]));
      }
    }
  }

  /**
   * @dev Sets the permission for sending native tokens to a specific beneficiary
   * @param beneficiary The beneficiary address for the scoped Send target.
   * @param permission The permission to be set for the specific function.
   */
  function scopeSendCapability(
    address beneficiary,
    SendPermission permission
  ) external onlyOwner {
    HoprCapabilityPermissions.scopeSendCapability(role, beneficiary, permission);
  }

  /**
   * @dev Sets the permission for sending native tokens to a specific for multple beneficiaries
   * @notice it can batch maxinum 7 capabilities. 
   * Encoding of permissions is left-padded in Little-Eidian format
   * @param encodedSigsPermissions The encoded function signatures and permissions
   * @param beneficiaries Array of beneficiary addresses for the scoped HoprToken target.
   */
  function batch7ScopeSendCapability(
    uint256 encodedSigsPermissions,
    address[] memory beneficiaries
  ) external onlyOwner {
    uint256 len = beneficiaries.length;
    if (len > 7) {
      revert HoprCapabilityPermissions.ArrayTooLong();
    }
    uint256[] memory permissions = HoprCapabilityPermissions.decodePermissionEnums(encodedSigsPermissions, len);

    for (uint256 i = 0; i < len; i++) {
      HoprCapabilityPermissions.scopeSendCapability(role, beneficiaries[i], SendPermission(permissions[i]));
    }
  }

  // ===========================================================
  // ----------------------- INHERITANCE -----------------------
  // ===========================================================

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
   * @dev Override {transferOwnership} so the owner cannot be changed once created
   */
  function transferOwnership(address /*newOwner*/) public view override(OwnableUpgradeable) onlyOwner {
    revert CannotChangeOwner();
  }
}