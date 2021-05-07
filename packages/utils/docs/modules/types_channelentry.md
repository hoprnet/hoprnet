[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / types/channelEntry

# Module: types/channelEntry

## Table of contents

### Classes

- [ChannelEntry](../classes/types_channelentry.channelentry.md)

### Type aliases

- [ChannelStatus](types_channelentry.md#channelstatus)

### Functions

- [generateChannelId](types_channelentry.md#generatechannelid)

## Type aliases

### ChannelStatus

Ƭ **ChannelStatus**: `"CLOSED"` \| `"OPEN"` \| `"PENDING_TO_CLOSE"`

Defined in: [types/channelEntry.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L6)

## Functions

### generateChannelId

▸ **generateChannelId**(`self`: [_Address_](../classes/types_primitives.address.md), `counterparty`: [_Address_](../classes/types_primitives.address.md)): [_Hash_](../classes/types_primitives.hash.md)

#### Parameters

| Name           | Type                                                |
| :------------- | :-------------------------------------------------- |
| `self`         | [_Address_](../classes/types_primitives.address.md) |
| `counterparty` | [_Address_](../classes/types_primitives.address.md) |

**Returns:** [_Hash_](../classes/types_primitives.hash.md)

Defined in: [types/channelEntry.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/types/channelEntry.ts#L8)
