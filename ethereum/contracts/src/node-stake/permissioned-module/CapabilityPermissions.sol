// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity >=0.8.0 <0.9.0;

import { Enum } from "safe-contracts/common/Enum.sol";
import { HoprChannels } from "../../Channels.sol";
import { EnumerableTargetSet, TargetSet } from "../../utils/EnumerableTargetSet.sol";
import {
    TargetUtils,
    Target,
    TargetPermission,
    TargetType,
    Clearance,
    CapabilityPermission
} from "../../utils/TargetUtils.sol";
import { IERC20, IERC777 } from "../../static/openzeppelin-contracts/ERC777.sol";

enum GranularPermission {
    NONE,
    ALLOW,
    BLOCK
}

struct Role {
    TargetSet targets; // target addresses that can be called
    mapping(address => bool) members; // eligible caller. May be able to receive native tokens (e.g. xDAI), if set to
        // allowed
    // For CHANNELS target: capabilityKey (bytes32) => channel Id (keccak256(src, dest)) => GranularPermission
    // For TOKEN target: capabilityKey (bytes32) => pair Id (keccak256(node address, spender address)) =>
    // GranularPermission
    // For SEND target:  bytes32(0x00) => pair Id (keccak256(node address, spender address)) => GranularPermission
    mapping(bytes32 => mapping(bytes32 => GranularPermission)) capabilities;
}

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
 * @dev Drawing inspiration from the `zodiac-modifier-roles-v1` `Permissions.sol` contract,
 * this library is designed to support a single role and offers a set of specific functions
 * for interacting with HoprChannels and HoprToken contracts
 *
 * Adapted from `Permissions.sol` at commit 454be9d3c26f90221ca717518df002d1eca1845f, which
 * was audited https://github.com/gnosis/zodiac-modifier-roles-v1/tree/main/packages/evm/docs
 *
 * It is specifically tailored for interaction with HoprChannels and HoprToken contracts.
 * Additionally, it enables the transfer of native tokens to designated addresses,
 * while restricting the invocation of payable functions.
 *
 * Some difference between this library and the original `Permissions.sol` contract are:
 * - This library is designed to support a single role
 * - No `DelegateCall` is allowed
 * - Target must be one of the three types: Token, Channels, SEND
 * - Only scoped functions are allowed. No more wildcard
 * - Calling payable function is not allowed.
 * - When calling HoprChannels contracts, permission is check with multiple parameters together
 * - For Channels targets, the default permission is ALLOWED. However, the default value for other targets is BLOCKED.
 * - Permissions are not stored bitwise in `scopeConig` (uint256) due to lack of customization
 * - Utility functions, such as `packLeft`, `packRight`, `unpackFunction`, `unpackParameter`, `checkExecutionOptions`
 * are removed
 * - Specific helper functions, such as `pluckOneStaticAddress`, `pluckTwoStaticAddresses`, `pluckDynamicAddresses`,  `pluckSendPayload`
 * are derived from `pluckStaticValue` and `pluckDynamicValue`
 * - helper functions to encode array of function signatures and their respective permissions are added.
 *
 * @notice Due to the deployed HoprToken.sol imports OpenZeppelin contract library locked at v4.4.2, while
 * HoprChannels contract imports OpenZeppelin contract of v4.8.3, it's not possible to import both contracts
 * the same time without creating conflicts. Therefore, two method identifiers of HoprToken contract are
 * defined with value instead of `.selector`
 */
