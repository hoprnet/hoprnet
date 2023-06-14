// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity >=0.8.0 <0.9.0;

import "safe-contracts/common/Enum.sol";
import "../../Channels.sol";

enum Clearance {
    NONE,
    FUNCTION
}

enum TargetType {
    NONE,
    TOKEN,
    CHANNELS,
    SEND
}

enum Permission {
    NONE,
    ALLOW_ALL,
    BLOCK_ALL,
    SPECIFIC_FALLBACK_ALLOW,
    SPECIFIC_FALLBACK_BLOCK
}

enum GranularPermission {
    NONE,
    ALLOW,
    BLOCK
}

struct DefaultPermissions {
    Permission defaultTargetPermission; // uint8. Default permission for the target
    Permission defaultRedeemTicketSafeFunctionPermisson; // uint8 (permission for Channels contract)
    Permission defaultBatchRedeemTicketsSafeFunctionPermisson; // uint8 (permission for Channels contract)
    Permission defaultCloseIncomingChannelSafeFunctionPermisson; // uint8 (permission for Channels contract)
    Permission defaultInitiateOutgoingChannelClosureSafeFunctionPermisson; // uint8 (permission for Channels contract)
    Permission defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson; // uint8 (permission for Channels contract)
    Permission defaultFundChannelMultiFunctionPermisson; // uint8 (permission for Channels contract)
    Permission defaultSetCommitmentSafeFunctionPermisson; // uint8 (permission for Channels contract)
    Permission defaultApproveFunctionPermisson; // uint8 (permission for Token contract)
    Permission defaultSendFunctionPermisson; // uint8 (permission for Token contract)
}

struct TargetAddress {
    Clearance clearance;    // uint8
    TargetType targetType;  // uint8
    DefaultPermissions defaultPermissions; // uint80
}

struct Role {
    mapping(address => bool) members;   // eligible caller. May be able to receive native tokens (e.g. xDAI), if set to allowed
    mapping(address => TargetAddress) targets;  // target addresses that can be called
    // For CHANNELS target: keyForFunctions (bytes32) => channel Id (keccak256(src, dest)) => GranularPermission
    // For TOKEN target: keyForFunctions (bytes32) => recipient Id (address in bytes32) => GranularPermission
    // For SEND target:  bytes32(0x00) => recipient Id (address in bytes32) => GranularPermission
    mapping(bytes32 => mapping(bytes32 => GranularPermission)) capabilities; 
}

/**
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
 * - Utility functions, such as `packLeft`, `packRight`, `unpackFunction`, `unpackParameter`, `checkExecutionOptions` are removed
 * - Specific helper functions, such as `pluckOneStaticAddress`, `pluckTwoStaticAddresses`, `pluckDynamicAddresses`,  `pluckSendPayload` are derived from `pluckStaticValue` and `pluckDynamicValue`
 * - helper functions to encode array of function signatures and their respective permissions are added.
 *
 * @notice Due to the deployed HoprToken.sol imports OpenZeppelin contract library locked at v4.4.2, while
 * HoprChannels contract imports OpenZeppelin contract of v4.8.3, it's not possible to import both contracts
 * the same time without creating conflicts. Therefore, two method identifiers of HoprToken contract are
 * defined with value instead of `.selector`
 */
