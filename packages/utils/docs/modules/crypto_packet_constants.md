[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / crypto/packet/constants

# Module: crypto/packet/constants

## Table of contents

### Variables

- [END\_PREFIX](crypto_packet_constants.md#end_prefix)
- [END\_PREFIX\_LENGTH](crypto_packet_constants.md#end_prefix_length)
- [HASH\_ALGORITHM](crypto_packet_constants.md#hash_algorithm)
- [HASH\_LENGTH](crypto_packet_constants.md#hash_length)
- [MAC\_LENGTH](crypto_packet_constants.md#mac_length)
- [PAYLOAD\_SIZE](crypto_packet_constants.md#payload_size)
- [SECRET\_LENGTH](crypto_packet_constants.md#secret_length)
- [TAG\_LENGTH](crypto_packet_constants.md#tag_length)

## Variables

### END\_PREFIX

• `Const` **END\_PREFIX**: ``255``= 0xff

Prefix that signals a relayer that it is the final
recipient

Defined in: [crypto/packet/constants.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L29)

___

### END\_PREFIX\_LENGTH

• `Const` **END\_PREFIX\_LENGTH**: ``1``= 1

Defined in: [crypto/packet/constants.ts:30](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L30)

___

### HASH\_ALGORITHM

• `Const` **HASH\_ALGORITHM**: ``"blake2s256"``= 'blake2s256'

Hash algorithm that is used to derive the shared secrets
and its output length

Defined in: [crypto/packet/constants.ts:5](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L5)

___

### HASH\_LENGTH

• `Const` **HASH\_LENGTH**: ``32``= 32

Defined in: [crypto/packet/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L6)

___

### MAC\_LENGTH

• `Const` **MAC\_LENGTH**: ``32``

Length of the MAC as used for integrity protection
of mixnet packets

Defined in: [crypto/packet/constants.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L18)

___

### PAYLOAD\_SIZE

• `Const` **PAYLOAD\_SIZE**: ``500``= 500

Size of the payload per packet

Defined in: [crypto/packet/constants.ts:23](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L23)

___

### SECRET\_LENGTH

• `Const` **SECRET\_LENGTH**: ``32``

Length of the shared secret that is derived from
the mixnet packet

Defined in: [crypto/packet/constants.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L12)

___

### TAG\_LENGTH

• `Const` **TAG\_LENGTH**: ``16``= 16

Length of the tag used to prevent from replay attacks

Defined in: [crypto/packet/constants.ts:35](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/crypto/packet/constants.ts#L35)
