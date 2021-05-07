[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet/constants

# Module: crypto/packet/constants

## Table of contents

### Variables

- [END_PREFIX](crypto_packet_constants.md#end_prefix)
- [END_PREFIX_LENGTH](crypto_packet_constants.md#end_prefix_length)
- [HASH_ALGORITHM](crypto_packet_constants.md#hash_algorithm)
- [HASH_LENGTH](crypto_packet_constants.md#hash_length)
- [MAC_LENGTH](crypto_packet_constants.md#mac_length)
- [PAYLOAD_SIZE](crypto_packet_constants.md#payload_size)
- [SECRET_LENGTH](crypto_packet_constants.md#secret_length)
- [TAG_LENGTH](crypto_packet_constants.md#tag_length)

## Variables

### END_PREFIX

• `Const` **END_PREFIX**: `255`= 0xff

Prefix that signals a relayer that it is the final
recipient

Defined in: [crypto/packet/constants.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L29)

---

### END_PREFIX_LENGTH

• `Const` **END_PREFIX_LENGTH**: `1`= 1

Defined in: [crypto/packet/constants.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L30)

---

### HASH_ALGORITHM

• `Const` **HASH_ALGORITHM**: `"blake2s256"`= 'blake2s256'

Hash algorithm that is used to derive the shared secrets
and its output length

Defined in: [crypto/packet/constants.ts:5](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L5)

---

### HASH_LENGTH

• `Const` **HASH_LENGTH**: `32`= 32

Defined in: [crypto/packet/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L6)

---

### MAC_LENGTH

• `Const` **MAC_LENGTH**: `32`

Length of the MAC as used for integrity protection
of mixnet packets

Defined in: [crypto/packet/constants.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L18)

---

### PAYLOAD_SIZE

• `Const` **PAYLOAD_SIZE**: `500`= 500

Size of the payload per packet

Defined in: [crypto/packet/constants.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L23)

---

### SECRET_LENGTH

• `Const` **SECRET_LENGTH**: `32`

Length of the shared secret that is derived from
the mixnet packet

Defined in: [crypto/packet/constants.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L12)

---

### TAG_LENGTH

• `Const` **TAG_LENGTH**: `16`= 16

Length of the tag used to prevent from replay attacks

Defined in: [crypto/packet/constants.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L35)
