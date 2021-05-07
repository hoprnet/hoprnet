[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/utils/checkPeerId

# Module: commands/utils/checkPeerId

## Table of contents

### Functions

- [checkPeerIdInput](commands_utils_checkpeerid.md#checkpeeridinput)
- [getPeerIdsAndAliases](commands_utils_checkpeerid.md#getpeeridsandaliases)

## Functions

### checkPeerIdInput

▸ **checkPeerIdInput**(`peerIdString`: _string_, `state?`: [_GlobalState_](commands_abstractcommand.md#globalstate)): _Promise_<PeerId\>

Takes a string, and checks whether it's an alias or a valid peerId,
then it generates a PeerId instance and returns it.

#### Parameters

| Name           | Type                                                     | Description                    |
| :------------- | :------------------------------------------------------- | :----------------------------- |
| `peerIdString` | _string_                                                 | query that contains the peerId |
| `state?`       | [_GlobalState_](commands_abstractcommand.md#globalstate) | -                              |

**Returns:** _Promise_<PeerId\>

a 'PeerId' instance

Defined in: [commands/utils/checkPeerId.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/checkPeerId.ts#L12)

---

### getPeerIdsAndAliases

▸ **getPeerIdsAndAliases**(`node`: Hopr, `state`: [_GlobalState_](commands_abstractcommand.md#globalstate), `ops?`: { `mustBeOnline`: _boolean_ ; `returnAlias`: _boolean_ }): _string_[]

Returns a list of peerIds and aliases.
Optionally, you may choose various options.

#### Parameters

| Name               | Type                                                     | Description                                |
| :----------------- | :------------------------------------------------------- | :----------------------------------------- |
| `node`             | Hopr                                                     | hopr node                                  |
| `state`            | [_GlobalState_](commands_abstractcommand.md#globalstate) | global state                               |
| `ops`              | _object_                                                 | -                                          |
| `ops.mustBeOnline` | _boolean_                                                | only return online peerIds                 |
| `ops.returnAlias`  | _boolean_                                                | when available, return the peerIds's alias |

**Returns:** _string_[]

an array of peerIds / aliases

Defined in: [commands/utils/checkPeerId.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/checkPeerId.ts#L34)
