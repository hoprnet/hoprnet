// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8;
pragma abicoder v2;

/**
 * This contract follows the principle of `zodiac/core/Module.sol`
 * but implement differently in order to overwrite functionalities
 */
import "./PermissionedModule/SimplifiedModule.sol";
import "./PermissionedModule/SimplifiedPermissions.sol";
import "../IHoprChannels.sol";

error AddressIsZero();
error NodeIsAdded();
error NodeIsNotAdded();
error InitializationErrorSafeSet();
error CannotChangeOwner();
error NotAllowedNode();
error NotHoprChannelsContract();

/**
 * @title Module to enable HOPR nodes to interact with HOPR Channels contract
 * on behalf of the Safe
 * Only those addresses that are added by the Safe can call execTransactionFromModule
 * A deployed Multisend contract address is included in the contract
 * Module can only execute DELEGATECALL to the Multisend contract 
 * Module can execute CALLs to HoprChannels contracts
 * Module can execute CALLs to HoprToken contracts
 */
contract HoprCapabilityBasedNodeManagementModule is SimplifiedModule {
  // Node addresses that can execute transaction for target
  mapping(address=>bool) public nodes;
  // HoprChannel addresses on which 
  mapping(address=>bool) public channels;
    
  // from SimplifiedPermissions. This module is a role (NODE_CHAIN_KEYS)
  Role internal role;

  modifier nodeOnly() {
    if(!nodes[_msgSender()]) {
      revert NotAllowedNode();
    }
    _;
  }

  event NodeAdded(address indexed avatar, address indexed node);
  event NodeRemoved(address indexed avatar, address indexed node);

  // set the avatar (safe) address to be zero for the singleton
  constructor() {
    avatar = address(0);
  }

  function setUp(bytes memory initParams) public override {
    (address _safe) = abi.decode(
        initParams,
        (address)
    );
    __Ownable_init();

    // cannot accept a zero address as Safe contract
    if (_safe == address(0)) {
      revert AddressIsZero();
    }

    // cannot setup again if it's been set up
    if (address(avatar) != address(0)) {
      revert InitializationErrorSafeSet();
    }

    // internally setAvatar and setTarget
    setAvatar(_safe);
    // transfer ownership
    _transferOwnership(_safe);
  }

  /**
   * @dev Add a node to be able to execute this module, to the target
   */
  function addNode(address nodeAddress) external onlyOwner {
    // cannot add a node that's added
    if (nodes[nodeAddress]) {
      revert NodeIsAdded();
    }
    nodes[nodeAddress] = true;
    emit NodeAdded(avatar, nodeAddress);
  }

  /**
   * @dev Remove a node from being able to execute this module, to the target
   */
  function removeNode(address nodeAddress) external onlyOwner {
    // cannot move a node that's not added
    if (!nodes[nodeAddress]) {
      revert NodeIsNotAdded();
    }
    nodes[nodeAddress] = false;
    emit NodeRemoved(avatar, nodeAddress);
  }

  /**
   * @dev Override {transferOwnership} so the owner cannot be changed once created
   */
  function transferOwnership(address newOwner) public override(OwnableUpgradeable) onlyOwner {
    revert CannotChangeOwner();
  }

  /**
   * @dev override {exec} function to include extra check on the permission
   * caller must be an added node
   */
  function exec(
    address to,
    uint256 value,
    bytes memory data,
    Enum.Operation operation
  ) internal override(SimplifiedModule) nodeOnly returns (bool success) {
    // perform additional check before proceed
    _permissionGuard(to, value, data, operation);
    super.exec(to, value, data, operation);
  }

  /**
   * @dev override {execAndReturnData} function to include extra check on the permission
   * caller must be an added node
   */
  function execAndReturnData(
    address to,
    uint256 value,
    bytes memory data,
    Enum.Operation operation
  ) internal override(SimplifiedModule) nodeOnly returns (bool success, bytes memory returnData) {
    // perform additional check before proceed
    _permissionGuard(to, value, data, operation);
    super.execAndReturnData(to, value, data, operation);
  }

  /**
   * @dev addtional check on the permission
   * This check is guard-like, but with an internal function and it's mandatory.
   * Implementation of this contract is heavily influenced by `gnosis/zodiac-modifier-roles-v1`
   *
   * Transactions that can be executed by this module is highly limited.
   * call: `to` must be HoprChannels contracts or HoprToken contract. 
   * For HoprChannels contract, allowed function selectors are limited to what's defined in `IHoprChannels`
   * For HoprToken contract, allowed function selectors are limited to `approve`, `send`
   * delegatecall: when callling HoprChannels contracts, similar checks are performed
   */
  function _permissionGuard(
    address to,
    uint256 value,
    bytes memory data,
    Enum.Operation operation
  ) internal {
    if (operation == Enum.Operation.Call) {
      // FIXME: update after the library is updated
      // _checkTransaction(to, value, data);
    } else {
      // delegate call
      // FIXME: update after the library is updated
      // _checkMultisendTransaction(data);
    }
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
    SimplifiedPermissions.scopeTarget(role, hoprChannelsAddress);
    // FIXME:
    // SimplifiedPermissions.scopeTarget(role, hoprTokenAddress);
    // add default scopr for hoprTokenAddress
  }
}