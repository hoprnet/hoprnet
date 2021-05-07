[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / contracts/commons

# Module: contracts/commons

## Table of contents

### Interfaces

- [TypedEvent](../interfaces/contracts_commons.typedevent.md)
- [TypedEventFilter](../interfaces/contracts_commons.typedeventfilter.md)

### Type aliases

- [TypedListener](contracts_commons.md#typedlistener)

## Type aliases

### TypedListener

Ƭ **TypedListener**<EventArgsArray, EventArgsObject\>: (...`listenerArg`: [...EventArgsArray, [*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>]) => *void*

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Type declaration

▸ (...`listenerArg`: [...EventArgsArray, [*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>]): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `...listenerArg` | [...EventArgsArray, [*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>] |

**Returns:** *void*

Defined in: packages/core-ethereum/src/contracts/commons.ts:15
