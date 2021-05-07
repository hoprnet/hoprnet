[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / indexer/types

# Module: indexer/types

## Table of contents

### Type aliases

- [Event](indexer_types.md#event)
- [EventNames](indexer_types.md#eventnames)

## Type aliases

### Event

Ƭ **Event**<T\>: [*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<extractEventArgs<ReturnType<[*HoprChannels*](../classes/contracts_hoprchannels.hoprchannels.md)[``"filters"``][T]\>\>\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `T` | [*EventNames*](indexer_types.md#eventnames) |

Defined in: [packages/core-ethereum/src/indexer/types.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/types.ts#L12)

___

### EventNames

Ƭ **EventNames**: keyof [*HoprChannels*](../classes/contracts_hoprchannels.hoprchannels.md)[``"filters"``]

Defined in: [packages/core-ethereum/src/indexer/types.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/types.ts#L11)
