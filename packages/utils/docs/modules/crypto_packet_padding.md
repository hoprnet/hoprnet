[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet/padding

# Module: crypto/packet/padding

## Table of contents

### Variables

- [PADDING_TAG](crypto_packet_padding.md#padding_tag)
- [PADDING_TAG_LENGTH](crypto_packet_padding.md#padding_tag_length)

### Functions

- [addPadding](crypto_packet_padding.md#addpadding)
- [removePadding](crypto_packet_padding.md#removepadding)

## Variables

### PADDING_TAG

• `Const` **PADDING_TAG**: _Uint8Array_

Defined in: [crypto/packet/padding.ts:4](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/padding.ts#L4)

---

### PADDING_TAG_LENGTH

• `Const` **PADDING_TAG_LENGTH**: `4`= 4

Defined in: [crypto/packet/padding.ts:5](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/padding.ts#L5)

## Functions

### addPadding

▸ **addPadding**(`msg`: Uint8Array): _Uint8Array_

Adds a deterministic padding to a given payload.

**`dev`** payloads that do not include the correct padding are
considered invalid

#### Parameters

| Name  | Type       | Description        |
| :---- | :--------- | :----------------- |
| `msg` | Uint8Array | the payload to pad |

**Returns:** _Uint8Array_

the padded payload

Defined in: [crypto/packet/padding.ts:14](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/padding.ts#L14)

---

### removePadding

▸ **removePadding**(`decoded`: Uint8Array): _Uint8Array_

Removes the padding from a given payload and fails if
the padding does not exist or if the payload has the
wrong size.

#### Parameters

| Name      | Type       | Description      |
| :-------- | :--------- | :--------------- |
| `decoded` | Uint8Array | a padded payload |

**Returns:** _Uint8Array_

the message without the padding

Defined in: [crypto/packet/padding.ts:31](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/padding.ts#L31)
