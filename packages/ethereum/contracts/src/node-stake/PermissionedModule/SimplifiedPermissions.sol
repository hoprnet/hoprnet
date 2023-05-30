// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity >=0.7.0 <0.9.0;

import "safe-contracts/common/Enum.sol";

// enum ParameterType {
//     Static,
//     Dynamic,    // bytes, string
//     Dynamic32   // non-nested arrays: address[] bytes32[] uint[] etc
// }

enum HoprChannelsPermission {
    Allowed,
    Blocked
}

enum HoprTokenPermission {
    Blocked,
    Allowed
}

enum SendPermission {
    Blocked,
    Allowed
}

enum Clearance {
    None,
    Function
}

enum TargetType {
    None,
    Token,
    Channels,
    Send
}

struct TargetAddress {
    Clearance clearance;
    TargetType targetType;
}

struct Role {
    mapping(address => bool) members;   // eligible caller. May be able to receive native tokens (e.g. xDAI), if set to allowed
    mapping(address => TargetAddress) targets;  // target addresses that can be called
    mapping(bytes32 => mapping(bytes32 => HoprChannelsPermission)) hoprChannelsCapability; // keyForFunctions (bytes32) => channel Id (keccak256(src, dest)) => HoprChannelsPermission
    mapping(bytes32 => mapping(address => HoprTokenPermission)) hoprTokenCapability; // keyForFunctions (bytes32) => beneficiary address => HoprTokenPermission
    mapping(address => SendPermission) sendCapability; // beneficiary address => SendPermission
}

/**
 * @dev Simplified zodiac-modifier-roles-v1 Permission.sol contract
 * This library supports only one role
 */
