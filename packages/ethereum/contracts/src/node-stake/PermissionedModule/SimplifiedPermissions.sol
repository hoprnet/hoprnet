// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity >=0.7.0 <0.9.0;

import "safe-contracts/common/Enum.sol";

enum ParameterType {
    Static,
    Dynamic,    // bytes, string
    Dynamic32   // non-nested arrays: address[] bytes32[] uint[] etc
}

enum ExecutionOptions {
    None,
    Send
}

enum Clearance {
    None,
    Function
}

enum HoprChannelPermission {
    Allowed,
    Blocked
}

enum TargetType {
    Channel,
    Token,
    Send
}

struct TargetAddress {
    Clearance clearance;
    ExecutionOptions options;
}

struct Role {
    mapping(address => bool) members;   // eligible caller. May be able to receive native tokens (e.g. xDAI)
    mapping(address => address) targetChannels;
    mapping(address => address) targetTokens;
    mapping(address => address) targetSend;
    mapping(bytes32 => uint256) functions;
    mapping(bytes32 => mapping(bytes32 => HoprChannelPermission)) hoprChannelsCapability; // keyForFunctions (bytes32) => channel Id (keccak256(src, dest)) => 
}

/**
 * @dev Simplified zodiac-modifier-roles-v1 Permission.sol contract
 * This library supports only one role
 */
