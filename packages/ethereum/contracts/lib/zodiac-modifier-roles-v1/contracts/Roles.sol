// SPDX-License-Identifier: LGPL-3.0-only
pragma solidity >=0.7.0 <0.9.0;

import "zodiac/core/Modifier.sol";
import "./Permissions.sol";

contract Roles is Modifier {
    address public multisend;

    mapping(address => uint16) public defaultRoles;
    mapping(uint16 => Role) internal roles;

    event AssignRoles(address module, uint16[] roles, bool[] memberOf);
    event SetMultisendAddress(address multisendAddress);
    event RolesModSetup(
        address indexed initiator,
        address indexed owner,
        address indexed avatar,
        address target
    );
    event SetDefaultRole(address module, uint16 defaultRole);

    /// `setUpModules` has already been called
    error SetUpModulesAlreadyCalled();

    /// Arrays must be the same length
    error ArraysDifferentLength();

    /// Sender is not a member of the role
    error NoMembership();

    /// Sender is allowed to make this call, but the internal transaction failed
    error ModuleTransactionFailed();

    /// @param _owner Address of the owner
    /// @param _avatar Address of the avatar (e.g. a Gnosis Safe)
    /// @param _target Address of the contract that will call exec function
    constructor(address _owner, address _avatar, address _target) {
        bytes memory initParams = abi.encode(_owner, _avatar, _target);
        setUp(initParams);
    }

    function setUp(bytes memory initParams) public override {
        (address _owner, address _avatar, address _target) = abi.decode(
            initParams,
            (address, address, address)
        );
        __Ownable_init();

        avatar = _avatar;
        target = _target;

        transferOwnership(_owner);
        setupModules();

        emit RolesModSetup(msg.sender, _owner, _avatar, _target);
    }

    function setupModules() internal {
        if (modules[SENTINEL_MODULES] != address(0)) {
            revert SetUpModulesAlreadyCalled();
        }
        modules[SENTINEL_MODULES] = SENTINEL_MODULES;
    }

    /// @dev Set the address of the expected multisend library
    /// @notice Only callable by owner.
    /// @param _multisend address of the multisend library contract
    function setMultisend(address _multisend) external onlyOwner {
        multisend = _multisend;
        emit SetMultisendAddress(multisend);
    }

    /// @dev Allows all calls made to an address.
    /// @notice Only callable by owner.
    /// @param role Role to set for
    /// @param targetAddress Address to be allowed
    /// @param options defines whether or not delegate calls and/or eth can be sent to the target address.
    function allowTarget(
        uint16 role,
        address targetAddress,
        ExecutionOptions options
    ) external onlyOwner {
        Permissions.allowTarget(roles[role], role, targetAddress, options);
    }

    /// @dev Disallows all calls made to an address.
    /// @notice Only callable by owner.
    /// @param role Role to set for
    /// @param targetAddress Address to be disallowed
    function revokeTarget(
        uint16 role,
        address targetAddress
    ) external onlyOwner {
        Permissions.revokeTarget(roles[role], role, targetAddress);
    }

    /// @dev Scopes calls to an address, limited to specific function signatures, and per function scoping rules.
    /// @notice Only callable by owner.
    /// @param role Role to set for.
    /// @param targetAddress Address to be scoped.
    function scopeTarget(
        uint16 role,
        address targetAddress
    ) external onlyOwner {
        Permissions.scopeTarget(roles[role], role, targetAddress);
    }

    /// @dev Allows a specific function signature on a scoped target.
    /// @notice Only callable by owner.
    /// @param role Role to set for
    /// @param targetAddress Scoped address on which a function signature should be allowed.
    /// @param functionSig Function signature to be allowed.
    /// @param options Defines whether or not delegate calls and/or eth can be sent to the function.
    function scopeAllowFunction(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        ExecutionOptions options
    ) external onlyOwner {
        Permissions.scopeAllowFunction(
            roles[role],
            role,
            targetAddress,
            functionSig,
            options
        );
    }

    /// @dev Disallows a specific function signature on a scoped target.
    /// @notice Only callable by owner.
    /// @param role Role to set for
    /// @param targetAddress Scoped address on which a function signature should be disallowed.
    /// @param functionSig Function signature to be disallowed.
    function scopeRevokeFunction(
        uint16 role,
        address targetAddress,
        bytes4 functionSig
    ) external onlyOwner {
        Permissions.scopeRevokeFunction(
            roles[role],
            role,
            targetAddress,
            functionSig
        );
    }

    /// @dev Sets scoping rules for a function, on a scoped address.
    /// @notice Only callable by owner.
    /// @param role Role to set for.
    /// @param targetAddress Scoped address on which scoping rules for a function are to be set.
    /// @param functionSig Function signature to be scoped.
    /// @param isParamScoped false for un-scoped, true for scoped.
    /// @param paramType Static, Dynamic or Dynamic32, depending on the parameter type.
    /// @param paramComp Any, or EqualTo, GreaterThan, or LessThan, depending on comparison type.
    /// @param compValue The reference value used while comparing and authorizing.
    /// @param options Defines whether or not delegate calls and/or eth can be sent to the function.
    function scopeFunction(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        bool[] calldata isParamScoped,
        ParameterType[] calldata paramType,
        Comparison[] calldata paramComp,
        bytes[] memory compValue,
        ExecutionOptions options
    ) external onlyOwner {
        Permissions.scopeFunction(
            roles[role],
            role,
            targetAddress,
            functionSig,
            isParamScoped,
            paramType,
            paramComp,
            compValue,
            options
        );
    }

    /// @dev Sets whether or not delegate calls and/or eth can be sent to a function on a scoped target.
    /// @notice Only callable by owner.
    /// @notice Only in play when target is scoped.
    /// @param role Role to set for.
    /// @param targetAddress Scoped address on which the ExecutionOptions for a function are to be set.
    /// @param functionSig Function signature on which the ExecutionOptions are to be set.
    /// @param options Defines whether or not delegate calls and/or eth can be sent to the function.
    function scopeFunctionExecutionOptions(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        ExecutionOptions options
    ) external onlyOwner {
        Permissions.scopeFunctionExecutionOptions(
            roles[role],
            role,
            targetAddress,
            functionSig,
            options
        );
    }

    /// @dev Sets and enforces scoping rules, for a single parameter of a function, on a scoped target.
    /// @notice Only callable by owner.
    /// @param role Role to set for.
    /// @param targetAddress Scoped address on which functionSig lives.
    /// @param functionSig Function signature to be scoped.
    /// @param paramIndex The index of the parameter to scope.
    /// @param paramType Static, Dynamic or Dynamic32, depending on the parameter type.
    /// @param paramComp Any, or EqualTo, GreaterThan, or LessThan, depending on comparison type.
    /// @param compValue The reference value used while comparing and authorizing.
    function scopeParameter(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        uint256 paramIndex,
        ParameterType paramType,
        Comparison paramComp,
        bytes calldata compValue
    ) external onlyOwner {
        Permissions.scopeParameter(
            roles[role],
            role,
            targetAddress,
            functionSig,
            paramIndex,
            paramType,
            paramComp,
            compValue
        );
    }

    /// @dev Sets and enforces scoping rules, for a single parameter of a function, on a scoped target.
    /// @notice Only callable by owner.
    /// @notice Parameter will be scoped with comparison type OneOf.
    /// @param role Role to set for.
    /// @param targetAddress Scoped address on which functionSig lives.
    /// @param functionSig Function signature to be scoped.
    /// @param paramIndex The index of the parameter to scope.
    /// @param paramType Static, Dynamic or Dynamic32, depending on the parameter type.
    /// @param compValues The reference values used while comparing and authorizing.
    function scopeParameterAsOneOf(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        uint256 paramIndex,
        ParameterType paramType,
        bytes[] calldata compValues
    ) external onlyOwner {
        Permissions.scopeParameterAsOneOf(
            roles[role],
            role,
            targetAddress,
            functionSig,
            paramIndex,
            paramType,
            compValues
        );
    }

    /// @dev Un-scopes a single parameter of a function, on a scoped target.
    /// @notice Only callable by owner.
    /// @param role Role to set for.
    /// @param targetAddress Scoped address on which functionSig lives.
    /// @param functionSig Function signature to be scoped.
    /// @param paramIndex The index of the parameter to un-scope.
    function unscopeParameter(
        uint16 role,
        address targetAddress,
        bytes4 functionSig,
        uint8 paramIndex
    ) external onlyOwner {
        Permissions.unscopeParameter(
            roles[role],
            role,
            targetAddress,
            functionSig,
            paramIndex
        );
    }

    /// @dev Assigns and revokes roles to a given module.
    /// @param module Module on which to assign/revoke roles.
    /// @param _roles Roles to assign/revoke.
    /// @param memberOf Assign (true) or revoke (false) corresponding _roles.
    function assignRoles(
        address module,
        uint16[] calldata _roles,
        bool[] calldata memberOf
    ) external onlyOwner {
        if (_roles.length != memberOf.length) {
            revert ArraysDifferentLength();
        }
        for (uint16 i = 0; i < _roles.length; i++) {
            roles[_roles[i]].members[module] = memberOf[i];
        }
        if (!isModuleEnabled(module)) {
            enableModule(module);
        }
        emit AssignRoles(module, _roles, memberOf);
    }

    /// @dev Sets the default role used for a module if it calls execTransactionFromModule() or execTransactionFromModuleReturnData().
    /// @param module Address of the module on which to set default role.
    /// @param role Role to be set as default.
    function setDefaultRole(address module, uint16 role) external onlyOwner {
        defaultRoles[module] = role;
        emit SetDefaultRole(module, role);
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
    ) public override moduleOnly returns (bool success) {
        Permissions.check(
            roles[defaultRoles[msg.sender]],
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
    ) public override moduleOnly returns (bool, bytes memory) {
        Permissions.check(
            roles[defaultRoles[msg.sender]],
            multisend,
            to,
            value,
            data,
            operation
        );
        return execAndReturnData(to, value, data, operation);
    }

    /// @dev Passes a transaction to the modifier assuming the specified role.
    /// @param to Destination address of module transaction
    /// @param value Ether value of module transaction
    /// @param data Data payload of module transaction
    /// @param operation Operation type of module transaction
    /// @param role Identifier of the role to assume for this transaction
    /// @param shouldRevert Should the function revert on inner execution returning success false?
    /// @notice Can only be called by enabled modules
    function execTransactionWithRole(
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation,
        uint16 role,
        bool shouldRevert
    ) public moduleOnly returns (bool success) {
        Permissions.check(roles[role], multisend, to, value, data, operation);
        success = exec(to, value, data, operation);
        if (shouldRevert && !success) {
            revert ModuleTransactionFailed();
        }
    }

    /// @dev Passes a transaction to the modifier assuming the specified role. Expects return data.
    /// @param to Destination address of module transaction
    /// @param value Ether value of module transaction
    /// @param data Data payload of module transaction
    /// @param operation Operation type of module transaction
    /// @param role Identifier of the role to assume for this transaction
    /// @param shouldRevert Should the function revert on inner execution returning success false?
    /// @notice Can only be called by enabled modules
    function execTransactionWithRoleReturnData(
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation,
        uint16 role,
        bool shouldRevert
    ) public moduleOnly returns (bool success, bytes memory returnData) {
        Permissions.check(roles[role], multisend, to, value, data, operation);
        (success, returnData) = execAndReturnData(to, value, data, operation);
        if (shouldRevert && !success) {
            revert ModuleTransactionFailed();
        }
    }
}