library SimplifiedPermissions {
    // HoprChannels method ids (TargetType.Channels)
    bytes4 internal constant FUND_CHANNEL_MULTI_SELECTOR = hex"4341abdd";
    bytes4 internal constant REDEEM_TICKET_SELECTOR = hex"0475568e";
    bytes4 internal constant REDEEM_TICKETS_SELECTOR = hex"c5ad200d";
    bytes4 internal constant INITIATE_CHANNEL_CLOSURE_SELECTOR = hex"88d2f3c9";
    bytes4 internal constant FINALIZE_CHANNEL_CLOSURE_SELECTOR = hex"833aae8d";
    bytes4 internal constant BUMP_CHANNEL_SELECTOR = hex"c4d93afb";
    // HoprTokens method ids (TargetType.Token)
    bytes4 internal constant APPROVE_SELECTOR = hex"095ea7b3";
    bytes4 internal constant SEND_SELECTOR = hex"9bd9bbc6";

    event RevokedTarget(address targetAddress);
    event ScopedTargetChannels(address targetAddress);
    event ScopedTargetToken(address targetAddress);
    event ScopedTargetSend(address targetAddress);

    event ScopedChannelsCapability(
        address targetAddress,
        bytes4 selector,
        bytes32 channelId,
        HoprChannelsPermission permission
    );
    event ScopedTokenCapability(
        address targetAddress,
        bytes4 selector,
        address beneficiary,
        HoprTokenPermission permission
    );
    event ScopedSendCapability(
        address beneficiary,
        SendPermission permission
    );


    /// Parameter Type is not supported
    error ParameterTypeNotSupported();

    /// Sender is not a member of the role
    error NoMembership();

    /// Arrays must be the same length
    error ArraysDifferentLength();

    /// Function signature too short
    error FunctionSignatureTooShort();
    
    // /// Function signature is not allowed
    // error FunctionSignatureNotAllowed();

    /// Role not allowed to delegate call to target address
    error DelegateCallNotAllowed();

    /// Role not allowed to call target address
    error TargetAddressNotAllowed();

    /// Role not allowed to call target when its type is not set
    error TargetTypeNotSet();

    /// Role not allowed to call this function on target address
    error FunctionNotAllowed();

    /// Role not allowed to send to target address
    error SendNotAllowed();

    /// Role not allowed to use bytes for parameter
    error ParameterNotAllowed();

    // /// Role not allowed to use bytes for parameter
    // error ParameterNotOneOfAllowed();

    // /// Role not allowed to use bytes less than value for parameter
    // error ParameterLessThanAllowed();

    // /// Role not allowed to use bytes greater than value for parameter
    // error ParameterGreaterThanAllowed();

    /// only multisend txs with an offset of 32 bytes are allowed
    error UnacceptableMultiSendOffset();

    /// OneOf Comparison must be set via dedicated function
    error UnsuitableOneOfComparison();

    /// Not possible to define gt/lt for Dynamic types
    error UnsuitableRelativeComparison();

    /// CompValue for static types should have a size of exactly 32 bytes
    error UnsuitableStaticCompValueSize();

    /// CompValue for Dynamic32 types should be a multiple of exactly 32 bytes
    error UnsuitableDynamic32CompValueSize();

    /// Exceeds the max number of params supported
    error ScopeMaxParametersExceeded();

    /// OneOf Comparison requires at least two compValues
    error NotEnoughCompValuesForOneOf();

    /// The provided calldata for execution is too short, or an OutOfBounds scoped parameter was configured
    error CalldataOutOfBounds();



    // ======================================================
    // ---------------------- CHECKERS ----------------------
    // ======================================================

    /**
     * @dev checks if the function is allowed to be passed to the avatar
     */
    function check(
        Role storage role,
        address multisend,
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation
    ) public view {
        if (!role.members[msg.sender]) {
            revert NoMembership();
        }
        if (multisend == to) {
            checkMultisendTransaction(role, data);
        } else {
            checkTransaction(role, to, value, data, operation);
        }
    }

    /*
     * @dev Splits a multisend data blob into transactions and forwards them to be checked.
     * @param data the packed transaction data (created by utils function buildMultiSendSafeTx).
     * @param role Role to check for.
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
     * @dev Main transaction to check the permission of transaction execution of a module
     * Only transctions to target
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
        if (target.clearance != Clearance.Function) {
            revert TargetAddressNotAllowed();
        }

        // delegate call is not allowed; value can only be sent with `Send`
        checkExecutionOptions(value, operation, target.targetType);

        if (target.targetType == TargetType.Token) {
            // check with HoprToken contract
            checkHoprTokenParameters(role, targetAddress, data);
            return;
        } else if (target.targetType == TargetType.Channels) {
            // check with HoprChannels contract
            checkHoprChannelsParameters(role, targetAddress, data);
            return;
        } else if (target.targetType == TargetType.Send) {
            if (role.sendCapability[targetAddress] != SendPermission.Allowed) {
                // not allowed to call the capability
                revert ParameterNotAllowed();
            }
            return;
        } else {

        }
    }

    /**
     * @dev Check if the transaction can send along native tokens
     * Check if the DelegatedCall is allowed
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
            targetType != TargetType.Send
        ) {
            revert SendNotAllowed();
        }

        if (targetType == TargetType.None) {
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
        bytes memory data
    ) internal view {
        bytes4 functionSig = bytes4(data);
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);

        if (functionSig == REDEEM_TICKETS_SELECTOR) {
            // only redeemTickets function has Dynamic32 type
            address[] memory srcs = pluckDynamicAddresses(data, 0);
            address[] memory dests = pluckDynamicAddresses(data, 1);

            if (srcs.length != dests.length) {
                revert ArraysDifferentLength();
            }

            for (uint256 i = 0; i < srcs.length; i++) {
                // check if functions on this channel can be called.
                compareHoprChannelsPermission(role, capabilityKey, srcs[i], dests[i]);
            }
        } else {
            // source and channel destination addreses are at the first and second places respectively
            (address src, address dest) = pluckTwoStaticAddresses(data);
            // check if functions on this channel can be called.
            compareHoprChannelsPermission(role, capabilityKey, src, dest);
        }
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
        bytes memory data
    ) internal view {
        bytes4 functionSig = bytes4(data);
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);

        // check if the first parameter is allowed
        address beneficiary = pluckOneStaticAddress(data, 0);  
        if (role.hoprTokenCapability[capabilityKey][beneficiary] != HoprTokenPermission.Allowed) {
            // not allowed to call the capability
            revert ParameterNotAllowed();
        }

        // if calling `send` method, it is equivalent to calling FUND_CHANNEL_MULTI_SELECTOR
        if (functionSig == SEND_SELECTOR) {
            bytes32 sendCapabilityKey = keyForFunctions(targetAddress, FUND_CHANNEL_MULTI_SELECTOR);
            // source and channel destination addreses are at the first and second places respectively
            (address src, address dest) = pluckSendPayload(data, 2);
            // check if functions on this channel can be called.
            compareHoprChannelsPermission(role, sendCapabilityKey, src, dest);
        }
    }

    // ======================================================
    // ----------------------- SETTERS ----------------------
    // ======================================================
    
    /*
     * @dev Forbid role members to call all the functions of any type (call or delegatecall)
     * of a given target.
     */
    function revokeTarget(
        Role storage role,
        address targetAddress
    ) external {
        role.targets[targetAddress] = TargetAddress(
            Clearance.None,
            TargetType.None
        );
        emit RevokedTarget(targetAddress);
    }

    /**
     * @dev Allow target address as HoprToken
     */
    function scopeTargeToken(
        Role storage role,
        address targetAddress
    ) external {
        role.targets[targetAddress] = TargetAddress(
            Clearance.Function,
            TargetType.Token
        );
        emit ScopedTargetToken(targetAddress);
    }

    /**
     * @dev Allow target address as HoprChannel
     */
    function scopeTargetChannels(
        Role storage role,
        address targetAddress
    ) external {
        role.targets[targetAddress] = TargetAddress(
            Clearance.Function,
            TargetType.Channels
        );
        emit ScopedTargetChannels(targetAddress);
    }

    /**
     * @dev Allow target address as beneficiary of Send
     */
    function scopeTargetSend(
        Role storage role,
        address targetAddress
    ) external {
        role.targets[targetAddress] = TargetAddress(
            Clearance.Function,
            TargetType.Send
        );
        emit ScopedTargetSend(targetAddress);
    }

    /**
     * @dev Set the permission for a specific function on a scoped HoprChannels target.
     */
    function scopeChannelCapability(
        Role storage role,
        address targetAddress,
        bytes4 functionSig,
        bytes32 channelId,
        HoprChannelsPermission permission
    ) external {
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);
        role.hoprChannelsCapability[capabilityKey][channelId] = permission;

        emit ScopedChannelsCapability(
            targetAddress,
            functionSig,
            channelId,
            permission
        );
    }

    /**
     * @dev Set the permission for a specific function on a scoped HoprToken target.
     */
    function scopeTokenCapability(
        Role storage role,
        address targetAddress,
        bytes4 functionSig,
        address beneficiary,
        HoprTokenPermission permission
    ) external {
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);
        role.hoprTokenCapability[capabilityKey][beneficiary] = permission;

        emit ScopedTokenCapability(
            targetAddress,
            functionSig,
            beneficiary,
            permission
        );
    }

    /**
     * @dev Set the permission for a specific function on a scoped HoprToken target.
     */
    function scopeTokenCapability(
        Role storage role,
        address beneficiary,
        SendPermission permission
    ) external {
        role.sendCapability[beneficiary] = permission;

        emit ScopedSendCapability(
            beneficiary,
            permission
        );
    }


    // TODO:
    /*
     *
     * HELPERS
     *
    */
    /** FIXME:
   * @param role reference to role storage
   * @param capabilityKey the id key of function on the target HoprChannels address
   * @param source the address of source
   * @param destination the address of destination
   */
    function compareHoprChannelsPermission(Role storage role, bytes32 capabilityKey, address source, address destination) internal view returns (bool) {
        // get channelId
        bytes32 channelId = keccak256(abi.encodePacked(source, destination));
        // check if it's allowed to call the channel
        if (role.hoprChannelsCapability[capabilityKey][channelId] != HoprChannelsPermission.Allowed) {
            // not allowed to call the capability
            revert ParameterNotAllowed();
        }
    }