library SimplifiedPermissions {
    // HoprChannels method ids
    bytes4 internal constant FUND_CHANNEL_MULTI_SELECTOR = hex"4341abdd";
    bytes4 internal constant REDEEM_TICKET_SELECTOR = hex"0475568e";
    bytes4 internal constant REDEEM_TICKETS_SELECTOR = hex"c5ad200d";
    bytes4 internal constant INITIATE_CHANNEL_CLOSURE_SELECTOR = hex"88d2f3c9";
    bytes4 internal constant FINALIZE_CHANNEL_CLOSURE_SELECTOR = hex"833aae8d";
    bytes4 internal constant BUMP_CHANNEL_SELECTOR = hex"c4d93afb";
    // HoprTokens method ids
    bytes4 internal constant APPROVE_SELECTOR = hex"095ea7b3";
    bytes4 internal constant SEND_SELECTOR = hex"9bd9bbc6";

    uint256 internal constant SCOPE_MAX_PARAMS = 48;
    event RevokeTarget(address targetAddress);
    event ScopeTarget(address targetAddress);

    event ScopeAllowFunction(
        address targetAddress,
        bytes4 selector,
        ExecutionOptions options,
        uint256 resultingScopeConfig
    );
    event ScopeRevokeFunction(
        address targetAddress,
        bytes4 selector,
        uint256 resultingScopeConfig
    );
    event ScopeFunction(
        address targetAddress,
        bytes4 functionSig,
        bool[] isParamScoped,
        ParameterType[] paramType,
        Comparison[] paramComp,
        bytes[] compValue,
        ExecutionOptions options,
        uint256 resultingScopeConfig
    );
    event ScopeFunctionExecutionOptions(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        ExecutionOptions options,
        uint256 resultingScopeConfig
    );
    event ScopeParameter(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        uint256 index,
        ParameterType paramType,
        Comparison paramComp,
        bytes compValue,
        uint256 resultingScopeConfig
    );
    event ScopeParameterAsOneOf(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        uint256 index,
        ParameterType paramType,
        bytes[] compValues,
        uint256 resultingScopeConfig
    );
    event UnscopeParameter(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        uint256 index,
        uint256 resultingScopeConfig
    );

    /// Parameter Type is not supported
    error ParameterTypeNotSupported();

    /// Sender is not a member of the role
    error NoMembership();

    /// Arrays must be the same length
    error ArraysDifferentLength();

    /// Function signature too short
    error FunctionSignatureTooShort();

    /// Role not allowed to delegate call to target address
    error DelegateCallNotAllowed();

    /// Role not allowed to call target address
    error TargetAddressNotAllowed();

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

    /** TODO: FIXME: 
     * @dev Main transaction to check the permission of transaction execution of a module
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
        if (target.clearance == Clearance.None) {
            revert TargetAddressNotAllowed();
        }

        // TODO: remove me, 
        // if (target.clearance == Clearance.Target) {
        //     checkExecutionOptions(value, operation, target.options);
        //     return;
        // }

        // when certain functions of the target address are allowed, 
        // check if the current transaction is calling one of them
        if (target.clearance == Clearance.Function) {
            uint256 scopeConfig = role.functions[
                keyForFunctions(targetAddress, bytes4(data))
            ];

            if (scopeConfig == 0) {
                revert FunctionNotAllowed();
            }

            // TODO: Replace this unpackFunction so that an addtional bool field for HoprChannels management gets unwrapped
            (ExecutionOptions options, bool isWildcarded, ) = unpackFunction(
                scopeConfig
            );


            checkExecutionOptions(value, operation, options);

            if (isWildcarded == false) {
                // // FIXME: add conditional route for HoprChannels management
                // if (isHoprChannelsInteraction == true) {
                //     checkHoprChannelsParameters(role, scopeConfig, targetAddress, data);
                // } else {
                    checkParameters(role, scopeConfig, targetAddress, data);
                // }
            }
            return;
        }

        assert(false);
    }

    /**
     * @dev Check if the transaction can send along native tokens
     * Check if the DelegatedCall is allowed
     */
    function checkExecutionOptions(
        uint256 value,
        Enum.Operation operation,
        ExecutionOptions options
    ) internal pure {
        // isSend && !canSend
        if (
            value > 0 &&
            options != ExecutionOptions.Send &&
            options != ExecutionOptions.Both
        ) {
            revert SendNotAllowed();
        }

        // isDelegateCall && !canDelegateCall
        if (
            operation == Enum.Operation.DelegateCall &&
            options != ExecutionOptions.DelegateCall &&
            options != ExecutionOptions.Both
        ) {
            revert DelegateCallNotAllowed();
        }
    }



    /* FIXME: `paramType` may be removed?
     * @dev Check parameters for HoprChannels capability
     * @param role reference to role storage
     * @param scopeConfig reference to role storage
     * @param targetAddress Address to check.
     * @param data the transaction data to check
     */
    function checkHoprChannelsParameters(
        Role storage role,
        ParameterType paramType,
        address targetAddress,
        bytes memory data
    ) internal view {
        bytes4 functionSig = bytes4(data);
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);

        if (paramType == ParameterType.Dynamic) {
            // Not suppose to have Dynamic type
            revert ParameterTypeNotSupported();
        }
        
        if (paramType == ParameterType.Static) {
            // Channel source and channel destination addreses are at the first and second places respectively
            (address src, address dest) = pluckTwoStaticAddresses(data);
            // check if functions on this channel can be called.
            compareHoprChannelsPermission(role, capabilityKey, src, dest);
        } else {
            // Dynamic32 type
            address[] memory srcs = pluckDynamicAddresses(data, 0);
            address[] memory dests = pluckDynamicAddresses(data, 1);

            if (srcs.length != dests.length) {
                revert ArraysDifferentLength();
            }

            for (uint256 i = 0; i < srcs.length; i++) {
                // check if functions on this channel can be called.
                compareHoprChannelsPermission(role, capabilityKey, srcs[i], dests[i]);
            }
        }
    }

// TODO:
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
        uint256 scopeConfig,
        address targetAddress,
        bytes memory data
    ) internal view {
        bytes4 functionSig = bytes4(data);
        bytes32 capabilityKey = keyForFunctions(targetAddress, functionSig);

        // check 


        // (, , uint256 length) = unpackFunction(scopeConfig);

        // for (uint256 i = 0; i < length; i++) {
        //     (
        //         bool isScoped,
        //         ParameterType paramType,
        //         Comparison paramComp
        //     ) = unpackParameter(scopeConfig, i);

        //     if (!isScoped) {
        //         continue;
        //     }

        //     bytes32 value;
        //     if (paramType != ParameterType.Static) {
        //         value = pluckDynamicValue(data, paramType, i);
        //     } else {
        //         value = pluckStaticValue(data, i);
        //     }

        //     bytes32 key = keyForCompValues(targetAddress, functionSig, i);
        //     if (paramComp != Comparison.OneOf) {
        //         compare(paramComp, role.compValues[key], value);
        //     } else {
        //         compareOneOf(role.compValuesOneOf[key], value);
        //     }
        // }
    }