library HoprCapabilityPermissions {
    using TargetUtils for Target;
    using EnumerableTargetSet for TargetSet;

    // HoprChannels method ids (TargetType.CHANNELS)
    bytes4 public constant REDEEM_TICKET_SELECTOR = HoprChannels.redeemTicketSafe.selector;
    bytes4 public constant CLOSE_INCOMING_CHANNEL_SELECTOR = HoprChannels.closeIncomingChannelSafe.selector;
    bytes4 public constant INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR =
        HoprChannels.initiateOutgoingChannelClosureSafe.selector;
    bytes4 public constant FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR =
        HoprChannels.finalizeOutgoingChannelClosureSafe.selector;
    bytes4 public constant FUND_CHANNEL_SELECTOR = HoprChannels.fundChannelSafe.selector;
    // HoprToken method ids (TargetType.TOKEN). As HoprToken contract is in production, its ABI is static
    bytes4 public constant APPROVE_SELECTOR = IERC20.approve.selector;
    bytes4 public constant SEND_SELECTOR = IERC777.send.selector;

    event RevokedTarget(address indexed targetAddress);
    event ScopedTargetChannels(address indexed targetAddress, Target target);
    event ScopedTargetToken(address indexed targetAddress, Target target);
    event ScopedTargetSend(address indexed targetAddress, Target target);
    event ScopedGranularChannelCapability(
        address indexed targetAddress, bytes32 indexed channelId, bytes4 selector, GranularPermission permission
    );
    event ScopedGranularTokenCapability(
        address indexed nodeAddress,
        address indexed targetAddress,
        address indexed recipientAddress,
        bytes4 selector,
        GranularPermission permission
    );
    event ScopedGranularSendCapability(
        address indexed nodeAddress, address indexed recipientAddress, GranularPermission permission
    );

    /// Sender is not a member of the role
    error NoMembership();

    /// Arrays must be the same length
    error ArraysDifferentLength();

    /// Arrays must not exceed the maximum length
    error ArrayTooLong();

    /// Address cannot be zero
    error AddressIsZero();

    /// Function signature too short
    error FunctionSignatureTooShort();

    /// Role not allowed to delegate call to target address
    error DelegateCallNotAllowed();

    /// Role not allowed to call target address
    error TargetAddressNotAllowed();

    /// Role not allowed to send to target address
    error SendNotAllowed();

    /// Role not allowed to use bytes for parameter
    error ParameterNotAllowed();

    /// only multisend txs with an offset of 32 bytes are allowed
    error UnacceptableMultiSendOffset();

    /// The provided calldata for execution is too short, or an OutOfBounds scoped parameter was configured
    error CalldataOutOfBounds();

    // Default permission not acquired
    error DefaultPermissionRejected();

    // Granular permission not acquired
    error GranularPermissionRejected();

    // Node permission rejected
    error NodePermissionRejected();

    // Permission not properly configured
    error PermissionNotConfigured();

    // target is already scoped
    error TargetIsScoped();

    // target is not yet scoped
    error TargetIsNotScoped();

    // ======================================================
    // ---------------------- CHECKERS ----------------------
    // ======================================================

    /**
     * @dev Checks the permission of a transaction execution based on the role membership and transaction details.
     * @param role The storage reference to the Role struct.
     * @param multisend The address of the multisend contract.
     * @param to The recipient address of the transaction.
     * @param value The value of the transaction.
     * @param data The transaction data.
     * @param operation The operation type of the transaction.
     */
    function check(
        Role storage role,
        address multisend,
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation
    )
        internal
        view
    {
        if (multisend == to) {
            // here the operation should be delegate
            checkMultisendTransaction(role, data);
        } else {
            checkTransaction(role, to, value, data, operation);
        }
    }

    /**
     * @dev Splits a multisend data blob into transactions and forwards them to be checked.
     * @param role The storage reference to the Role struct.
     * @param data The packed transaction data (created by the `buildMultiSendSafeTx` utility function).
     */
    function checkMultisendTransaction(Role storage role, bytes memory data) internal view {
        Enum.Operation operation;
        address to;
        uint256 value;
        bytes memory out;
        uint256 dataLength;

        uint256 offset;
        assembly {
            offset := mload(add(data, 36))
        }
        if (offset != 32) {
            revert UnacceptableMultiSendOffset();
        }

        // transaction data (1st tx operation) reads at byte 100,
        // 4 bytes (multisend_id) + 32 bytes (offset_multisend_data) + 32 bytes multisend_data_length
        // increment i by the transaction data length
        // + 85 bytes of the to, value, and operation bytes until we reach the end of the data
        for (uint256 i = 100; i < data.length; i += (85 + dataLength)) {
            assembly {
                // First byte of the data is the operation.
                // We shift by 248 bits (256 - 8 [operation byte]) right since mload will always load 32 bytes (a word).
                // This will also zero out unused data.
                operation := shr(0xf8, mload(add(data, i)))
                // We offset the load address by 1 byte (operation byte)
                // We shift it right by 96 bits (256 - 160 [20 address bytes]) to right-align the data and zero out
                // unused data.
                to := shr(0x60, mload(add(data, add(i, 0x01))))
                // We offset the load address by 21 byte (operation byte + 20 address bytes)
                value := mload(add(data, add(i, 0x15)))
                // We offset the load address by 53 byte (operation byte + 20 address bytes + 32 value bytes)
                dataLength := mload(add(data, add(i, 0x35)))
                // load actual transaction data with an offset of 53 byte (operation byte + 20 address bytes + 32 value
                // bytes)
                out := add(data, add(i, 0x35))
            }
            checkTransaction(role, to, value, out, operation);
        }
    }

    /**
     * @dev Main transaction to check the permission of transaction execution of a module.
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the target contract.
     * @param value The value of the transaction.
     * @param data The transaction data.
     * @param operation The operation type of the transaction.
     */
    function checkTransaction(
        Role storage role,
        address targetAddress,
        uint256 value,
        bytes memory data,
        Enum.Operation operation
    )
        internal
        view
    {
        if (data.length != 0 && data.length < 4) {
            revert FunctionSignatureTooShort();
        }

        Target target = role.targets.get(targetAddress);

        // target is in scope; delegate call is not allowed; value can only be sent with `SEND`
        checkExecutionOptions(value, operation, target);

        bytes4 functionSig = bytes4(data);

        // check default permissions and get the fallback permission
        TargetPermission defaultPermission = getDefaultPermission(data.length, target, functionSig);
        // allow early revert or early return
        if (defaultPermission == TargetPermission.BLOCK_ALL) {
            revert DefaultPermissionRejected();
        } else if (defaultPermission == TargetPermission.ALLOW_ALL) {
            return;
        }

        GranularPermission granularPermission;
        // check function permission
        if (target.getTargetType() == TargetType.TOKEN) {
            // check with HoprToken contract
            granularPermission =
                checkHoprTokenParameters(role, keyForFunctions(targetAddress, functionSig), functionSig, data);
        } else if (target.getTargetType() == TargetType.CHANNELS) {
            // check with HoprChannels contract
            granularPermission =
                checkHoprChannelsParameters(role, keyForFunctions(targetAddress, functionSig), functionSig, data);
        } else if (target.getTargetType() == TargetType.SEND) {
            granularPermission = checkSendParameters(role, targetAddress);
        }

        // check permission result
        if (
            granularPermission == GranularPermission.BLOCK
                || (
                    granularPermission == GranularPermission.NONE
                        && defaultPermission == TargetPermission.SPECIFIC_FALLBACK_BLOCK
                )
        ) {
            revert GranularPermissionRejected();
        } else if (
            granularPermission == GranularPermission.ALLOW
                || (
                    granularPermission == GranularPermission.NONE
                        && defaultPermission == TargetPermission.SPECIFIC_FALLBACK_ALLOW
                )
        ) {
            return;
        } else {
            revert PermissionNotConfigured();
        }
    }

    /**
     * @dev Check if target is scoped; if the transaction can send along native tokens; if DelegatedCall is allowed.
     * @param value The value of the transaction.
     * @param operation The operation type of the transaction.
     * @param target The stored target
     */
    function checkExecutionOptions(uint256 value, Enum.Operation operation, Target target) internal pure {
        if (target.getTargetClearance() != Clearance.FUNCTION) {
            revert TargetAddressNotAllowed();
        }

        // delegate call is not allowed;
        if (operation == Enum.Operation.DelegateCall) {
            revert DelegateCallNotAllowed();
        }

        // send native tokens is only available to a set of addresses
        if (value > 0 && !target.isTargetType(TargetType.SEND)) {
            revert SendNotAllowed();
        }
    }

    /*
     * @dev Check parameters for HoprChannels capability
     * @param role reference to role storage
     * @param capabilityKey Key to the capability.
     * @param functionSig Function method ID
     * @param data payload (with function signature)
     */
    function checkHoprChannelsParameters(
        Role storage role,
        bytes32 capabilityKey,
        bytes4 functionSig,
        bytes memory data
    )
        internal
        view
        returns (GranularPermission)
    {
        // check the first two evm slots of data payload
        // according to the following ABIs
        //  - fundChannelSafe(address self, address account, Balance amount)  // src,dst
        //  - redeemTicketSafe(address self, RedeemableTicket calldata redeemable) // dst,channelId
        //  - initiateOutgoingChannelClosureSafe(address self, address destination) // src,dst
        //  - closeIncomingChannelSafe(address self, address source) // dst,src
        //  - finalizeOutgoingChannelClosureSafe(address self, address destination) // src,dst
        //  - setCommitmentSafe(address self, address source, bytes32 newCommitment) // dst,src
        address self = pluckOneStaticAddress(0, data);
        // the first slot should always store the self address
        if (self != msg.sender) {
            revert NodePermissionRejected();
        }

        bytes32 channelId;
        if (functionSig == REDEEM_TICKET_SELECTOR) {
            channelId = pluckOneBytes32(1, data);
        } else if (functionSig == CLOSE_INCOMING_CHANNEL_SELECTOR) {
            address source = pluckOneStaticAddress(1, data);
            channelId = getChannelId(source, self);
        } else if (
            functionSig == INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR
                || functionSig == FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR || functionSig == FUND_CHANNEL_SELECTOR
        ) {
            address destination = pluckOneStaticAddress(1, data);
            channelId = getChannelId(self, destination);
        } else {
            revert ParameterNotAllowed();
        }

        return role.capabilities[capabilityKey][channelId];
    }

    /*
     * @dev Will revert if a transaction has a parameter that is not allowed
     * @notice This function is invoked on non-HoprChannels contracts (i.e. HoprTokens)
     * @param role reference to role storage
     * @param capabilityKey Key to the capability.
     * @param functionSig Function method ID
     * @param data payload (with function signature)
     */
    function checkHoprTokenParameters(
        Role storage role,
        bytes32 capabilityKey,
        bytes4 functionSig,
        bytes memory data
    )
        internal
        view
        returns (GranularPermission)
    {
        // for APPROVE_SELECTOR the abi is (address, uint256)
        // for SEND_SELECTOR the abi is (address, uint256, bytes)
        // note that beneficiary could event be a CHANNELS target.
        // Calling send to a HoprChannels contract is equivalent to calling
        // fundChannel or fundChannelsMulti function. However, granular control is skipped!
        if (functionSig != APPROVE_SELECTOR && functionSig != SEND_SELECTOR) {
            revert ParameterNotAllowed();
        }
        address beneficiary = pluckOneStaticAddress(0, data);
        bytes32 pairId = getChannelId(msg.sender, beneficiary);
        return role.capabilities[capabilityKey][pairId];
    }

    /**
     * @dev Checks the parameters for sending native tokens.
     * @param role The Role storage instance.
     * @param targetAddress The target address for the send operation.
     */
    function checkSendParameters(Role storage role, address targetAddress) internal view returns (GranularPermission) {
        bytes32 pairId = getChannelId(msg.sender, targetAddress);
        return role.capabilities[bytes32(0)][pairId];
    }

    /**
     * @dev check the default target permission for target and for the function
     * returns the default permission
     * @param dataLength Length of data payload
     * @param target Taret of the operation
     * @param functionSig bytes4 method Id of the operation
     */
    function getDefaultPermission(
        uint256 dataLength,
        Target target,
        bytes4 functionSig
    )
        internal
        pure
        returns (TargetPermission)
    {
        // check default target permission
        TargetPermission defaultTargetPermission = target.getDefaultTargetPermission();
        // early return when the permission allows
        if (
            dataLength == 0 || functionSig == bytes4(0) || defaultTargetPermission == TargetPermission.ALLOW_ALL
                || defaultTargetPermission == TargetPermission.BLOCK_ALL
        ) {
            return defaultTargetPermission;
        }

        CapabilityPermission defaultFunctionPermission;
        if (functionSig == REDEEM_TICKET_SELECTOR) {
            defaultFunctionPermission = target.getDefaultCapabilityPermissionAt(0);
        } else if (functionSig == CLOSE_INCOMING_CHANNEL_SELECTOR) {
            defaultFunctionPermission = target.getDefaultCapabilityPermissionAt(2);
        } else if (functionSig == INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR) {
            defaultFunctionPermission = target.getDefaultCapabilityPermissionAt(3);
        } else if (functionSig == FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR) {
            defaultFunctionPermission = target.getDefaultCapabilityPermissionAt(4);
        } else if (functionSig == FUND_CHANNEL_SELECTOR) {
            defaultFunctionPermission = target.getDefaultCapabilityPermissionAt(5);
        } else if (functionSig == APPROVE_SELECTOR) {
            defaultFunctionPermission = target.getDefaultCapabilityPermissionAt(7);
        } else if (functionSig == SEND_SELECTOR) {
            defaultFunctionPermission = target.getDefaultCapabilityPermissionAt(8);
        } else {
            revert ParameterNotAllowed();
        }
        // only when function permission is not defined, use target default permission
        if (defaultFunctionPermission == CapabilityPermission.NONE) {
            return defaultTargetPermission;
        } else {
            return TargetUtils.convertFunctionToTargetPermission(defaultFunctionPermission);
        }
    }

    // ======================================================
    // ----------------------- SETTERS ----------------------
    // ======================================================

    /**
     * @dev Revokes the target address from the Role by setting its clearance and target type to None.
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the target to be revoked.
     */
    function revokeTarget(Role storage role, address targetAddress) internal {
        bool result = role.targets.remove(targetAddress);
        if (result) {
            emit RevokedTarget(targetAddress);
        } else {
            revert TargetIsNotScoped();
        }
    }

    /**
     * @dev Allows the target address to be scoped as a HoprToken (TOKEN)
     * by setting its clearance and target type accordingly.
     * @param role The storage reference to the Role struct.
     * @param target target to be scoped as a beneficiary of SEND.
     */
    function scopeTargetToken(Role storage role, Target target) internal {
        address targetAddress = target.getTargetAddress();
        if (targetAddress == address(0)) {
            revert AddressIsZero();
        }
        // check targetAddress is not scoped
        if (role.targets.contains(targetAddress)) {
            revert TargetIsScoped();
        }

        // force overwrite irrelevant defaults
        Target updatedTarget = target.forceWriteAsTargetType(TargetType.TOKEN);
        role.targets.add(updatedTarget);

        emit ScopedTargetToken(targetAddress, updatedTarget);
    }

    /**
     * @dev Allows the target address to be scoped as a HoprChannels contract (CHANNELS)
     * by setting its clearance and target type accordingly.
     * @param role The storage reference to the Role struct.
     * @param target target to be scoped as a beneficiary of SEND.
     */
    function scopeTargetChannels(Role storage role, Target target) internal {
        address targetAddress = target.getTargetAddress();
        if (targetAddress == address(0)) {
            revert AddressIsZero();
        }
        // check targetAddress is not scoped
        if (role.targets.contains(targetAddress)) {
            revert TargetIsScoped();
        }
        // force overwrite irrelevant defaults
        Target updatedTarget = target.forceWriteAsTargetType(TargetType.CHANNELS);
        role.targets.add(updatedTarget);

        emit ScopedTargetChannels(targetAddress, updatedTarget);
    }

    /**
     * @dev Allows the target address to be scoped as a beneficiary of SEND by setting its clearance and target type
     * accordingly.
     * @notice It overwrites the irrelevant fields in DefaultPermissions struct
     * @param role The storage reference to the Role struct.
     * @param target target to be scoped as a beneficiary of SEND.
     */
    function scopeTargetSend(Role storage role, Target target) internal {
        address targetAddress = target.getTargetAddress();
        if (targetAddress == address(0)) {
            revert AddressIsZero();
        }
        // check targetAddress is not scoped
        if (role.targets.contains(targetAddress)) {
            revert TargetIsScoped();
        }

        // force overwrite irrelevant defaults
        Target updatedTarget = target.forceWriteAsTargetType(TargetType.SEND);
        role.targets.add(updatedTarget);

        emit ScopedTargetSend(targetAddress, updatedTarget);
    }

    /**
     * @dev Sets permissions for a set of max. 7 functions on a scoped CHANNELS target.
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the scoped CHANNELS target.
     * @param channelId The channelId of the scoped CHANNELS target.
     * @param encodedSigsPermissions encoded permission using encodeFunctionSigsAndPermissions
     */
    function scopeChannelsCapabilities(
        Role storage role,
        address targetAddress,
        bytes32 channelId,
        bytes32 encodedSigsPermissions
    )
        internal
    {
        (bytes4[] memory functionSigs, GranularPermission[] memory permissions) =
            HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedSigsPermissions, 7);

        for (uint256 i = 0; i < 7; i++) {
            if (functionSigs[i] != bytes4(0)) {
                bytes32 capabilityKey = keyForFunctions(targetAddress, functionSigs[i]);
                role.capabilities[capabilityKey][channelId] = permissions[i];

                emit ScopedGranularChannelCapability(targetAddress, channelId, functionSigs[i], permissions[i]);
            }
        }
    }

    /**
     * @dev Sets the permission for a specific function on a scoped TOKEN target.
     * @notice As only two function signatures are allowed, the length is set to 2
     * @param role The storage reference to the Role struct.
     * @param nodeAddress The address of the caller node.
     * @param targetAddress The address of the scoped TOKEN target.
     * @param beneficiary The beneficiary address for the scoped TOKEN target.
     * @param encodedSigsPermissions encoded permission using encodeFunctionSigsAndPermissions
     */
    function scopeTokenCapabilities(
        Role storage role,
        address nodeAddress,
        address targetAddress,
        address beneficiary,
        bytes32 encodedSigsPermissions
    )
        internal
    {
        (bytes4[] memory functionSigs, GranularPermission[] memory permissions) =
            HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedSigsPermissions, 2);

        for (uint256 i = 0; i < 2; i++) {
            if (functionSigs[i] != bytes4(0)) {
                bytes32 capabilityKey = keyForFunctions(targetAddress, functionSigs[i]);
                role.capabilities[capabilityKey][getChannelId(nodeAddress, targetAddress)] = permissions[i];

                emit ScopedGranularTokenCapability(
                    nodeAddress, targetAddress, beneficiary, functionSigs[i], permissions[i]
                );
            }
        }
    }

    /**
     * @dev Sets the permission for sending native tokens to a specific beneficiary
     * @notice The capability ID for sending native tokens is bytes32(0x00)
     * @param nodeAddress The address of the caller node
     * @param beneficiary The beneficiary address for the scoped SEND target.
     * @param permission The permission to be set for the specific function.
     */
    function scopeSendCapability(
        Role storage role,
        address nodeAddress,
        address beneficiary,
        GranularPermission permission
    )
        internal
    {
        role.capabilities[bytes32(0)][getChannelId(nodeAddress, beneficiary)] = permission;

        emit ScopedGranularSendCapability(nodeAddress, beneficiary, permission);
    }

    // ======================================================
    // ----------------------- HELPERS ----------------------
    // ======================================================

    function getChannelId(address source, address destination) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(source, destination));
    }

    /**
     * @dev Retrieves a static address value from the given `data` byte array at the specified `index`.
     * @param data The byte array containing the data.
     * @param index The index of the static address value to retrieve.
     * @return addr The static address value at the specified index.
     */
    function pluckOneStaticAddress(uint256 index, bytes memory data) internal pure returns (address) {
        // pre-check: is there a word available for the current parameter at argumentsBlock?
        if (data.length < 4 + index * 32 + 32) {
            revert CalldataOutOfBounds();
        }

        uint256 offset = 4 + index * 32;
        address addr;
        assembly {
            // add 32 - jump over the length encoding of the data bytes array
            addr := mload(add(32, add(data, offset)))
        }
        return addr;
    }

    /**
     * @dev Extracts one bytes32 from the `data` byte array.
     * @param data The byte array containing the bytes32.
     * @param index The index of the static bytes32 value to retrieve.
     * @return by32 The second bytes32 extracted from the `data` byte array.
     */
    function pluckOneBytes32(uint256 index, bytes memory data) internal pure returns (bytes32) {
        // pre-check: is there a word available for the current parameter at argumentsBlock?
        if (data.length < 4 + index * 32 + 32) {
            revert CalldataOutOfBounds();
        }

        uint256 offset = 4 + index * 32;
        bytes32 by32;
        assembly {
            // add 32 - jump over the length encoding of the data bytes array
            by32 := mload(add(32, add(data, offset)))
        }
        return by32;
    }

    /**
     * @dev Returns the unique key for a function of a given `targetAddress`.
     * @param targetAddress The address of the target contract.
     * @param functionSig The function signature of the target function.
     * @return key The unique key representing the target function.
     */
    function keyForFunctions(address targetAddress, bytes4 functionSig) internal pure returns (bytes32) {
        return bytes32(abi.encodePacked(targetAddress, functionSig));
    }

    /**
     * @dev Returns arrays of bytes32 that concates function signatures (bytes4 = 32 bits)
     * together with granular permissions (per channel id or per beneficiary) (2 bits)
     * It can take maxinum 7 sets (256 / (32 + 2) ~= 7) of function signatures and permissions
     * @notice Signature encoding is right-padded; Index 0 is the left most and grows to the right
     * Permission encoding is left-padded; Index grows from right to the left.
     * Returns a bytes32 and length of sigature and permissions
     * @param functionSigs array of function signatures on target
     * @param permissions array of granular permissions on target
     */
    function encodeFunctionSigsAndPermissions(
        bytes4[] memory functionSigs,
        GranularPermission[] memory permissions
    )
        internal
        pure
        returns (bytes32 encoded, uint256 length)
    {
        uint256 len = functionSigs.length;
        if (len > 7) {
            revert ArrayTooLong();
        }
        if (functionSigs.length != permissions.length) {
            revert ArraysDifferentLength();
        }

        bytes32 val;
        // add function signatures
        for (uint256 i = 0; i < len; i++) {
            // first right shift (32 - 4) * 8 = 224 bits
            // then left shift (32 - 4 * i - 4) * 8 = (224 - 32 * i) bits
            val |= (bytes32(functionSigs[i]) >> 224) << (224 - (32 * i));
        }
        for (uint256 i = 0; i < len; i++) {
            // shift by two bits
            val |= bytes32(uint256(permissions[i])) << (2 * i);
        }
        return (val, len);
    }

    /**
     * @dev Returns an bytes4 array which decodes from the combined encoding
     * of function signature and permissions. It can take maxinum 7 items.
     * Encoding of function signatures is right-padded, where indexes grow from right to left
     * Encoding of permissions is left-padded, where indexes grow from left to right
     * @param encoded encode permissions in bytes32
     * @param length length of permissions
     */
    function decodeFunctionSigsAndPermissions(
        bytes32 encoded,
        uint256 length
    )
        internal
        pure
        returns (bytes4[] memory functionSigs, GranularPermission[] memory permissions)
    {
        if (length > 7) {
            revert ArrayTooLong();
        }
        functionSigs = new bytes4[](length);
        permissions = new GranularPermission[](length);
        // decode function signature
        for (uint256 i = 0; i < length; i++) {
            // first right shift (32 - 4 * i - 4) * 8 = (224 - 32 * i) bits
            // then left shift (32 - 4) * 8 = 224 bits
            functionSigs[i] = bytes4((encoded >> (224 - (32 * i))) << 224);
        }
        // decode permissions
        for (uint256 j = 0; j < length; j++) {
            // first left shift 256 - 2 - 2 * j = 254 - 2 * j bits
            // then right shift 256 - 2 = 254 bits
            permissions[j] = GranularPermission(uint8((uint256(encoded) << (254 - (2 * j))) >> 254));
        }
    }
}