//   /** FIXME:
//    * @param source the address of source
//    * @param destination the address of destination
//    * @return the channel id
//    */
//   function _getChannelId(address source, address destination) internal pure returns (bytes32) {
//     return keccak256(abi.encodePacked(source, destination));
//   }

    /** FIXME:
     * @dev Pluck a bytes32 at index position into address
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

    /** FIXME:
     * @dev Pluck first two bytes32 into two addresses
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
    /** FIXME:
     * @dev pluck array of addresses from data 
     */
    function pluckDynamicAddresses(
        bytes memory data,
        uint256 index
    ) internal pure returns (address[] memory decodedAddresses) {
        // pre-check: is there a word available for the current parameter at argumentsBlock?
        if (data.length < 4 + index * 32 + 32) {
            revert CalldataOutOfBounds();
        }

        /*
         * Encoded calldata:
         * 4  bytes -> function selector
         * 32 bytes -> sequence, one chunk per parameter
         *
         * There is one (byte32) chunk per parameter. Depending on type it contains:
         * Static    -> value encoded inline (not plucked by this function)
         * Dynamic   -> a byte offset to encoded data payload
         * Dynamic32 -> a byte offset to encoded data payload
         * Note: Fixed Sized Arrays (e.g., bool[2]), are encoded inline
         * Note: Nested types also do not follow the above described rules, and are unsupported
         * Note: The offset to payload does not include 4 bytes for functionSig
         *
         *
         * At encoded payload, the first 32 bytes are the length encoding of the parameter payload. Depending on ParameterType:
         * Dynamic   -> length in bytes
         * Dynamic32 -> length in bytes32
         * Note: Dynamic types are: bytes, string
         * Note: Dynamic32 types are non-nested arrays: address[] bytes32[] uint[] etc
         */

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

        // prefix 32 bytes of offset
        return abi.decode(abi.encodePacked(uint256(32),slice(data, start, end)), (address[]));
    }

    // TODO:
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
        uint256 start = 4 + offsetPayload + 32;
        uint256 end = start + lengthPayload;

        // are we slicing out of bounds?
        if (data.length < end) {
            revert CalldataOutOfBounds();
        }

        (address a, address b, , ) = abi.decode(slice(data, start, end), (address, address, uint256, uint256));
        return (a, b);
    }

    // // TODO:
    // function pluckDynamicValue(
    //     bytes memory data,
    //     ParameterType paramType,
    //     uint256 index
    // ) internal pure returns (bytes32) {
    //     assert(paramType != ParameterType.Static);
    //     // pre-check: is there a word available for the current parameter at argumentsBlock?
    //     if (data.length < 4 + index * 32 + 32) {
    //         revert CalldataOutOfBounds();
    //     }

    //     /*
    //      * Encoded calldata:
    //      * 4  bytes -> function selector
    //      * 32 bytes -> sequence, one chunk per parameter
    //      *
    //      * There is one (byte32) chunk per parameter. Depending on type it contains:
    //      * Static    -> value encoded inline (not plucked by this function)
    //      * Dynamic   -> a byte offset to encoded data payload
    //      * Dynamic32 -> a byte offset to encoded data payload
    //      * Note: Fixed Sized Arrays (e.g., bool[2]), are encoded inline
    //      * Note: Nested types also do not follow the above described rules, and are unsupported
    //      * Note: The offset to payload does not include 4 bytes for functionSig
    //      *
    //      *
    //      * At encoded payload, the first 32 bytes are the length encoding of the parameter payload. Depending on ParameterType:
    //      * Dynamic   -> length in bytes
    //      * Dynamic32 -> length in bytes32
    //      * Note: Dynamic types are: bytes, string
    //      * Note: Dynamic32 types are non-nested arrays: address[] bytes32[] uint[] etc
    //      */

    //     // the start of the parameter block
    //     // 32 bytes - length encoding of the data bytes array
    //     // 4  bytes - function sig
    //     uint256 argumentsBlock;
    //     assembly {
    //         argumentsBlock := add(data, 36)
    //     }

    //     // the two offsets are relative to argumentsBlock
    //     uint256 offset = index * 32;
    //     uint256 offsetPayload;
    //     assembly {
    //         offsetPayload := mload(add(argumentsBlock, offset))
    //     }

    //     uint256 lengthPayload;
    //     assembly {
    //         lengthPayload := mload(add(argumentsBlock, offsetPayload))
    //     }

    //     // account for:
    //     // 4  bytes - functionSig
    //     // 32 bytes - length encoding for the parameter payload
    //     uint256 start = 4 + offsetPayload + 32;
    //     uint256 end = start +
    //         (
    //             paramType == ParameterType.Dynamic32
    //                 ? lengthPayload * 32
    //                 : lengthPayload
    //         );

    //     // are we slicing out of bounds?
    //     if (data.length < end) {
    //         revert CalldataOutOfBounds();
    //     }

    //     return keccak256(slice(data, start, end));
    // }

    // TODO:
    function pluckStaticValue(
        bytes memory data,
        uint256 index
    ) internal pure returns (bytes32) {
        // pre-check: is there a word available for the current parameter at argumentsBlock?
        if (data.length < 4 + index * 32 + 32) {
            revert CalldataOutOfBounds();
        }

        uint256 offset = 4 + index * 32;
        bytes32 value;
        assembly {
            // add 32 - jump over the length encoding of the data bytes array
            value := mload(add(32, add(data, offset)))
        }
        return value;
    }

    // TODO:
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
     * @dev Get the unique key for a function of a given target
     */
    function keyForFunctions(
        address targetAddress,
        bytes4 functionSig
    ) public pure returns (bytes32) {
        return bytes32(abi.encodePacked(targetAddress, functionSig));
    }
}