// // TODO:
//     function compare(
//         Comparison paramComp,
//         bytes32 compValue,
//         bytes32 value
//     ) internal pure {
//         if (paramComp == Comparison.EqualTo && value != compValue) {
//             revert ParameterNotAllowed();
//         } else if (paramComp == Comparison.GreaterThan && value <= compValue) {
//             revert ParameterLessThanAllowed();
//         } else if (paramComp == Comparison.LessThan && value >= compValue) {
//             revert ParameterGreaterThanAllowed();
//         }
//     }
// // TODO:
//     function compareOneOf(
//         bytes32[] storage compValue,
//         bytes32 value
//     ) internal view {
//         for (uint256 i = 0; i < compValue.length; i++) {
//             if (value == compValue[i]) return;
//         }
//         revert ParameterNotOneOfAllowed();
//     }

    /*
     *
     * SETTERS
     *
     */
    
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
            ExecutionOptions.None
        );
        emit RevokeTarget(targetAddress);
    }

    /**
     * @dev Allow certain functions of the target address to be called.
     */
    function scopeTarget(
        Role storage role,
        address targetAddress
    ) external {
        role.targets[targetAddress] = TargetAddress(
            Clearance.Function,
            ExecutionOptions.None
        );
        emit ScopeTarget(targetAddress);
    }

    /**
     * @dev Allows a specific function signature on a scoped target.
     * This is the default config for all the functions
     */
    function scopeAllowFunction(
        Role storage role,
        address targetAddress,
        bytes4 functionSig,
        ExecutionOptions options
    ) external {
        /*
         * packLeft(
         *    0           -> start from a fresh scopeConfig
         *    options     -> externally provided options
         *    true        -> mark the function as wildcarded
         *    0           -> length
         * )
         */
        uint256 scopeConfig = packLeft(0, options, true, 0);
        role.functions[
            keyForFunctions(targetAddress, functionSig)
        ] = scopeConfig;
        emit ScopeAllowFunction(
            targetAddress,
            functionSig,
            options,
            scopeConfig
        );
    }

    /**
     * @dev Disallows a specific function signature on a scoped target.
     */
    function scopeRevokeFunction(
        Role storage role,
        address targetAddress,
        bytes4 functionSig
    ) external {
        role.functions[keyForFunctions(targetAddress, functionSig)] = 0;
        emit ScopeRevokeFunction(targetAddress, functionSig, 0);
    }

    // TODO:
    function scopeFunction(
        Role storage role,
        address targetAddress,
        bytes4 functionSig,
        bool[] memory isScoped,
        ParameterType[] memory paramType,
        Comparison[] memory paramComp,
        bytes[] calldata compValue,
        ExecutionOptions options
    ) external {
        uint256 length = isScoped.length;

        if (
            length != paramType.length ||
            length != paramComp.length ||
            length != compValue.length
        ) {
            revert ArraysDifferentLength();
        }

        if (length > SCOPE_MAX_PARAMS) {
            revert ScopeMaxParametersExceeded();
        }

        for (uint256 i = 0; i < length; i++) {
            if (isScoped[i]) {
                enforceComp(paramType[i], paramComp[i]);
                enforceCompValue(paramType[i], compValue[i]);
            }
        }

        /*
         * packLeft(
         *    0           -> start from a fresh scopeConfig
         *    options     -> externally provided options
         *    false       -> mark the function as not wildcarded
         *    0           -> length
         * )
         */
        uint256 scopeConfig = packLeft(0, options, false, length);
        for (uint256 i = 0; i < length; i++) {
            scopeConfig = packRight(
                scopeConfig,
                i,
                isScoped[i],
                paramType[i],
                paramComp[i]
            );
        }

        //set scopeConfig
        role.functions[
            keyForFunctions(targetAddress, functionSig)
        ] = scopeConfig;

        //set compValues
        for (uint256 i = 0; i < length; i++) {
            role.compValues[
                keyForCompValues(targetAddress, functionSig, i)
            ] = compressCompValue(paramType[i], compValue[i]);
        }
        emit ScopeFunction(
            targetAddress,
            functionSig,
            isScoped,
            paramType,
            paramComp,
            compValue,
            options,
            scopeConfig
        );
    }

    // TODO:
    function scopeFunctionExecutionOptions(
        Role storage role,
        uint16 roleId,
        address targetAddress,
        bytes4 functionSig,
        ExecutionOptions options
    ) external {
        bytes32 key = keyForFunctions(targetAddress, functionSig);

        //set scopeConfig
        uint256 scopeConfig = packOptions(role.functions[key], options);

        role.functions[
            keyForFunctions(targetAddress, functionSig)
        ] = scopeConfig;

        emit ScopeFunctionExecutionOptions(
            roleId,
            targetAddress,
            functionSig,
            options,
            scopeConfig
        );
    }

    // TODO:
    function scopeParameter(
        Role storage role,
        uint16 roleId,
        address targetAddress,
        bytes4 functionSig,
        uint256 index,
        ParameterType paramType,
        Comparison paramComp,
        bytes calldata compValue
    ) external {
        if (index >= SCOPE_MAX_PARAMS) {
            revert ScopeMaxParametersExceeded();
        }

        enforceComp(paramType, paramComp);
        enforceCompValue(paramType, compValue);

        // set scopeConfig
        bytes32 key = keyForFunctions(targetAddress, functionSig);
        uint256 scopeConfig = packParameter(
            role.functions[key],
            index,
            true, // isScoped
            paramType,
            paramComp
        );
        role.functions[key] = scopeConfig;

        // set compValue
        role.compValues[
            keyForCompValues(targetAddress, functionSig, index)
        ] = compressCompValue(paramType, compValue);

        emit ScopeParameter(
            roleId,
            targetAddress,
            functionSig,
            index,
            paramType,
            paramComp,
            compValue,
            scopeConfig
        );
    }

    // TODO:
    function scopeParameterAsOneOf(
        Role storage role,
        uint16 roleId,
        address targetAddress,
        bytes4 functionSig,
        uint256 index,
        ParameterType paramType,
        bytes[] calldata compValues
    ) external {
        if (index >= SCOPE_MAX_PARAMS) {
            revert ScopeMaxParametersExceeded();
        }

        if (compValues.length < 2) {
            revert NotEnoughCompValuesForOneOf();
        }

        for (uint256 i = 0; i < compValues.length; i++) {
            enforceCompValue(paramType, compValues[i]);
        }

        // set scopeConfig
        bytes32 key = keyForFunctions(targetAddress, functionSig);
        uint256 scopeConfig = packParameter(
            role.functions[key],
            index,
            true, // isScoped
            paramType,
            Comparison.OneOf
        );
        role.functions[key] = scopeConfig;

        // set compValue
        key = keyForCompValues(targetAddress, functionSig, index);
        role.compValuesOneOf[key] = new bytes32[](compValues.length);
        for (uint256 i = 0; i < compValues.length; i++) {
            role.compValuesOneOf[key][i] = compressCompValue(
                paramType,
                compValues[i]
            );
        }

        emit ScopeParameterAsOneOf(
            roleId,
            targetAddress,
            functionSig,
            index,
            paramType,
            compValues,
            scopeConfig
        );
    }

    // TODO:
    function unscopeParameter(
        Role storage role,
        uint16 roleId,
        address targetAddress,
        bytes4 functionSig,
        uint256 index
    ) external {
        if (index >= SCOPE_MAX_PARAMS) {
            revert ScopeMaxParametersExceeded();
        }

        // set scopeConfig
        bytes32 key = keyForFunctions(targetAddress, functionSig);
        uint256 scopeConfig = packParameter(
            role.functions[key],
            index,
            false, // isScoped
            ParameterType(0),
            Comparison(0)
        );
        role.functions[key] = scopeConfig;

        emit UnscopeParameter(
            roleId,
            targetAddress,
            functionSig,
            index,
            scopeConfig
        );
    }

    // TODO:
    function enforceComp(
        ParameterType paramType,
        Comparison paramComp
    ) internal pure {
        if (paramComp == Comparison.OneOf) {
            revert UnsuitableOneOfComparison();
        }

        if (
            (paramType != ParameterType.Static) &&
            (paramComp != Comparison.EqualTo)
        ) {
            revert UnsuitableRelativeComparison();
        }
    }

    // TODO:
    function enforceCompValue(
        ParameterType paramType,
        bytes calldata compValue
    ) internal pure {
        if (paramType == ParameterType.Static && compValue.length != 32) {
            revert UnsuitableStaticCompValueSize();
        }

        if (
            paramType == ParameterType.Dynamic32 && compValue.length % 32 != 0
        ) {
            revert UnsuitableDynamic32CompValueSize();
        }
    }

  /** FIXME:
   * @param role reference to role storage
   * @param capabilityKey the id key of function on the target HoprChannels address
   * @param source the address of source
   * @param destination the address of destination
   */
    function compareHoprChannelsPermission(Role storage role, bytes32 capabilityKey, address source, address destination) internal view returns (bool) {
        // get chainId
        bytes32 chainId = keccak256(abi.encodePacked(source, destination));
        // check if it's allowed to call the channel
        if (role.hoprChannelsCapability[capabilityKey][chainId] != HoprChannelPermission.Allowed) {
            // not allowed to call the capability
            revert ParameterNotAllowed();
        }
    }

    // TODO:
    /*
     *
     * HELPERS
     *
     */