library HoprCapabilityPermissions {
    // HoprChannels method ids (TargetType.CHANNELS)
    bytes4 internal constant REDEEM_TICKET_SELECTOR = HoprChannels.redeemTicketSafe.selector;
    bytes4 internal constant BATCH_REDEEM_TICKETS_SELECTOR = HoprChannels.batchRedeemTicketsSafe.selector;
    bytes4 internal constant CLOSE_INCOMING_CHANNEL_SELECTOR = HoprChannels.closeIncomingChannelSafe.selector;
    bytes4 internal constant INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR = HoprChannels.initiateOutgoingChannelClosureSafe.selector;
    bytes4 internal constant FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR = HoprChannels.finalizeOutgoingChannelClosureSafe.selector;
    bytes4 internal constant FUND_CHANNEL_MULTI_SELECTOR = HoprChannels.fundChannelMulti.selector;
    bytes4 internal constant SET_COMMITMENT_SELECTOR = HoprChannels.setCommitmentSafe.selector;
    // HoprToken method ids (TargetType.TOKEN). As HoprToken contract is in production, its ABI is static
    bytes4 internal constant APPROVE_SELECTOR = hex"095ea7b3"; // equivalent to `HoprToken.approve.selector`, for ABI "approve(address,uint256)"
    bytes4 internal constant SEND_SELECTOR = hex"9bd9bbc6"; // equivalent to `HoprToken.send.selector`, for ABI "send(address,uint256,bytes)"

    event RevokedTarget(address indexed targetAddress);
    event ScopedTarget(address indexed targetAddress, TargetType targetType, DefaultPermissions defaultPermission);
    event ScopedGranularChannelCapability(
        address indexed targetAddress,
        bytes32 indexed channelId,
        bytes4 selector,
        GranularPermission permission
    );
    event ScopedGranularTokenCapability(
        address indexed targetAddress,
        address indexed recipientAddress,
        bytes4 selector,
        GranularPermission permission
    );
    event ScopedGranularSendCapability(
        address indexed recipientAddress,
        GranularPermission permission
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

    /// Role not allowed to call target when its type is not set
    error TargetTypeNotSet();

    /// Role not allowed to send to target address
    error SendNotAllowed();

    /// Role not allowed to use bytes for parameter
    error ParameterNotAllowed();

    /// only multisend txs with an offset of 32 bytes are allowed
    error UnacceptableMultiSendOffset();

    /// The provided calldata for execution is too short, or an OutOfBounds scoped parameter was configured
    error CalldataOutOfBounds();

    // Permission not acquired
    error PermissionRejected();


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
    ) internal view {
        if (!role.members[msg.sender]) {
            revert NoMembership();
        }
        if (multisend == to) {
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
    function checkMultisendTransaction(
        Role storage role,
        bytes memory data
    ) internal view {
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
                // We shift it right by 96 bits (256 - 160 [20 address bytes]) to right-align the data and zero out unused data.
                to := shr(0x60, mload(add(data, add(i, 0x01))))
                // We offset the load address by 21 byte (operation byte + 20 address bytes)
                value := mload(add(data, add(i, 0x15)))
                // We offset the load address by 53 byte (operation byte + 20 address bytes + 32 value bytes)
                dataLength := mload(add(data, add(i, 0x35)))
                // We offset the load address by 85 byte (operation byte + 20 address bytes + 32 value bytes + 32 data length bytes)
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
    ) internal view {
        if (data.length != 0 && data.length < 4) {
            revert FunctionSignatureTooShort();
        }

        TargetAddress storage target = role.targets[targetAddress];
        if (target.clearance != Clearance.FUNCTION) {
            revert TargetAddressNotAllowed();
        }

        // delegate call is not allowed; value can only be sent with `SEND`
        checkExecutionOptions(value, operation, target.targetType);

        // check default target permission
        Permission defaultTargetPermission = target.defaultPermissions.defaultTargetPermission;
        if ( defaultTargetPermission == Permission.NONE || defaultTargetPermission == Permission.BLOCK_ALL) {
            revert PermissionRejected();
        } else if (defaultTargetPermission == Permission.ALLOW_ALL) {
            return;
        }

        // check function permission
        if (target.targetType == TargetType.TOKEN) {
            // check with HoprToken contract
            checkHoprTokenParameters(role, targetAddress, target.defaultPermissions, data);
            return;
        } else if (target.targetType == TargetType.CHANNELS) {
            // check with HoprChannels contract
            checkHoprChannelsParameters(role, targetAddress, target.defaultPermissions, data);
            return;
        } else if (target.targetType == TargetType.SEND) {
            checkSendParameters(role, targetAddress, data.length, target.defaultPermissions);
            return;
        }
    }

    // function getDefaultPermissions(
    //     DefaultPermissions defaultPermission, 
    //     bytes4 functionSig
    // ) internal view returns () {

    // }

    /**
     * @dev Check if the transaction can send along native tokens and if DelegatedCall is allowed.
     * @param value The value of the transaction.
     * @param operation The operation type of the transaction.
     * @param targetType The target type of the transaction.
     */
    function checkExecutionOptions(
        uint256 value,
        Enum.Operation operation,
        TargetType targetType
    ) internal pure {
         // delegate call is not allowed; 
        if (
            operation == Enum.Operation.DelegateCall
        ) {
            revert DelegateCallNotAllowed();
        }
        
        // send native tokens is only available to a set of addresses
        if (
            value > 0 &&
            targetType != TargetType.SEND
        ) {
            revert SendNotAllowed();
        }

        if (targetType == TargetType.NONE) {
            revert TargetTypeNotSet();
        }
    }

    /*
     * @dev Check parameters for HoprChannels capability
     * @param role reference to role storage
     * @param scopeConfig reference to role storage
     * @param targetAddress Address to check.
     * @param data the transaction data to check
     */
    function checkHoprChannelsParameters(
        Role storage role,
        address targetAddress,
        DefaultPermissions storage defaultPermissions,
        bytes memory data
    ) internal view {
        bytes4 functionSig = bytes4(data);
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);
        Permission defaultFunctionPermission;
        bytes32 channelId;

        if (functionSig == REDEEM_TICKET_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultRedeemTicketSafeFunctionPermisson;
            (address self, HoprChannels.TicketData memory ticketData) = abi.decode(sliceFrom(data, 4), (address, HoprChannels.TicketData));
            channelId = ticketData.channelId;
        } else if (functionSig == BATCH_REDEEM_TICKETS_SELECTOR) {
            // loop // TODO:
        } else if (functionSig == CLOSE_INCOMING_CHANNEL_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultCloseIncomingChannelSafeFunctionPermisson;
            (address self, address source) = abi.decode(sliceFrom(data, 4), (address, address));
            channelId = getChannelId(source, self);
        } else if (functionSig == INITIATE_OUTGOING_CHANNEL_CLOSURE_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultInitiateOutgoingChannelClosureSafeFunctionPermisson;
            (address self, address destination) = abi.decode(sliceFrom(data, 4), (address, address));
            channelId = getChannelId(self, destination);
        } else if (functionSig == FINALIZE_OUTGOING_CHANNEL_CLOSURE_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson;
            (address self, address destination) = abi.decode(sliceFrom(data, 4), (address, address));
            channelId = getChannelId(self, destination);
        } else if (functionSig == FUND_CHANNEL_MULTI_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultFundChannelMultiFunctionPermisson;
            (, , address source,) = abi.decode(sliceFrom(data, 4), (address, HoprChannels.Balance, address, HoprChannels.Balance));
            // TODO:
        } else if (functionSig == SET_COMMITMENT_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultSetCommitmentSafeFunctionPermisson;
            (address self, , address source) = abi.decode(sliceFrom(data, 4), (address, bytes32, address));
            channelId = getChannelId(source, self);
        } else {
            revert ParameterNotAllowed();
        }
        // TODO: compare channelId permission and function permission
    }

    /*
     * @dev Will revert if a transaction has a parameter that is not allowed
     * @notice This function is invoked on non-HoprChannels contracts (i.e. HoprTokens)
     * @param role reference to role storage
     * @param scopeConfig reference to role storage
     * @param targetAddress Address to check.
     * @param data the transaction data to check
     */
    function checkHoprTokenParameters(
        Role storage role,
        address targetAddress,
        DefaultPermissions storage defaultPermissions,
        bytes memory data
    ) internal view {
        bytes4 functionSig = bytes4(data);
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);
        Permission defaultFunctionPermission;
        // // check if the first parameter is allowed
        // address beneficiary = pluckOneStaticAddress(data, 0);  
        // if (role.hoprTokenCapability[capabilityKey][beneficiary] != HoprTokenPermission.Allowed) {
        //     // not allowed to call the capability
        //     revert ParameterNotAllowed();
        // }

        // // if calling `send` method, it is equivalent to calling FUND_CHANNEL_MULTI_SELECTOR
        // if (functionSig == SEND_SELECTOR) {
        //     bytes32 sendCapabilityKey = keyForFunctions(targetAddress, FUND_CHANNEL_MULTI_SELECTOR);
        //     // source and channel destination addreses are at the first and second places respectively
        //     (address src, address dest) = pluckSendPayload(data, 2);
        //     // check if functions on this channel can be called.
        //     compareHoprChannelsPermission(role, sendCapabilityKey, src, dest);
        // }

        if (functionSig == APPROVE_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultApproveFunctionPermisson;
            // TODO: get beneficiary address
            // (address self, HoprChannels.TicketData memory ticketData) = abi.decode(sliceFrom(data, 4), (address, HoprChannels.TicketData));
            // channelId = ticketData.channelId;
        } else if (functionSig == SEND_SELECTOR) {
            defaultFunctionPermission = defaultPermissions.defaultSendFunctionPermisson;
            // TODO: check extract channel Id from send payload 
        } else {
            revert ParameterNotAllowed();
        }
        // TODO: compare beneficiary permission and function permission
    }

    /**
     * @dev Checks the parameters for sending native tokens.
     * @param role The Role storage instance.
     * @param targetAddress The target address for the send operation.
     * @param dataLength The length of the data associated with the send operation.
     */
    function checkSendParameters(
        Role storage role,
        address targetAddress,
        uint256 dataLength,
        DefaultPermissions storage defaultPermissions
    ) internal view {
        if (
            role.capabilities[bytes32(0)][bytes32(uint256(uint160(targetAddress)))] == GranularPermission.BLOCK || 
            (defaultPermissions.defaultTargetPermission == Permission.SPECIFIC_FALLBACK_BLOCK && role.capabilities[bytes32(0)][bytes32(uint256(uint160(targetAddress)))] == GranularPermission.NONE)
        ) {
            // not allowed to send
            revert SendNotAllowed();
        }
        if (dataLength > 0) {
            // not allowed to call payable functions
            revert ParameterNotAllowed();
        }
    }

    // /** FIXME:
    //  * @dev Compares the permission for calling a HoprChannels contract.
    //  * @param role The storage reference to the Role struct.
    //  * @param capabilityKey The key representing the capability.
    //  * @param source The source address of the HOPR channel.
    //  * @param destination The destination address of the HOPR channel.
    //  */
    // function compareHoprChannelsPermission(
    //     Role storage role, 
    //     bytes32 capabilityKey, 
    //     address source, 
    //     address destination
    // ) internal view {
    //     // get channelId
    //     bytes32 channelId = keccak256(abi.encodePacked(source, destination));
    //     // check if it's allowed to call the channel
    //     if (role.hoprChannelsCapability[capabilityKey][channelId] != HoprChannelsPermission.Allowed) {
    //         // not allowed to call the capability
    //         revert ParameterNotAllowed();
    //     }
    // }

    // ======================================================
    // ----------------------- SETTERS ----------------------
    // ======================================================
    
    /**
     * @dev Revokes the target address from the Role by setting its clearance and target type to None.
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the target to be revoked.
     */
    function revokeTarget(
        Role storage role,
        address targetAddress
    ) external {
        role.targets[targetAddress] = TargetAddress({
            clearance: Clearance.NONE,
            targetType: TargetType.NONE,
            defaultPermissions: DefaultPermissions({
                defaultTargetPermission: Permission.NONE,
                defaultRedeemTicketSafeFunctionPermisson: Permission.NONE,
                defaultBatchRedeemTicketsSafeFunctionPermisson: Permission.NONE,
                defaultCloseIncomingChannelSafeFunctionPermisson: Permission.NONE,
                defaultInitiateOutgoingChannelClosureSafeFunctionPermisson: Permission.NONE,
                defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson: Permission.NONE,
                defaultFundChannelMultiFunctionPermisson: Permission.NONE,
                defaultSetCommitmentSafeFunctionPermisson: Permission.NONE,
                defaultApproveFunctionPermisson: Permission.NONE,
                defaultSendFunctionPermisson: Permission.NONE
            })
        });
        emit RevokedTarget(targetAddress);
    }

    /**
     * @dev Allows the target address to be scoped as a HoprToken (TOKEN) 
     * by setting its clearance and target type accordingly.
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the target to be scoped as a TOKEN target.
     * @param defaultPermissions The default permissions for the TOKEN target
     */
    function scopeTargetToken(
        Role storage role,
        address targetAddress,
        DefaultPermissions memory defaultPermissions
    ) external {
        _scopeTarget(
            role, 
            targetAddress, 
            TargetType.TOKEN,
            DefaultPermissions({
                defaultTargetPermission: Permission.NONE,
                defaultRedeemTicketSafeFunctionPermisson: Permission.NONE,
                defaultBatchRedeemTicketsSafeFunctionPermisson: Permission.NONE,
                defaultCloseIncomingChannelSafeFunctionPermisson: Permission.NONE,
                defaultInitiateOutgoingChannelClosureSafeFunctionPermisson: Permission.NONE,
                defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson: Permission.NONE,
                defaultFundChannelMultiFunctionPermisson: Permission.NONE,
                defaultSetCommitmentSafeFunctionPermisson: Permission.NONE,
                defaultApproveFunctionPermisson: defaultPermissions.defaultApproveFunctionPermisson,
                defaultSendFunctionPermisson: defaultPermissions.defaultSendFunctionPermisson
            })
        );
    }

    /**
     * @dev Allows the target address to be scoped as a HoprChannels contract (CHANNELS)
     * by setting its clearance and target type accordingly.
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the target to be scoped as a CHANNELS target
     * @param defaultPermissions The default permissions for the CHANNELS target
     */
    function scopeTargetChannels(
        Role storage role,
        address targetAddress,
        DefaultPermissions memory defaultPermissions
    ) external {
        _scopeTarget(
            role, 
            targetAddress, 
            TargetType.CHANNELS,
            DefaultPermissions({
                defaultTargetPermission: defaultPermissions.defaultTargetPermission,
                defaultRedeemTicketSafeFunctionPermisson: defaultPermissions.defaultRedeemTicketSafeFunctionPermisson,
                defaultBatchRedeemTicketsSafeFunctionPermisson: defaultPermissions.defaultBatchRedeemTicketsSafeFunctionPermisson,
                defaultCloseIncomingChannelSafeFunctionPermisson: defaultPermissions.defaultCloseIncomingChannelSafeFunctionPermisson,
                defaultInitiateOutgoingChannelClosureSafeFunctionPermisson: defaultPermissions.defaultInitiateOutgoingChannelClosureSafeFunctionPermisson,
                defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson: defaultPermissions.defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson,
                defaultFundChannelMultiFunctionPermisson: defaultPermissions.defaultFundChannelMultiFunctionPermisson,
                defaultSetCommitmentSafeFunctionPermisson: defaultPermissions.defaultSetCommitmentSafeFunctionPermisson,
                defaultApproveFunctionPermisson: Permission.NONE,
                defaultSendFunctionPermisson: Permission.NONE
            })
        );
    }

    /**
     * @dev Allows the target address to be scoped as a beneficiary of SEND by setting its clearance and target type accordingly.
     * @notice It overwrites the irrelevant fields in DefaultPermissions struct
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the target to be scoped as a beneficiary of SEND.
     * @param defaultPermissions The default permissions for the SEND target
     */
    function scopeTargetSend(
        Role storage role,
        address targetAddress,
        DefaultPermissions memory defaultPermissions
    ) external {
        _scopeTarget(
            role, 
            targetAddress, 
            TargetType.SEND,
            DefaultPermissions({
                defaultTargetPermission: Permission.NONE,
                defaultRedeemTicketSafeFunctionPermisson: Permission.NONE,
                defaultBatchRedeemTicketsSafeFunctionPermisson: Permission.NONE,
                defaultCloseIncomingChannelSafeFunctionPermisson: Permission.NONE,
                defaultInitiateOutgoingChannelClosureSafeFunctionPermisson: Permission.NONE,
                defaultFinalizeOutgoingChannelClosureSafeFunctionPermisson: Permission.NONE,
                defaultFundChannelMultiFunctionPermisson: Permission.NONE,
                defaultSetCommitmentSafeFunctionPermisson: Permission.NONE,
                defaultApproveFunctionPermisson: Permission.NONE,
                defaultSendFunctionPermisson: Permission.NONE
            })
        );
    }

    /**
     * @dev Allows the target address to be scoped with a perset permissions.
     * @param _role The storage reference to the Role struct.
     * @param _targetAddress The address of the target to be scoped
     * @param _targetType The type of the target
     * @param _defaultPermissions The default permissions for target
     */
    function _scopeTarget(
        Role storage _role,
        address _targetAddress,
        TargetType _targetType,
        DefaultPermissions memory _defaultPermissions
    ) private {
        if (_targetAddress == address(0)) {
            revert AddressIsZero();
        }
        _role.targets[_targetAddress] = TargetAddress({
            clearance: Clearance.FUNCTION,
            targetType: _targetType,
            defaultPermissions: _defaultPermissions
        });
        emit ScopedTarget(_targetAddress, _targetType, _defaultPermissions);
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
    ) external {
        (bytes4[] memory functionSigs, GranularPermission[] memory permissions) = HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedSigsPermissions, 7);

        for (uint256 i = 0; i < 7; i++) {
            if (functionSigs[i] != bytes4(0)) {
                bytes32 capabilityKey = keyForFunctions(targetAddress, functionSigs[i]);
                role.capabilities[capabilityKey][channelId] = permissions[i];

                emit ScopedGranularChannelCapability(
                    targetAddress,
                    channelId,
                    functionSigs[i],
                    permissions[i]
                );
            }
        }
    }

    /**
     * @dev Sets the permission for a specific function on a scoped TOKEN target.
     * @notice As only two function signatures are allowed, the length is set to 2
     * @param role The storage reference to the Role struct.
     * @param targetAddress The address of the scoped TOKEN target.
     * @param beneficiary The beneficiary address for the scoped TOKEN target.
     * @param encodedSigsPermissions encoded permission using encodeFunctionSigsAndPermissions
     */
    function scopeTokenCapabilities(
        Role storage role,
        address targetAddress,
        address beneficiary,
        bytes32 encodedSigsPermissions
    ) external {
        (bytes4[] memory functionSigs, GranularPermission[] memory permissions) = HoprCapabilityPermissions.decodeFunctionSigsAndPermissions(encodedSigsPermissions, 2);

        for (uint256 i = 0; i < 2; i++) {
            if (functionSigs[i] != bytes4(0)) {
                bytes32 capabilityKey = keyForFunctions(targetAddress, functionSigs[i]);
                role.capabilities[capabilityKey][bytes32(uint256(uint160(beneficiary)))] = permissions[i];

                emit ScopedGranularTokenCapability(
                    targetAddress,
                    beneficiary,
                    functionSigs[i],
                    permissions[i]
                );
            }
        }
    }

    /**
     * @dev Sets the permission for sending native tokens to a specific beneficiary
     * @notice The capability ID for sending native tokens is bytes32(0x00)
     * @param beneficiary The beneficiary address for the scoped SEND target.
     * @param permission The permission to be set for the specific function.
     */
    function scopeSendCapability(
        Role storage role,
        address beneficiary,
        GranularPermission permission
    ) external {
        role.capabilities[bytes32(0)][bytes32(uint256(uint160(beneficiary)))] = permission;

        emit ScopedGranularSendCapability(
            beneficiary,
            permission
        );
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
    function pluckOneStaticAddress(
        bytes memory data,
        uint256 index
    ) internal pure returns (address) {
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
     * @dev Extracts two addresses from the `data` byte array.
     * @param data The byte array containing the addresses.
     * @return addr0 The first address extracted from the `data` byte array.
     * @return addr1 The second address extracted from the `data` byte array.
     */
    function pluckTwoStaticAddresses(
        bytes memory data
    ) internal pure returns (address, address) {
        // pre-check: is there a word available for the current parameter at argumentsBlock?
        if (data.length < 4 + 1 * 32 + 32) {
            revert CalldataOutOfBounds();
        }
        address addr0;
        address addr1;
        assembly {
            // add 32 - jump over the length encoding of the data bytes array
            // offset0 = 4 + 0 * 32;
            addr0 := mload(add(32, add(data, 4)))
            // offset1 = 4 + 1 * 32;
            addr1 := mload(add(32, add(data, 36)))
        }
        return (addr0, addr1);
    }

    /**
     * @dev Returns an array of dynamically sized addresses decoded from a portion of the `data` byte array.
     * @param data The byte array containing the encoded addresses.
     * @param index The index of the parameter in the `data` byte array.
     * @return decodedAddresses An array of decoded addresses.
     */
    function pluckDynamicAddresses(
        bytes memory data,
        uint256 index
    ) internal pure returns (address[] memory decodedAddresses) {
        // pre-check: is there a word available for the current parameter at argumentsBlock?
        if (data.length < 4 + index * 32 + 32) {
            revert CalldataOutOfBounds();
        }

        // the start of the parameter block
        // 32 bytes - length encoding of the data bytes array
        // 4  bytes - function sig
        uint256 argumentsBlock;
        assembly {
            argumentsBlock := add(data, 36)
        }

        // the two offsets are relative to argumentsBlock
        uint256 offset = index * 32;
        uint256 offsetPayload;
        assembly {
            offsetPayload := mload(add(argumentsBlock, offset))
        }

        uint256 lengthPayload;
        assembly {
            lengthPayload := mload(add(argumentsBlock, offsetPayload))
        }

        // account for:
        // 4  bytes - functionSig
        // 32 bytes - length encoding for the parameter payload
        // start with length and followed by actual values
        uint256 start = 4 + offsetPayload;
        uint256 end = start + lengthPayload * 32 + 32;

        // are we slicing out of bounds?
        if (data.length < end) {
            revert CalldataOutOfBounds();
        }

        // prefix 32 bytes of offset which indicates the location of length
        return abi.decode(abi.encodePacked(uint256(32),slice(data, start, end)), (address[]));
    }

    /**
     * @dev Extracts and returns two addresses from a specific portion of the `data` byte array.
     * @param data The byte array containing the data.
     * @param index The index of the parameter to extract.
     * @return a The first address extracted from the specified portion of the `data` byte array.
     * @return b The second address extracted from the specified portion of the `data` byte array.
     */
    function pluckSendPayload(
        bytes memory data,
        uint256 index
    ) internal pure returns (address, address) {
        // pre-check: is there a word available for the current parameter at argumentsBlock?
        if (data.length < 4 + index * 32 + 32) {
            revert CalldataOutOfBounds();
        }

        // the start of the parameter block
        // 32 bytes - length encoding of the data bytes array
        // 4  bytes - function sig
        uint256 argumentsBlock;
        assembly {
            argumentsBlock := add(data, 36)
        }

        // the two offsets are relative to argumentsBlock
        uint256 offset = index * 32;
        uint256 offsetPayload;
        assembly {
            offsetPayload := mload(add(argumentsBlock, offset))
        }

        uint256 lengthPayload;
        assembly {
            lengthPayload := mload(add(argumentsBlock, offsetPayload))
        }

        // account for:
        // 4  bytes - functionSig
        // 32 bytes - length encoding for the parameter payload
        // Note that the start has skipped length location
        uint256 start = 4 + offsetPayload + 32;
        uint256 end = start + lengthPayload;

        // are we slicing out of bounds?
        if (data.length < end) {
            revert CalldataOutOfBounds();
        }

        (address a, address b, , ) = abi.decode(slice(data, start, end), (address, address, uint256, uint256));
        return (a, b);
    }

    /**
     * @dev Returns a copy of a portion of the `data` byte array.
     * @param data The byte array to slice.
     * @param start The starting index of the slice (inclusive).
     * @param end The ending index of the slice (exclusive).
     * @return result A new byte array containing the sliced portion.
     */
    function slice(
        bytes memory data,
        uint256 start,
        uint256 end
    ) internal pure returns (bytes memory result) {
        result = new bytes(end - start);
        for (uint256 j = start; j < end; j++) {
            result[j - start] = data[j];
        }
    }

    /**
     * @dev Returns a copy of a portion of the `data` byte array.
     * @param data The byte array to slice.
     * @param start The starting index of the slice (inclusive).
     * @return result A new byte array containing the sliced portion.
     */
    function sliceFrom(
        bytes memory data,
        uint256 start
    ) internal pure returns (bytes memory result) {
        result = new bytes(data.length - start);
        for (uint256 j = start; j < data.length; j++) {
            result[j - start] = data[j];
        }
    }

    /**
     * @dev Returns the unique key for a function of a given `targetAddress`.
     * @param targetAddress The address of the target contract.
     * @param functionSig The function signature of the target function.
     * @return key The unique key representing the target function.
     */
    function keyForFunctions(
        address targetAddress,
        bytes4 functionSig
    ) internal pure returns (bytes32) {
        return bytes32(abi.encodePacked(targetAddress, functionSig));
    }

    /**
     * @dev Returns arrays of bytes32 that concates function signatures (bytes4 = 32 bits)
     * together with granular permissions (per channel id or per beneficiary) (2 bits)
     * It can take maxinum 7 sets (256 / (32 + 2) ~= 7) of function signatures and permissions
     * @notice Signature encoding is right-padded; Index 0 is the left most and grows to the right
     * Permission encoding is left-padded; Index grows from right to the left.
     * Returns a bytes32 and length of sigature and permissions
     */
    function encodeFunctionSigsAndPermissions(
       bytes4[] memory functionSigs,
       GranularPermission[] memory permissions
    ) internal pure returns (bytes32 encoded, uint256 length) {
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
            val |= (bytes32(functionSigs[i]) >> 224) << (224 - 32 * i);
        }
        for (uint256 i = 0; i < len; i++) {
            // shift by two bits
            val |= bytes32(uint256(permissions[i])) << 2 * i;
        }
        return (val, len);
    }

    /**
     * @dev Returns an bytes4 array which decodes from the combined encoding
     * of function signature and permissions. It can take maxinum 7 items.
     * Encoding of function signatures is right-padded, where indexes grow from left to right
     * Encoding of permissions is left-padded, where indexes grow from left to right
     */
    function decodeFunctionSigsAndPermissions(
        bytes32 encoded, 
        uint256 length
    ) internal pure returns (bytes4[] memory functionSigs, GranularPermission[] memory permissions) {
        if (length > 7) {
            revert ArrayTooLong();
        }
        functionSigs = new bytes4[](length);
        permissions = new GranularPermission[](length);
        // decode function signature
        for (uint256 i = 0; i < length; i++) {
            // first right shift (32 - 4 * i - 4) * 8 = (224 - 32 * i) bits
            // then left shift (32 - 4) * 8 = 224 bits
            functionSigs[i] = bytes4((encoded >> (224 - 32 * i)) << 224);
        }
        // decode permissions
        for (uint256 j = 0; j < length; j++) {
            // first left shift 256 - 2 - 2 * j = 254 - 2 * j bits
            // then right shift 256 - 2 = 254 bits
            permissions[j] = GranularPermission(uint8((uint256(encoded) << (254 - 2 * j)) >> 254));
        }
    }
}
