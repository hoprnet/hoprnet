// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

/**
 * This contract follows the principle of `zodiac/core/Module.sol`
 * but implement differently in order to overwrite functionalities
 */
import { Enum } from "safe-contracts/common/Enum.sol";
import { SimplifiedModule } from "./SimplifiedModule.sol";
import { HoprCapabilityPermissions, Role, GranularPermission } from "./CapabilityPermissions.sol";
import { HoprChannels } from "../../Channels.sol";
import { IHoprNodeManagementModule } from "../../interfaces/INodeManagementModule.sol";
import { EnumerableTargetSet, TargetSet, TargetUtils, Target } from "../../utils/EnumerableTargetSet.sol";

/**
 *    &&&&
 *    &&&&
 *    &&&&
 *    &&&&  &&&&&&&&&       &&&&&&&&&&&&          &&&&&&&&&&/   &&&&.&&&&&&&&&
 *    &&&&&&&&&   &&&&&   &&&&&&     &&&&&,     &&&&&    &&&&&  &&&&&&&&   &&&&
 *     &&&&&&      &&&&  &&&&#         &&&&   &&&&&       &&&&& &&&&&&     &&&&&
 *     &&&&&       &&&&/ &&&&           &&&& #&&&&        &&&&  &&&&&
 *     &&&&         &&&& &&&&&         &&&&  &&&&        &&&&&  &&&&&
 *     %%%%        /%%%%   %%%%%%   %%%%%%   %%%%  %%%%%%%%%    %%%%%
 *    %%%%%        %%%%      %%%%%%%%%%%    %%%%   %%%%%%       %%%%
 *                                          %%%%
 *                                          %%%%
 *                                          %%%%
 *
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
    using TargetUtils for Target;
    using EnumerableTargetSet for TargetSet;

    bool public constant isHoprNodeManagementModule = true;
    // address to send delegated multisend calls to
    address public multisend;
    // from HoprCapabilityPermissions. This module is a Role where members are NODE_CHAIN_KEYs
    Role internal role;

    event SetMultisendAddress(address indexed multisendAddress);
    event NodeAdded(address indexed node);
    event NodeRemoved(address indexed node);

    // when the contract has already been initialized
    error AlreadyInitialized();
    // when a node is a member of the role
    error WithMembership();
    // Once module gets created, the ownership cannot be transferred
    error CannotChangeOwner();
    // when safe and multisend address are the same
    error SafeMultisendSameAddress();

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
        (address _safe, address _multisend, bytes32 _defaultTokenChannelsTarget) =
            abi.decode(initParams, (address, address, bytes32));

        // cannot accept a zero address as Safe or multisend contract
        if (_safe == address(0) || _multisend == address(0)) {
            revert HoprCapabilityPermissions.AddressIsZero();
        }

        // cannot use same address for safe and multisend
        if (_safe == _multisend) {
            revert SafeMultisendSameAddress();
        }

        // cannot setup again if it's been set up
        if (owner() != address(0) || multisend != address(0)) {
            revert AlreadyInitialized();
        }

        // internally setTarget
        multisend = _multisend;
        _addChannelsAndTokenTarget(Target.wrap(uint256(_defaultTokenChannelsTarget)));
        // transfer ownership
        _transferOwnership(_safe);
        emit SetMultisendAddress(_multisend);
    }

    /**
     * @dev try to get target given a target address. It does not revert if `targetAddress` is not scoped
     * @param targetAddress Address of target
     */
    function tryGetTarget(address targetAddress) external view returns (bool, Target) {
        return role.targets.tryGet(targetAddress);
    }

    /**
     * @dev get all the scoped targets
     */
    function getTargets() external view returns (Target[] memory) {
        return role.targets.values();
    }

    /**
     * @dev get the granular permission
     * @param capabilityKey Key to the capability.
     * @param pairId hashed value of the pair of concern, e.g. for channel `keccak256(src,dst)`, for token and send
     * `keccak256(owner,spender)`
     */
    function getGranularPermissions(bytes32 capabilityKey, bytes32 pairId) external view returns (GranularPermission) {
        return role.capabilities[capabilityKey][pairId];
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
     * @param nodeAddress address of node
     */
    function addNode(address nodeAddress) external onlyOwner {
        _addNode(nodeAddress);
    }

    /**
     * @dev Remove a node from being able to execute this module, to the target
     * @param nodeAddress address of node
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
     * @param defaultTarget The default target with default permissions for CHANNELS and TOKEN target
     */
    function addChannelsAndTokenTarget(Target defaultTarget) external onlyOwner {
        _addChannelsAndTokenTarget(defaultTarget);
    }

    /**
     * @dev Include a node as a member, set its default SEND permissions
     */
    function includeNode(Target nodeDefaultTarget) external onlyOwner {
        address nodeAddress = nodeDefaultTarget.getTargetAddress();
        // add a node as a member
        _addNode(nodeAddress);
        // scope default capabilities
        HoprCapabilityPermissions.scopeTargetSend(role, nodeDefaultTarget);
        // scope granular capabilities to send native tokens to itself
        HoprCapabilityPermissions.scopeSendCapability(role, nodeAddress, nodeAddress, GranularPermission.ALLOW);
    }

    /**
     * @dev Scopes the target address as a HoprChannels target
     * @param defaultTarget The default target with default permissions for CHANNELS target
     */
    function scopeTargetChannels(Target defaultTarget) external onlyOwner {
        HoprCapabilityPermissions.scopeTargetChannels(role, defaultTarget);
    }

    /**
     * @dev Scopes the target address as a HoprToken target
     * @param defaultTarget The default target with default permissions for TOKEN target
     */
    function scopeTargetToken(Target defaultTarget) external onlyOwner {
        HoprCapabilityPermissions.scopeTargetToken(role, defaultTarget);
    }

    /**
     * @dev Scopes the target address as a Send target, so native tokens can be
     * transferred from the avatar to the target.
     * @notice Only member is allowed to be a beneficiary
     * @param defaultTarget The default target with default permissions for SEND target
     */
    function scopeTargetSend(Target defaultTarget) external onlyOwner {
        address beneficiaryAddress = defaultTarget.getTargetAddress();
        if (!role.members[beneficiaryAddress]) {
            revert HoprCapabilityPermissions.NoMembership();
        }
        HoprCapabilityPermissions.scopeTargetSend(role, defaultTarget);
    }

    /**
     * @dev Revokes the target address from the scope
     * @param targetAddress The address of the target to be revoked.
     */
    function revokeTarget(address targetAddress) external onlyOwner {
        HoprCapabilityPermissions.revokeTarget(role, targetAddress);
    }

    /**
     * @dev Sets the permission for a set of functions on a scoped CHANNELS target for a given channel
     * @notice it can batch maxinum 7 capabilities.
     * Encoding of function signatures is right-padded, where indexes grow from left to right
     * Encoding of permissions is left-padded, where indexes grow from left to right
     * @param targetAddress The address of the scoped HoprChannels target.
     * @param channelId The channelId of the scoped HoprChannels target.
     * @param encodedSigsPermissions The encoded function signatures and permissions
     */
    function scopeChannelsCapabilities(
        address targetAddress,
        bytes32 channelId,
        bytes32 encodedSigsPermissions
    )
        external
        onlyOwner
    {
        HoprCapabilityPermissions.scopeChannelsCapabilities(role, targetAddress, channelId, encodedSigsPermissions);
    }

    /**
     * @dev Sets the permissions for functions on a scoped HoprToken target for different beneficiaries
     * @notice it can batch maxinum 7 capabilities.
     * Encoding of function signatures is right-padded, where indexes grow from left to right
     * Encoding of permissions is left-padded, where indexes grow from left to right
     * @param nodeAddress The address of the caller node
     * @param targetAddress The address of the scoped HoprToken target.
     * @param beneficiary The beneficiary address for the scoped HoprToken target.
     * @param encodedSigsPermissions The encoded function signatures and permissions
     */
    function scopeTokenCapabilities(
        address nodeAddress,
        address targetAddress,
        address beneficiary,
        bytes32 encodedSigsPermissions
    )
        external
        onlyOwner
    {
        HoprCapabilityPermissions.scopeTokenCapabilities(
            role, nodeAddress, targetAddress, beneficiary, encodedSigsPermissions
        );
    }

    /**
     * @dev Sets the permission for sending native tokens to a specific beneficiary
     * @param nodeAddress The address of the caller node
     * @param beneficiary The beneficiary address for the scoped Send target.
     * @param permission The permission to be set for the specific function.
     */
    function scopeSendCapability(
        address nodeAddress,
        address beneficiary,
        GranularPermission permission
    )
        external
        onlyOwner
    {
        HoprCapabilityPermissions.scopeSendCapability(role, nodeAddress, beneficiary, permission);
    }

    // ===========================================================
    // ------------------------ UTILITIES ------------------------
    // ===========================================================
    /**
     * @dev help encode function permissions into a bytes32
     * @param functionSigs array of function signatures on target
     * @param permissions array of granular permissions on target
     */
    function encodeFunctionSigsAndPermissions(
        bytes4[] memory functionSigs,
        GranularPermission[] memory permissions
    )
        external
        pure
        returns (bytes32 encoded, uint256 length)
    {
        return HoprCapabilityPermissions.encodeFunctionSigsAndPermissions(functionSigs, permissions);
    }

    /**
     * @dev help encode function permissions into a bytes32
     * @param encoded encode permissions in bytes32
     * @param length length of permissions
     */
    function decodeFunctionSigsAndPermissions(
        bytes32 encoded,
        uint256 length
    )
        external
        pure
        returns (bytes4[] memory functionSigs, GranularPermission[] memory permissions)
    {
        return HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encoded, length);
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
    )
        public
        nodeOnly
        returns (bool success)
    {
        HoprCapabilityPermissions.check(role, multisend, to, value, data, operation);
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
    )
        public
        nodeOnly
        returns (bool, bytes memory)
    {
        HoprCapabilityPermissions.check(role, multisend, to, value, data, operation);
        return execAndReturnData(to, value, data, operation);
    }

    /**
     * @dev Private function to scope a channel and token target.
     * @param defaultTarget The default target with default permissions for CHANNELS and TOKEN target
     */
    function _addChannelsAndTokenTarget(Target defaultTarget) private {
        // get channels andtokens contract
        address hoprChannelsAddress = defaultTarget.getTargetAddress();
        address hoprTokenAddress = address(HoprChannels(hoprChannelsAddress).token());

        // add default scope for Channels TargetType, with the build target for hoprChannels address
        HoprCapabilityPermissions.scopeTargetChannels(role, defaultTarget.forceWriteTargetAddress(hoprChannelsAddress));
        // add default scope for Token TargetType
        HoprCapabilityPermissions.scopeTargetToken(role, defaultTarget.forceWriteTargetAddress(hoprTokenAddress));
    }

    /**
     * @dev private function to add a node as a member of the module
     */
    function _addNode(address nodeAddress) private {
        // cannot add a node that's added
        if (role.members[nodeAddress]) {
            revert WithMembership();
        }
        role.members[nodeAddress] = true;
        emit NodeAdded(nodeAddress);
    }
}
