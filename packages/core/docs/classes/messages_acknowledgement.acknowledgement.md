[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / [messages/acknowledgement](../modules/messages_acknowledgement.md) / Acknowledgement

# Class: Acknowledgement

[messages/acknowledgement](../modules/messages_acknowledgement.md).Acknowledgement

## Table of contents

### Constructors

- [constructor](messages_acknowledgement.acknowledgement.md#constructor)

### Properties

- [ackKeyShare](messages_acknowledgement.acknowledgement.md#ackkeyshare)

### Accessors

- [ackChallenge](messages_acknowledgement.acknowledgement.md#ackchallenge)
- [SIZE](messages_acknowledgement.acknowledgement.md#size)

### Methods

- [serialize](messages_acknowledgement.acknowledgement.md#serialize)
- [create](messages_acknowledgement.acknowledgement.md#create)
- [deserialize](messages_acknowledgement.acknowledgement.md#deserialize)

## Constructors

### constructor

\+ `Private` **new Acknowledgement**(`ackSignature`: _Uint8Array_, `challengeSignature`: _Uint8Array_, `ackKeyShare`: _Uint8Array_): [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name                 | Type         |
| :------------------- | :----------- |
| `ackSignature`       | _Uint8Array_ |
| `challengeSignature` | _Uint8Array_ |
| `ackKeyShare`        | _Uint8Array_ |

**Returns:** [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/acknowledgement.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L8)

## Properties

### ackKeyShare

• **ackKeyShare**: _Uint8Array_

## Accessors

### ackChallenge

• get **ackChallenge**(): _PublicKey_

**Returns:** _PublicKey_

Defined in: [packages/core/src/messages/acknowledgement.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L63)

---

### SIZE

• `Static` get **SIZE**(): _number_

**Returns:** _number_

Defined in: [packages/core/src/messages/acknowledgement.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L15)

## Methods

### serialize

▸ **serialize**(): _Uint8Array_

**Returns:** _Uint8Array_

Defined in: [packages/core/src/messages/acknowledgement.ts:67](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L67)

---

### create

▸ `Static` **create**(`challenge`: [_Challenge_](messages_challenge.challenge.md), `derivedSecret`: _Uint8Array_, `privKey`: _PeerId_): [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name            | Type                                           |
| :-------------- | :--------------------------------------------- |
| `challenge`     | [_Challenge_](messages_challenge.challenge.md) |
| `derivedSecret` | _Uint8Array_                                   |
| `privKey`       | _PeerId_                                       |

**Returns:** [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/acknowledgement.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L19)

---

### deserialize

▸ `Static` **deserialize**(`preArray`: _Uint8Array_, `ownPubKey`: _PeerId_, `senderPubKey`: _PeerId_): [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name           | Type         |
| :------------- | :----------- |
| `preArray`     | _Uint8Array_ |
| `ownPubKey`    | _PeerId_     |
| `senderPubKey` | _PeerId_     |

**Returns:** [_Acknowledgement_](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/acknowledgement.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L28)
