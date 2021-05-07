[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / identity

# Module: identity

## Table of contents

### Type aliases

- [IdentityOptions](identity.md#identityoptions)

### Variables

- [KEYPAIR\_CIPHER\_ALGORITHM](identity.md#keypair_cipher_algorithm)
- [KEYPAIR\_CIPHER\_KEY\_LENGTH](identity.md#keypair_cipher_key_length)
- [KEYPAIR\_IV\_LENGTH](identity.md#keypair_iv_length)
- [KEYPAIR\_MESSAGE\_DIGEST\_ALGORITHM](identity.md#keypair_message_digest_algorithm)
- [KEYPAIR\_PADDING](identity.md#keypair_padding)
- [KEYPAIR\_SALT\_LENGTH](identity.md#keypair_salt_length)
- [KEYPAIR\_SCRYPT\_PARAMS](identity.md#keypair_scrypt_params)

### Functions

- [deserializeKeyPair](identity.md#deserializekeypair)
- [getIdentity](identity.md#getidentity)
- [serializeKeyPair](identity.md#serializekeypair)

## Type aliases

### IdentityOptions

Ƭ **IdentityOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `idPath` | *string* |
| `initialize` | *boolean* |
| `password` | *string* |

Defined in: [identity.ts:78](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L78)

## Variables

### KEYPAIR\_CIPHER\_ALGORITHM

• `Const` **KEYPAIR\_CIPHER\_ALGORITHM**: ``"chacha20"``= 'chacha20'

Defined in: [identity.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L10)

___

### KEYPAIR\_CIPHER\_KEY\_LENGTH

• `Const` **KEYPAIR\_CIPHER\_KEY\_LENGTH**: ``32``= 32

Defined in: [identity.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L12)

___

### KEYPAIR\_IV\_LENGTH

• `Const` **KEYPAIR\_IV\_LENGTH**: ``16``= 16

Defined in: [identity.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L11)

___

### KEYPAIR\_MESSAGE\_DIGEST\_ALGORITHM

• `Const` **KEYPAIR\_MESSAGE\_DIGEST\_ALGORITHM**: ``"sha256"``= 'sha256'

Defined in: [identity.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L16)

___

### KEYPAIR\_PADDING

• `Const` **KEYPAIR\_PADDING**: *Buffer*

Defined in: [identity.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L15)

___

### KEYPAIR\_SALT\_LENGTH

• `Const` **KEYPAIR\_SALT\_LENGTH**: ``32``= 32

Defined in: [identity.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L13)

___

### KEYPAIR\_SCRYPT\_PARAMS

• `Const` **KEYPAIR\_SCRYPT\_PARAMS**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `N` | *number* |
| `p` | *number* |
| `r` | *number* |

Defined in: [identity.ts:14](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L14)

## Functions

### deserializeKeyPair

▸ **deserializeKeyPair**(`encryptedSerializedKeyPair`: Uint8Array, `password`: Uint8Array): *Promise*<PeerId\>

Deserializes a serialized key pair and returns a peerId.

**`notice`** This method will ask for a password to decrypt the encrypted
private key.

**`notice`** The decryption of the private key makes use of a memory-hard
hash function and consumes therefore a lot of memory.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `encryptedSerializedKeyPair` | Uint8Array | the encoded and encrypted key pair |
| `password` | Uint8Array | - |

**Returns:** *Promise*<PeerId\>

Defined in: [identity.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L49)

___

### getIdentity

▸ **getIdentity**(`options`: [*IdentityOptions*](identity.md#identityoptions)): *Promise*<PeerId\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `options` | [*IdentityOptions*](identity.md#identityoptions) |

**Returns:** *Promise*<PeerId\>

Defined in: [identity.ts:100](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L100)

___

### serializeKeyPair

▸ **serializeKeyPair**(`peerId`: PeerId, `password`: Uint8Array): *Uint8Array*

Serializes a given peerId by serializing the included private key and public key.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peerId` | PeerId | the peerId that should be serialized |
| `password` | Uint8Array | - |

**Returns:** *Uint8Array*

Defined in: [identity.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L23)
