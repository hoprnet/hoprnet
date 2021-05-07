[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/utils/checkPeerId

# Module: commands/utils/checkPeerId

## Table of contents

### Functions

- [checkPeerIdInput](commands_utils_checkpeerid.md#checkpeeridinput)
- [getPeerIdsAndAliases](commands_utils_checkpeerid.md#getpeeridsandaliases)

## Functions

### checkPeerIdInput

▸ **checkPeerIdInput**(`peerIdString`: *string*, `state?`: [*GlobalState*](commands_abstractcommand.md#globalstate)): *Promise*<PeerId\>

Takes a string, and checks whether it's an alias or a valid peerId,
then it generates a PeerId instance and returns it.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peerIdString` | *string* | query that contains the peerId |
| `state?` | [*GlobalState*](commands_abstractcommand.md#globalstate) | - |

**Returns:** *Promise*<PeerId\>

a 'PeerId' instance

Defined in: [commands/utils/checkPeerId.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/checkPeerId.ts#L12)

___

### getPeerIdsAndAliases

▸ **getPeerIdsAndAliases**(`node`: Hopr, `state`: [*GlobalState*](commands_abstractcommand.md#globalstate), `ops?`: { `mustBeOnline`: *boolean* ; `returnAlias`: *boolean*  }): *string*[]

Returns a list of peerIds and aliases.
Optionally, you may choose various options.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `node` | Hopr | hopr node |
| `state` | [*GlobalState*](commands_abstractcommand.md#globalstate) | global state |
| `ops` | *object* | - |
| `ops.mustBeOnline` | *boolean* | only return online peerIds |
| `ops.returnAlias` | *boolean* | when available, return the peerIds's alias |

**Returns:** *string*[]

an array of peerIds / aliases

Defined in: [commands/utils/checkPeerId.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/checkPeerId.ts#L34)
