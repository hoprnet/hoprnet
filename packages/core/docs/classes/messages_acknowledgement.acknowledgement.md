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

\+ `Private` **new Acknowledgement**(`ackSignature`: *Uint8Array*, `challengeSignature`: *Uint8Array*, `ackKeyShare`: *Uint8Array*): [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `ackSignature` | *Uint8Array* |
| `challengeSignature` | *Uint8Array* |
| `ackKeyShare` | *Uint8Array* |

**Returns:** [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/acknowledgement.ts:8](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L8)

## Properties

### ackKeyShare

• **ackKeyShare**: *Uint8Array*

## Accessors

### ackChallenge

• get **ackChallenge**(): *PublicKey*

**Returns:** *PublicKey*

Defined in: [packages/core/src/messages/acknowledgement.ts:63](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L63)

___

### SIZE

• `Static` get **SIZE**(): *number*

**Returns:** *number*

Defined in: [packages/core/src/messages/acknowledgement.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L15)

## Methods

### serialize

▸ **serialize**(): *Uint8Array*

**Returns:** *Uint8Array*

Defined in: [packages/core/src/messages/acknowledgement.ts:67](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L67)

___

### create

▸ `Static` **create**(`challenge`: [*Challenge*](messages_challenge.challenge.md), `derivedSecret`: *Uint8Array*, `privKey`: *PeerId*): [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `challenge` | [*Challenge*](messages_challenge.challenge.md) |
| `derivedSecret` | *Uint8Array* |
| `privKey` | *PeerId* |

**Returns:** [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/acknowledgement.ts:19](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L19)

___

### deserialize

▸ `Static` **deserialize**(`preArray`: *Uint8Array*, `ownPubKey`: *PeerId*, `senderPubKey`: *PeerId*): [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `preArray` | *Uint8Array* |
| `ownPubKey` | *PeerId* |
| `senderPubKey` | *PeerId* |

**Returns:** [*Acknowledgement*](messages_acknowledgement.acknowledgement.md)

Defined in: [packages/core/src/messages/acknowledgement.ts:28](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/messages/acknowledgement.ts#L28)
