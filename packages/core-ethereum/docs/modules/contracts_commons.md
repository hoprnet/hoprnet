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

Ƭ **TypedListener**<EventArgsArray, EventArgsObject\>: (...`listenerArg`: [...EventArgsArray, [_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>]) => _void_

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Type declaration

▸ (...`listenerArg`: [...EventArgsArray, [_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>]): _void_

#### Parameters

| Name             | Type                                                                                                                  |
| :--------------- | :-------------------------------------------------------------------------------------------------------------------- |
| `...listenerArg` | [...EventArgsArray, [_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>] |

**Returns:** _void_

Defined in: packages/core-ethereum/src/contracts/commons.ts:15
