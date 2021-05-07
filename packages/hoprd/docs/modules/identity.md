[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / identity

# Module: identity

## Table of contents

### Type aliases

- [IdentityOptions](identity.md#identityoptions)

### Variables

- [KEYPAIR_CIPHER_ALGORITHM](identity.md#keypair_cipher_algorithm)
- [KEYPAIR_CIPHER_KEY_LENGTH](identity.md#keypair_cipher_key_length)
- [KEYPAIR_IV_LENGTH](identity.md#keypair_iv_length)
- [KEYPAIR_MESSAGE_DIGEST_ALGORITHM](identity.md#keypair_message_digest_algorithm)
- [KEYPAIR_PADDING](identity.md#keypair_padding)
- [KEYPAIR_SALT_LENGTH](identity.md#keypair_salt_length)
- [KEYPAIR_SCRYPT_PARAMS](identity.md#keypair_scrypt_params)

### Functions

- [deserializeKeyPair](identity.md#deserializekeypair)
- [getIdentity](identity.md#getidentity)
- [serializeKeyPair](identity.md#serializekeypair)

## Type aliases

### IdentityOptions

Ƭ **IdentityOptions**: _object_

#### Type declaration

| Name         | Type      |
| :----------- | :-------- |
| `idPath`     | _string_  |
| `initialize` | _boolean_ |
| `password`   | _string_  |

Defined in: [identity.ts:78](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L78)

## Variables

### KEYPAIR_CIPHER_ALGORITHM

• `Const` **KEYPAIR_CIPHER_ALGORITHM**: `"chacha20"`= 'chacha20'

Defined in: [identity.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L10)

---

### KEYPAIR_CIPHER_KEY_LENGTH

• `Const` **KEYPAIR_CIPHER_KEY_LENGTH**: `32`= 32

Defined in: [identity.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L12)

---

### KEYPAIR_IV_LENGTH

• `Const` **KEYPAIR_IV_LENGTH**: `16`= 16

Defined in: [identity.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L11)

---

### KEYPAIR_MESSAGE_DIGEST_ALGORITHM

• `Const` **KEYPAIR_MESSAGE_DIGEST_ALGORITHM**: `"sha256"`= 'sha256'

Defined in: [identity.ts:16](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L16)

---

### KEYPAIR_PADDING

• `Const` **KEYPAIR_PADDING**: _Buffer_

Defined in: [identity.ts:15](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L15)

---

### KEYPAIR_SALT_LENGTH

• `Const` **KEYPAIR_SALT_LENGTH**: `32`= 32

Defined in: [identity.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L13)

---

### KEYPAIR_SCRYPT_PARAMS

• `Const` **KEYPAIR_SCRYPT_PARAMS**: _object_

#### Type declaration

| Name | Type     |
| :--- | :------- |
| `N`  | _number_ |
| `p`  | _number_ |
| `r`  | _number_ |

Defined in: [identity.ts:14](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L14)

## Functions

### deserializeKeyPair

▸ **deserializeKeyPair**(`encryptedSerializedKeyPair`: Uint8Array, `password`: Uint8Array): _Promise_<PeerId\>

Deserializes a serialized key pair and returns a peerId.

**`notice`** This method will ask for a password to decrypt the encrypted
private key.

**`notice`** The decryption of the private key makes use of a memory-hard
hash function and consumes therefore a lot of memory.

#### Parameters

| Name                         | Type       | Description                        |
| :--------------------------- | :--------- | :--------------------------------- |
| `encryptedSerializedKeyPair` | Uint8Array | the encoded and encrypted key pair |
| `password`                   | Uint8Array | -                                  |

**Returns:** _Promise_<PeerId\>

Defined in: [identity.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L49)

---

### getIdentity

▸ **getIdentity**(`options`: [_IdentityOptions_](identity.md#identityoptions)): _Promise_<PeerId\>

#### Parameters

| Name      | Type                                             |
| :-------- | :----------------------------------------------- |
| `options` | [_IdentityOptions_](identity.md#identityoptions) |

**Returns:** _Promise_<PeerId\>

Defined in: [identity.ts:100](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L100)

---

### serializeKeyPair

▸ **serializeKeyPair**(`peerId`: PeerId, `password`: Uint8Array): _Uint8Array_

Serializes a given peerId by serializing the included private key and public key.

#### Parameters

| Name       | Type       | Description                          |
| :--------- | :--------- | :----------------------------------- |
| `peerId`   | PeerId     | the peerId that should be serialized |
| `password` | Uint8Array | -                                    |

**Returns:** _Uint8Array_

Defined in: [identity.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/identity.ts#L23)
