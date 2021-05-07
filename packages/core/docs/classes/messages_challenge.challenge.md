[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [messages/challenge](../modules/messages_challenge.md) / Challenge

# Class: Challenge

[messages/challenge](../modules/messages_challenge.md).Challenge

## Table of contents

### Constructors

- [constructor](messages_challenge.challenge.md#constructor)

### Accessors

- [SIZE](messages_challenge.challenge.md#size)

### Methods

- [serialize](messages_challenge.challenge.md#serialize)
- [solve](messages_challenge.challenge.md#solve)
- [create](messages_challenge.challenge.md#create)
- [deserialize](messages_challenge.challenge.md#deserialize)

## Constructors

### constructor

\+ `Private` **new Challenge**(`ackChallenge`: *Uint8Array*, `signature`: *Uint8Array*): [*Challenge*](messages_challenge.challenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackChallenge` | *Uint8Array* |
| `signature` | *Uint8Array* |

**Returns:** [*Challenge*](messages_challenge.challenge.md)

Defined in: [packages/core/src/messages/challenge.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L8)

## Accessors

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [packages/core/src/messages/challenge.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L11)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [packages/core/src/messages/challenge.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L34)

___

### solve

▸ **solve**(`secret`: *Uint8Array*): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `secret` | *Uint8Array* |

**Returns:** *boolean*

Defined in: [packages/core/src/messages/challenge.ts:50](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L50)

___

### create

▸ `Static` **create**(`ackChallenge`: *Uint8Array*, `privKey`: *PeerId*): [*Challenge*](messages_challenge.challenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackChallenge` | *Uint8Array* |
| `privKey` | *PeerId* |

**Returns:** [*Challenge*](messages_challenge.challenge.md)

Defined in: [packages/core/src/messages/challenge.ts:38](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L38)

___

### deserialize

▸ `Static` **deserialize**(`preArray`: *Uint8Array* \| *Buffer*, `ackChallenge`: *Uint8Array*, `pubKey`: *PeerId*): [*Challenge*](messages_challenge.challenge.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `preArray` | *Uint8Array* \| *Buffer* |
| `ackChallenge` | *Uint8Array* |
| `pubKey` | *PeerId* |

**Returns:** [*Challenge*](messages_challenge.challenge.md)

Defined in: [packages/core/src/messages/challenge.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L15)
