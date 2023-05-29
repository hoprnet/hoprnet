## Difference
This is a simplified `Permission.sol` contract from `zodiac-modifier-roles-v1`.
A fundamental difference is that **this library supports only one role**.

- No `DelegateCall` or `Send` is allowed to be executed from the module. Reducing the `ExecutionOptions` to possibly be `None`, which is the ordinary `call`.
### types
1. Remove `Target` from `Clearance`, so target addresses can only be scoped (`Clearance.Function`) or not allowed (`Clearance.None`)


### setters
1. `allowTarget` is deleted. `allowTarget` allows role members to call arbitrary functions of any type (call & delegatecall) to a given target. This function is NOT supported by NodeManagement, because we want to scope functions. Event `AllowTarget` is thus removed.
2. `revokeTarget`, which is the reciprocal function of `allowTarget`. It is STILL supported by NodeManagement as a shortcut to block all the transactions going to the target. However, as there's only one role, `roleId` parameter of this function and of `RevokeTarget` event are removed.
3. `scopeTarget` allows certain functions of the target address to be called. It is STILL suppored, as it fulfills the purpose of managing a scope. However, as there's only one role, `roleId` parameter of this function and of `ScopeTarget` event are removed.
4. `scopeAllowFunction` allows a specific function signature on a scoped target. It's the default config for all the functions. However, as there's only one role, `roleId` parameter of this function and of `ScopeAllowFunction` event are removed.
5. `scopeRevokeFunction` allows a specific function signature on a scoped target. However, as there's only one role, `roleId` parameter of this function and of `ScopeRevokeFunction` event are removed.
5. `scopeFunction` allows a specific function signature on a scoped target. However, as there's only one role, `roleId` parameter of this function and of `ScopeFunction` event are removed.
6. `checkTransaction` is the main transaction that checks the permission of transaction execution of a module. It by default doesn't 

- Added NatSpec comments on `packLeft`, `packRight`, `unpackFunction`, `unpackParameter`, `checkExecutionOptions`

- Added specific checks for HoprChannels  `checkHoprChannelsParameters`;