//   /** FIXME:
//    * @param source the address of source
//    * @param destination the address of destination
//    * @return the channel id
//    */
//   function _getChannelId(address source, address destination) internal pure returns (bytes32) {
//     return keccak256(abi.encodePacked(source, destination));
//   }

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
    function pluckDynamicValue(
        bytes memory data,
        ParameterType paramType,
        uint256 index
    ) internal pure returns (bytes32) {
        assert(paramType != ParameterType.Static);
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
        uint256 start = 4 + offsetPayload + 32;
        uint256 end = start +
            (
                paramType == ParameterType.Dynamic32
                    ? lengthPayload * 32
                    : lengthPayload
            );

        // are we slicing out of bounds?
        if (data.length < end) {
            revert CalldataOutOfBounds();
        }

        return keccak256(slice(data, start, end));
    }

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

    // TODO:
    /*
     * pack/unpack are bit helpers for scopeConfig
     */
    function packParameter(
        uint256 scopeConfig,
        uint256 index,
        bool isScoped,
        ParameterType paramType,
        Comparison paramComp
    ) internal pure returns (uint256) {
        (ExecutionOptions options, , uint256 prevLength) = unpackFunction(
            scopeConfig
        );

        uint256 nextLength = index + 1 > prevLength ? index + 1 : prevLength;

        return
            packLeft(
                packRight(scopeConfig, index, isScoped, paramType, paramComp),
                options,
                false, // isWildcarded=false
                nextLength
            );
    }

    // TODO:
    function packOptions(
        uint256 scopeConfig,
        ExecutionOptions options
    ) internal pure returns (uint256) {
        uint256 optionsMask = 3 << 254;

        scopeConfig &= ~optionsMask;
        scopeConfig |= uint256(options) << 254;

        return scopeConfig;
    }

    /**
     * @dev Update the left 16 bits of `scopeConfig` and return the new `scopeConfig`
     */
    function packLeft(
        uint256 scopeConfig,
        ExecutionOptions options,
        bool isWildcarded,
        uint256 length
    ) internal pure returns (uint256) {
        // LEFT SIDE
        // 2   bits -> options
        // 1   bits -> isWildcarded
        // 5   bits -> unused
        // 8   bits -> length
        // RIGHT SIDE
        // 48  bits -> isScoped
        // 96  bits -> paramType (2 bits per entry 48*2)
        // 96  bits -> paramComp (2 bits per entry 48*2)

        // Wipe the LEFT SIDE clean. Start from there
        scopeConfig = (scopeConfig << 16) >> 16;

        // set options -> 256 - 2 = 254
        scopeConfig |= uint256(options) << 254;

        // set isWildcarded -> 256 - 2 - 1 = 253
        if (isWildcarded) {
            scopeConfig |= 1 << 253;
        }

        // set Length -> 48 + 96 + 96 = 240
        scopeConfig |= length << 240;

        return scopeConfig;
    }

    /**
     * @dev Update the right 240 bits of `scopeConfig` and return the new `scopeConfig`
     */
    function packRight(
        uint256 scopeConfig,
        uint256 index,
        bool isScoped,
        ParameterType paramType,
        Comparison paramComp
    ) internal pure returns (uint256) {
        // LEFT SIDE
        // 2   bits -> options
        // 1   bits -> isWildcarded
        // 5   bits -> unused
        // 8   bits -> length
        // RIGHT SIDE
        // 48  bits -> isScoped
        // 96  bits -> paramType (2 bits per entry 48*2)
        // 96  bits -> paramComp (2 bits per entry 48*2)
        uint256 isScopedMask = 1 << (index + 96 + 96);
        uint256 paramTypeMask = 3 << (index * 2 + 96);
        uint256 paramCompMask = 3 << (index * 2);

        if (isScoped) {
            scopeConfig |= isScopedMask;
        } else {
            scopeConfig &= ~isScopedMask;
        }

        scopeConfig &= ~paramTypeMask;
        scopeConfig |= uint256(paramType) << (index * 2 + 96);

        scopeConfig &= ~paramCompMask;
        scopeConfig |= uint256(paramComp) << (index * 2);

        return scopeConfig;
    }

    /**
     * @dev Read the left 16 bits of `scopeConfig` and decode to types
     */
    function unpackFunction(
        uint256 scopeConfig
    )
        internal
        pure
        returns (ExecutionOptions options, bool isWildcarded, uint256 length)
    {
        uint256 isWildcardedMask = 1 << 253;

        options = ExecutionOptions(scopeConfig >> 254);
        isWildcarded = scopeConfig & isWildcardedMask != 0;
        length = (scopeConfig << 8) >> 248;
    }

    /**
     * @dev Read the right 240 bits of `scopeConfig` and decode to types
     */
    function unpackParameter(
        uint256 scopeConfig,
        uint256 index
    )
        internal
        pure
        returns (bool isScoped, ParameterType paramType, Comparison paramComp)
    {
        uint256 isScopedMask = 1 << (index + 96 + 96);
        uint256 paramTypeMask = 3 << (index * 2 + 96);
        uint256 paramCompMask = 3 << (index * 2);

        isScoped = (scopeConfig & isScopedMask) != 0;
        paramType = ParameterType(
            (scopeConfig & paramTypeMask) >> (index * 2 + 96)
        );
        paramComp = Comparison((scopeConfig & paramCompMask) >> (index * 2));
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
// TODO:
    function keyForCompValues(
        address targetAddress,
        bytes4 functionSig,
        uint256 index
    ) public pure returns (bytes32) {
        return
            bytes32(abi.encodePacked(targetAddress, functionSig, uint8(index)));
    }
// TODO:
    function compressCompValue(
        ParameterType paramType,
        bytes calldata compValue
    ) internal pure returns (bytes32) {
        return
            paramType == ParameterType.Static
                ? bytes32(compValue)
                : keccak256(compValue);
    }
}
