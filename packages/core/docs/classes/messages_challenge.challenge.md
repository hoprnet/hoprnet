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

\+ `Private` **new Challenge**(`ackChallenge`: _Uint8Array_, `signature`: _Uint8Array_): [_Challenge_](messages_challenge.challenge.md)

#### Parameters

| Name           | Type         |
| :------------- | :----------- |
| `ackChallenge` | _Uint8Array_ |
| `signature`    | _Uint8Array_ |

**Returns:** [_Challenge_](messages_challenge.challenge.md)

Defined in: [packages/core/src/messages/challenge.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L8)

## Accessors

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [packages/core/src/messages/challenge.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L11)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [packages/core/src/messages/challenge.ts:34](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L34)

---

### solve

▸ **solve**(`secret`: _Uint8Array_): _boolean_

#### Parameters

| Name     | Type         |
| :------- | :----------- |
| `secret` | _Uint8Array_ |

**Returns:** _boolean_

Defined in: [packages/core/src/messages/challenge.ts:50](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L50)

---

### create

▸ `Static` **create**(`ackChallenge`: _Uint8Array_, `privKey`: _PeerId_): [_Challenge_](messages_challenge.challenge.md)

#### Parameters

| Name           | Type         |
| :------------- | :----------- |
| `ackChallenge` | _Uint8Array_ |
| `privKey`      | _PeerId_     |

**Returns:** [_Challenge_](messages_challenge.challenge.md)

Defined in: [packages/core/src/messages/challenge.ts:38](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L38)

---

### deserialize

▸ `Static` **deserialize**(`preArray`: _Uint8Array_ \| _Buffer_, `ackChallenge`: _Uint8Array_, `pubKey`: _PeerId_): [_Challenge_](messages_challenge.challenge.md)

#### Parameters

| Name           | Type                     |
| :------------- | :----------------------- |
| `preArray`     | _Uint8Array_ \| _Buffer_ |
| `ackChallenge` | _Uint8Array_             |
| `pubKey`       | _PeerId_                 |

**Returns:** [_Challenge_](messages_challenge.challenge.md)

Defined in: [packages/core/src/messages/challenge.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/challenge.ts#L15)
