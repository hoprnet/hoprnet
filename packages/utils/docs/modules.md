[@hoprnet/hopr-utils](README.md) / Exports

# @hoprnet/hopr-utils

## Table of contents

### Classes

- [AccountEntry](classes/accountentry.md)
- [AcknowledgedTicket](classes/acknowledgedticket.md)
- [Address](classes/address.md)
- [Balance](classes/balance.md)
- [ChannelEntry](classes/channelentry.md)
- [Hash](classes/hash.md)
- [HoprDB](classes/hoprdb.md)
- [NativeBalance](classes/nativebalance.md)
- [PRG](classes/prg.md)
- [PRP](classes/prp.md)
- [PublicKey](classes/publickey.md)
- [Signature](classes/signature.md)
- [Snapshot](classes/snapshot.md)
- [Ticket](classes/ticket.md)
- [UINT256](classes/uint256.md)
- [UnacknowledgedTicket](classes/unacknowledgedticket.md)

### Interfaces

- [Intermediate](interfaces/intermediate.md)
- [NetOptions](interfaces/netoptions.md)

### Type aliases

- [ChannelStatus](modules.md#channelstatus)
- [DialOpts](modules.md#dialopts)
- [DialResponse](modules.md#dialresponse)
- [Hosts](modules.md#hosts)
- [LibP2PHandlerArgs](modules.md#libp2phandlerargs)
- [LibP2PHandlerFunction](modules.md#libp2phandlerfunction)
- [PRGParameters](modules.md#prgparameters)
- [PRPParameters](modules.md#prpparameters)
- [PromiseValue](modules.md#promisevalue)
- [U8aAndSize](modules.md#u8aandsize)

### Variables

- [ADDRESS_LENGTH](modules.md#address_length)
- [A_EQUALS_B](modules.md#a_equals_b)
- [A_STRICLY_LESS_THAN_B](modules.md#a_stricly_less_than_b)
- [A_STRICTLY_GREATER_THAN_B](modules.md#a_strictly_greater_than_b)
- [HASH_LENGTH](modules.md#hash_length)
- [LENGTH_PREFIX_LENGTH](modules.md#length_prefix_length)
- [MULTI_ADDR_MAX_LENGTH](modules.md#multi_addr_max_length)
- [POR_STRING_LENGTH](modules.md#por_string_length)
- [PRG_COUNTER_LENGTH](modules.md#prg_counter_length)
- [PRG_IV_LENGTH](modules.md#prg_iv_length)
- [PRG_KEY_LENGTH](modules.md#prg_key_length)
- [PRIVATE_KEY_LENGTH](modules.md#private_key_length)
- [PRP_IV_LENGTH](modules.md#prp_iv_length)
- [PRP_KEY_LENGTH](modules.md#prp_key_length)
- [PRP_MIN_LENGTH](modules.md#prp_min_length)
- [PUBLIC_KEY_LENGTH](modules.md#public_key_length)
- [SECP256K1_CONSTANTS](modules.md#secp256k1_constants)
- [SIGNATURE_LENGTH](modules.md#signature_length)
- [SIGNATURE_RECOVERY_LENGTH](modules.md#signature_recovery_length)
- [UNCOMPRESSED_PUBLIC_KEY_LENGTH](modules.md#uncompressed_public_key_length)
- [b58StringRegex](modules.md#b58stringregex)
- [durations](modules.md#durations)

### Functions

- [UnAcknowledgedTickets](modules.md#unacknowledgedtickets)
- [cacheNoArgAsyncFunction](modules.md#cachenoargasyncfunction)
- [convertPubKeyFromB58String](modules.md#convertpubkeyfromb58string)
- [convertPubKeyFromPeerId](modules.md#convertpubkeyfrompeerid)
- [createFirstChallenge](modules.md#createfirstchallenge)
- [createPacket](modules.md#createpacket)
- [createPoRString](modules.md#createporstring)
- [deriveAckKeyShare](modules.md#deriveackkeyshare)
- [dial](modules.md#dial)
- [forwardTransform](modules.md#forwardtransform)
- [gcd](modules.md#gcd)
- [generateChannelId](modules.md#generatechannelid)
- [generateKeyShares](modules.md#generatekeyshares)
- [getB58String](modules.md#getb58string)
- [getHeaderLength](modules.md#getheaderlength)
- [getPacketLength](modules.md#getpacketlength)
- [hasB58String](modules.md#hasb58string)
- [isExpired](modules.md#isexpired)
- [iterateHash](modules.md#iteratehash)
- [lengthPrefixedToU8a](modules.md#lengthprefixedtou8a)
- [libp2pSendMessage](modules.md#libp2psendmessage)
- [libp2pSendMessageAndExpectResponse](modules.md#libp2psendmessageandexpectresponse)
- [libp2pSubscribe](modules.md#libp2psubscribe)
- [limitConcurrency](modules.md#limitconcurrency)
- [moveDecimalPoint](modules.md#movedecimalpoint)
- [oneAtATime](modules.md#oneatatime)
- [parseHosts](modules.md#parsehosts)
- [parseJSON](modules.md#parsejson)
- [preVerify](modules.md#preverify)
- [privKeyToPeerId](modules.md#privkeytopeerid)
- [pubKeyToPeerId](modules.md#pubkeytopeerid)
- [randomChoice](modules.md#randomchoice)
- [randomFloat](modules.md#randomfloat)
- [randomInteger](modules.md#randominteger)
- [randomPermutation](modules.md#randompermutation)
- [randomSubset](modules.md#randomsubset)
- [recoverIteratedHash](modules.md#recoveriteratedhash)
- [sampleGroupElement](modules.md#samplegroupelement)
- [serializeToU8a](modules.md#serializetou8a)
- [stringToU8a](modules.md#stringtou8a)
- [timeoutAfter](modules.md#timeoutafter)
- [toLengthPrefixedU8a](modules.md#tolengthprefixedu8a)
- [toU8a](modules.md#tou8a)
- [u8aAdd](modules.md#u8aadd)
- [u8aAllocate](modules.md#u8aallocate)
- [u8aCompare](modules.md#u8acompare)
- [u8aConcat](modules.md#u8aconcat)
- [u8aEquals](modules.md#u8aequals)
- [u8aSplit](modules.md#u8asplit)
- [u8aToHex](modules.md#u8atohex)
- [u8aToNumber](modules.md#u8atonumber)
- [u8aXOR](modules.md#u8axor)
- [validateAcknowledgement](modules.md#validateacknowledgement)

## Type aliases

### ChannelStatus

Ƭ **ChannelStatus**: `"CLOSED"` \| `"OPEN"` \| `"PENDING_TO_CLOSE"`

Defined in: [types/channelEntry.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L6)

---

### DialOpts

Ƭ **DialOpts**: _object_

#### Type declaration

| Name      | Type     |
| :-------- | :------- |
| `timeout` | _number_ |

Defined in: [libp2p/index.ts:82](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L82)

---

### DialResponse

Ƭ **DialResponse**: { `resp`: { `protocol`: _string_ ; `stream`: MuxedStream } ; `status`: `"SUCCESS"` } \| { `status`: `"E_TIMEOUT"` } \| { `dht`: _boolean_ ; `error`: Error ; `status`: `"E_DIAL"` } \| { `error`: Error ; `query`: PeerId ; `status`: `"E_DHT_QUERY"` }

Defined in: [libp2p/index.ts:86](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L86)

---

### Hosts

Ƭ **Hosts**: _object_

#### Type declaration

| Name   | Type                                     |
| :----- | :--------------------------------------- |
| `ip4?` | [_NetOptions_](interfaces/netoptions.md) |
| `ip6?` | [_NetOptions_](interfaces/netoptions.md) |

Defined in: [hosts.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/hosts.ts#L6)

---

### LibP2PHandlerArgs

Ƭ **LibP2PHandlerArgs**: _object_

#### Type declaration

| Name         | Type        |
| :----------- | :---------- |
| `connection` | Connection  |
| `protocol`   | _string_    |
| `stream`     | MuxedStream |

Defined in: [libp2p/index.ts:231](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L231)

---

### LibP2PHandlerFunction

Ƭ **LibP2PHandlerFunction**: (`msg`: Uint8Array, `remotePeer`: PeerId) => _any_

#### Type declaration

▸ (`msg`: Uint8Array, `remotePeer`: PeerId): _any_

#### Parameters

| Name         | Type       |
| :----------- | :--------- |
| `msg`        | Uint8Array |
| `remotePeer` | PeerId     |

**Returns:** _any_

Defined in: [libp2p/index.ts:232](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L232)

---

### PRGParameters

Ƭ **PRGParameters**: _object_

#### Type declaration

| Name  | Type       |
| :---- | :--------- |
| `iv`  | Uint8Array |
| `key` | Uint8Array |

Defined in: [crypto/prg.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L11)

---

### PRPParameters

Ƭ **PRPParameters**: _object_

#### Type declaration

| Name  | Type       |
| :---- | :--------- |
| `iv`  | Uint8Array |
| `key` | Uint8Array |

Defined in: [crypto/prp.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L16)

---

### PromiseValue

Ƭ **PromiseValue**<T\>: T _extends_ _PromiseLike_<_infer_ U\> ? U : T

Infer the return value of a promise

#### Type parameters

| Name |
| :--- |
| `T`  |

Defined in: [typescript/index.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/typescript/index.ts#L4)

---

### U8aAndSize

Ƭ **U8aAndSize**: [Uint8Array, *number*]

Defined in: [u8a/index.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/index.ts#L20)

## Variables

### ADDRESS_LENGTH

• `Const` **ADDRESS_LENGTH**: `20`= 20

Defined in: [constants.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L4)

---

### A_EQUALS_B

• `Const` **A_EQUALS_B**: `0`= 0

Defined in: [u8a/u8aCompare.ts:2](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/u8aCompare.ts#L2)

---

### A_STRICLY_LESS_THAN_B

• `Const` **A_STRICLY_LESS_THAN_B**: `-1`= -1

Defined in: [u8a/u8aCompare.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/u8aCompare.ts#L1)

---

### A_STRICTLY_GREATER_THAN_B

• `Const` **A_STRICTLY_GREATER_THAN_B**: `1`= 1

Defined in: [u8a/u8aCompare.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/u8aCompare.ts#L3)

---

### HASH_LENGTH

• `Const` **HASH_LENGTH**: `32`= 32

Defined in: [constants.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L5)

---

### LENGTH_PREFIX_LENGTH

• `Const` **LENGTH_PREFIX_LENGTH**: `4`= 4

Defined in: [u8a/constants.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/constants.ts#L1)

---

### MULTI_ADDR_MAX_LENGTH

• `Const` **MULTI_ADDR_MAX_LENGTH**: `200`= 200

Defined in: [constants.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L9)

---

### POR_STRING_LENGTH

• `Const` **POR_STRING_LENGTH**: _number_

Defined in: [crypto/por/index.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/por/index.ts#L9)

---

### PRG_COUNTER_LENGTH

• `Const` **PRG_COUNTER_LENGTH**: `4`= 4

Defined in: [crypto/prg.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L7)

---

### PRG_IV_LENGTH

• `Const` **PRG_IV_LENGTH**: `12`= 12

Defined in: [crypto/prg.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L6)

---

### PRG_KEY_LENGTH

• `Const` **PRG_KEY_LENGTH**: `16`

Defined in: [crypto/prg.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prg.ts#L5)

---

### PRIVATE_KEY_LENGTH

• `Const` **PRIVATE_KEY_LENGTH**: `32`= 32

Defined in: [constants.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L1)

---

### PRP_IV_LENGTH

• `Const` **PRP_IV_LENGTH**: _number_

Defined in: [crypto/prp.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L13)

---

### PRP_KEY_LENGTH

• `Const` **PRP_KEY_LENGTH**: _number_

Defined in: [crypto/prp.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L12)

---

### PRP_MIN_LENGTH

• `Const` **PRP_MIN_LENGTH**: `32`

Defined in: [crypto/prp.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/prp.ts#L14)

---

### PUBLIC_KEY_LENGTH

• `Const` **PUBLIC_KEY_LENGTH**: `33`= 33

Defined in: [constants.ts:2](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L2)

---

### SECP256K1_CONSTANTS

• `Const` **SECP256K1_CONSTANTS**: _object_

Several ECDSA on secp256k1 related constants

#### Type declaration

| Name                             | Type     |
| :------------------------------- | :------- |
| `COMPRESSED_PUBLIC_KEY_LENGTH`   | _number_ |
| `PRIVATE_KEY_LENGTH`             | _number_ |
| `RECOVERABLE_SIGNATURE_LENGTH`   | _number_ |
| `SIGNATURE_LENGTH`               | _number_ |
| `UNCOMPRESSED_PUBLIC_KEY_LENGTH` | _number_ |

Defined in: [crypto/constants.ts:4](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/constants.ts#L4)

---

### SIGNATURE_LENGTH

• `Const` **SIGNATURE_LENGTH**: `64`= 64

Defined in: [constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L6)

---

### SIGNATURE_RECOVERY_LENGTH

• `Const` **SIGNATURE_RECOVERY_LENGTH**: `1`= 1

Defined in: [constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L7)

---

### UNCOMPRESSED_PUBLIC_KEY_LENGTH

• `Const` **UNCOMPRESSED_PUBLIC_KEY_LENGTH**: `66`= 66

Defined in: [constants.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/constants.ts#L3)

---

### b58StringRegex

• `Const` **b58StringRegex**: _RegExp_

Regular expresion used to match b58Strings

Defined in: [libp2p/index.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L22)

---

### durations

• `Const` **durations**: _object_

#### Type declaration

| Name      | Type                              |
| :-------- | :-------------------------------- |
| `days`    | (`days`: _number_) => _number_    |
| `hours`   | (`hours`: _number_) => _number_   |
| `minutes` | (`minutes`: _number_) => _number_ |
| `seconds` | (`seconds`: _number_) => _number_ |

Defined in: [time.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/time.ts#L1)

## Functions

### UnAcknowledgedTickets

▸ **UnAcknowledgedTickets**(`encodedAckChallenge`: [_Address_](classes/address.md)): Uint8Array

#### Parameters

| Name                  | Type                            |
| :-------------------- | :------------------------------ |
| `encodedAckChallenge` | [_Address_](classes/address.md) |

**Returns:** Uint8Array

Defined in: [db.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/db.ts#L44)

---

### cacheNoArgAsyncFunction

▸ **cacheNoArgAsyncFunction**<T\>(`func`: () => _Promise_<T\>, `expiry`: _number_): _function_

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name     | Type                |
| :------- | :------------------ |
| `func`   | () => _Promise_<T\> |
| `expiry` | _number_            |

**Returns:** () => _Promise_<T\>

Defined in: [cache.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/cache.ts#L3)

---

### convertPubKeyFromB58String

▸ **convertPubKeyFromB58String**(`b58string`: _string_): _Promise_<PublicKey\>

Takes a B58String and converts them to a PublicKey

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `b58string` | _string_ |

**Returns:** _Promise_<PublicKey\>

Defined in: [libp2p/index.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L39)

---

### convertPubKeyFromPeerId

▸ **convertPubKeyFromPeerId**(`peerId`: PeerId): _Promise_<PublicKey\>

Takes a peerId and returns its corresponding public key.

#### Parameters

| Name     | Type   | Description                              |
| :------- | :----- | :--------------------------------------- |
| `peerId` | PeerId | the PeerId used to generate a public key |

**Returns:** _Promise_<PublicKey\>

Defined in: [libp2p/index.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L29)

---

### createFirstChallenge

▸ **createFirstChallenge**(`secretB`: Uint8Array, `secretC?`: Uint8Array): _object_

Takes the secrets which the first and the second relayer are able
to derive from the packet header and computes the challenge for
the first ticket.

#### Parameters

| Name       | Type       | Description                |
| :--------- | :--------- | :------------------------- |
| `secretB`  | Uint8Array | shared secret with node +1 |
| `secretC?` | Uint8Array | shared secret with node +2 |

**Returns:** _object_

| Name              | Type                                |
| :---------------- | :---------------------------------- |
| `ackChallenge`    | [_PublicKey_](classes/publickey.md) |
| `ownKey`          | Uint8Array                          |
| `ticketChallenge` | [_PublicKey_](classes/publickey.md) |

the challenge for the first ticket sent to the first relayer

Defined in: [crypto/por/index.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/por/index.ts#L21)

---

### createPacket

▸ **createPacket**(`secrets`: Uint8Array[], `alpha`: Uint8Array, `msg`: Uint8Array, `path`: PeerId[], `maxHops`: _number_, `additionalDataRelayerLength`: _number_, `additionalDataRelayer`: Uint8Array[], `additionalDataLastHop?`: Uint8Array): Uint8Array

Creates a mixnet packet

#### Parameters

| Name                          | Type         | Description                                                    |
| :---------------------------- | :----------- | :------------------------------------------------------------- |
| `secrets`                     | Uint8Array[] | -                                                              |
| `alpha`                       | Uint8Array   | -                                                              |
| `msg`                         | Uint8Array   | payload to send                                                |
| `path`                        | PeerId[]     | nodes to use for relaying, including the final destination     |
| `maxHops`                     | _number_     | maximal number of hops to use                                  |
| `additionalDataRelayerLength` | _number_     | -                                                              |
| `additionalDataRelayer`       | Uint8Array[] | additional data to put next to each node's routing information |
| `additionalDataLastHop?`      | Uint8Array   | additional data for the final destination                      |

**Returns:** Uint8Array

the packet as u8a

Defined in: [crypto/packet/index.ts:65](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/packet/index.ts#L65)

---

### createPoRString

▸ **createPoRString**(`secretC`: Uint8Array, `secretD?`: Uint8Array): Uint8Array

Creates the bitstring containing the PoR challenge for the next
downstream node as well as the hint that is used to verify the
challenge that is given to the relayer.

#### Parameters

| Name       | Type       | Description                |
| :--------- | :--------- | :------------------------- |
| `secretC`  | Uint8Array | shared secret with node +2 |
| `secretD?` | Uint8Array | shared secret with node +3 |

**Returns:** Uint8Array

the bitstring that is embedded next to the routing
information for each relayer

Defined in: [crypto/por/index.ts:47](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/por/index.ts#L47)

---

### deriveAckKeyShare

▸ **deriveAckKeyShare**(`secret`: Uint8Array): _Uint8Array_

Comutes the key share that is embedded in the acknowledgement
for a packet and thereby unlocks the incentive for the previous
relayer for transforming and delivering the packet

#### Parameters

| Name     | Type       | Description                                  |
| :------- | :--------- | :------------------------------------------- |
| `secret` | Uint8Array | shared secret with the creator of the packet |

**Returns:** _Uint8Array_

Defined in: [crypto/por/keyDerivation.ts:30](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/por/keyDerivation.ts#L30)

---

### dial

▸ **dial**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: _string_, `opts?`: [_DialOpts_](modules.md#dialopts)): _Promise_<[_DialResponse_](modules.md#dialresponse)\>

Combines libp2p methods such as dialProtocol and peerRouting.findPeer
to establish a connection.
Contains a baseline protection against dialing same addresses twice.

#### Parameters

| Name          | Type                              | Description               |
| :------------ | :-------------------------------- | :------------------------ |
| `libp2p`      | LibP2P                            | a libp2p instance         |
| `destination` | PeerId                            | PeerId of the destination |
| `protocol`    | _string_                          | -                         |
| `opts?`       | [_DialOpts_](modules.md#dialopts) |                           |

**Returns:** _Promise_<[_DialResponse_](modules.md#dialresponse)\>

Defined in: [libp2p/index.ts:114](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L114)

---

### forwardTransform

▸ **forwardTransform**(`privKey`: PeerId, `packet`: Uint8Array, `additionalDataRelayerLength`: _number_, `additionalDataLastHopLength`: _number_, `maxHops`: _number_): LastNodeOutput \| RelayNodeOutput

Applies the transformation to the header to forward
it to the next node or deliver it to the user

#### Parameters

| Name                          | Type       | Description                                                            |
| :---------------------------- | :--------- | :--------------------------------------------------------------------- |
| `privKey`                     | PeerId     | private key of the relayer                                             |
| `packet`                      | Uint8Array | incoming packet as u8a                                                 |
| `additionalDataRelayerLength` | _number_   | length of the additional data next the routing information of each hop |
| `additionalDataLastHopLength` | _number_   | lenght of the additional data for the last hop                         |
| `maxHops`                     | _number_   | maximal amount of hops                                                 |

**Returns:** LastNodeOutput \| RelayNodeOutput

whether the packet is valid, if yes returns
the transformed packet, the public key of the next hop
and the data next to the routing information. If current
hop is the final recipient, it returns the plaintext

Defined in: [crypto/packet/index.ts:128](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/packet/index.ts#L128)

---

### gcd

▸ **gcd**(`a`: _number_, `b`: _number_): _number_

Computes the greatest common divisor of two integers

#### Parameters

| Name | Type     | Description   |
| :--- | :------- | :------------ |
| `a`  | _number_ | first number  |
| `b`  | _number_ | second number |

**Returns:** _number_

Defined in: [math/gcd.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/math/gcd.ts#L6)

---

### generateChannelId

▸ **generateChannelId**(`self`: [_Address_](classes/address.md), `counterparty`: [_Address_](classes/address.md)): [_Hash_](classes/hash.md)

#### Parameters

| Name           | Type                            |
| :------------- | :------------------------------ |
| `self`         | [_Address_](classes/address.md) |
| `counterparty` | [_Address_](classes/address.md) |

**Returns:** [_Hash_](classes/hash.md)

Defined in: [types/channelEntry.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/types/channelEntry.ts#L8)

---

### generateKeyShares

▸ **generateKeyShares**(`path`: PeerId[]): _object_

Performs an offline Diffie-Hellman key exchange with
the nodes along the given path

#### Parameters

| Name   | Type     | Description                           |
| :----- | :------- | :------------------------------------ |
| `path` | PeerId[] | the path to use for the mixnet packet |

**Returns:** _object_

| Name      | Type         |
| :-------- | :----------- |
| `alpha`   | Uint8Array   |
| `secrets` | Uint8Array[] |

the first group element and the shared secrets
with the nodes along the path

Defined in: [crypto/packet/keyShares.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/packet/keyShares.ts#L16)

---

### getB58String

▸ **getB58String**(`content`: _string_): _string_

Returns the b58String within a given content. Returns empty string if none is found.

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `content` | _string_ |

**Returns:** _string_

Defined in: [libp2p/index.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L66)

---

### getHeaderLength

▸ **getHeaderLength**(`maxHops`: _number_, `additionalDataRelayerLength`: _number_, `additionalDataLastHopLength`: _number_): _number_

#### Parameters

| Name                          | Type     |
| :---------------------------- | :------- |
| `maxHops`                     | _number_ |
| `additionalDataRelayerLength` | _number_ |
| `additionalDataLastHopLength` | _number_ |

**Returns:** _number_

Defined in: [crypto/packet/index.ts:28](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/packet/index.ts#L28)

---

### getPacketLength

▸ **getPacketLength**(`maxHops`: _number_, `additionalDataRelayerLength`: _number_, `additionalDataLastHopLength`: _number_): _number_

#### Parameters

| Name                          | Type     |
| :---------------------------- | :------- |
| `maxHops`                     | _number_ |
| `additionalDataRelayerLength` | _number_ |
| `additionalDataLastHopLength` | _number_ |

**Returns:** _number_

Defined in: [crypto/packet/index.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/packet/index.ts#L39)

---

### hasB58String

▸ **hasB58String**(`content`: _string_): Boolean

Returns true or false if given string does not contain a b58string

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `content` | _string_ |

**Returns:** Boolean

Defined in: [libp2p/index.ts:49](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L49)

---

### isExpired

▸ **isExpired**(`value`: _number_, `now`: _number_, `ttl`: _number_): _boolean_

Compares timestamps to find out if "value" has expired.

#### Parameters

| Name    | Type     | Description                               |
| :------ | :------- | :---------------------------------------- |
| `value` | _number_ | timestamp to compare with                 |
| `now`   | _number_ | timestamp example: `new Date().getTime()` |
| `ttl`   | _number_ | in milliseconds                           |

**Returns:** _boolean_

true if it's expired

Defined in: [time.ts:23](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/time.ts#L23)

---

### iterateHash

▸ **iterateHash**(`seed`: Uint8Array \| _undefined_, `hashFunc`: (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array, `iterations`: _number_, `stepSize`: _number_, `hint?`: (`index`: _number_) => Uint8Array \| _undefined_ \| _Promise_<Uint8Array \| undefined\>): _Promise_<{ `hash`: Uint8Array ; `intermediates`: [_Intermediate_](interfaces/intermediate.md)[] }\>

#### Parameters

| Name         | Type                                                                                    |
| :----------- | :-------------------------------------------------------------------------------------- |
| `seed`       | Uint8Array \| _undefined_                                                               |
| `hashFunc`   | (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array                        |
| `iterations` | _number_                                                                                |
| `stepSize`   | _number_                                                                                |
| `hint?`      | (`index`: _number_) => Uint8Array \| _undefined_ \| _Promise_<Uint8Array \| undefined\> |

**Returns:** _Promise_<{ `hash`: Uint8Array ; `intermediates`: [_Intermediate_](interfaces/intermediate.md)[] }\>

Defined in: [crypto/hashIterator.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/hashIterator.ts#L7)

---

### lengthPrefixedToU8a

▸ **lengthPrefixedToU8a**(`arg`: Uint8Array, `additionalPadding?`: Uint8Array, `targetLength?`: _number_): _Uint8Array_

Decodes a length-prefixed array and returns the encoded data.

#### Parameters

| Name                 | Type       | Description                  |
| :------------------- | :--------- | :--------------------------- |
| `arg`                | Uint8Array | array to decode              |
| `additionalPadding?` | Uint8Array | additional padding to remove |
| `targetLength?`      | _number_   | optional target length       |

**Returns:** _Uint8Array_

Defined in: [u8a/lengthPrefixedToU8a.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/lengthPrefixedToU8a.ts#L11)

---

### libp2pSendMessage

▸ **libp2pSendMessage**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: _string_, `message`: Uint8Array, `opts?`: [_DialOpts_](modules.md#dialopts)): _Promise_<void\>

#### Parameters

| Name          | Type                              |
| :------------ | :-------------------------------- |
| `libp2p`      | LibP2P                            |
| `destination` | PeerId                            |
| `protocol`    | _string_                          |
| `message`     | Uint8Array                        |
| `opts?`       | [_DialOpts_](modules.md#dialopts) |

**Returns:** _Promise_<void\>

Defined in: [libp2p/index.ts:195](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L195)

---

### libp2pSendMessageAndExpectResponse

▸ **libp2pSendMessageAndExpectResponse**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: _string_, `message`: Uint8Array, `opts?`: [_DialOpts_](modules.md#dialopts)): _Promise_<Uint8Array\>

#### Parameters

| Name          | Type                              |
| :------------ | :-------------------------------- |
| `libp2p`      | LibP2P                            |
| `destination` | PeerId                            |
| `protocol`    | _string_                          |
| `message`     | Uint8Array                        |
| `opts?`       | [_DialOpts_](modules.md#dialopts) |

**Returns:** _Promise_<Uint8Array\>

Defined in: [libp2p/index.ts:211](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L211)

---

### libp2pSubscribe

▸ **libp2pSubscribe**(`libp2p`: LibP2P, `protocol`: _string_, `handler`: [_LibP2PHandlerFunction_](modules.md#libp2phandlerfunction), `includeReply?`: _boolean_): _void_

#### Parameters

| Name           | Type                                                        | Default value |
| :------------- | :---------------------------------------------------------- | :------------ |
| `libp2p`       | LibP2P                                                      | -             |
| `protocol`     | _string_                                                    | -             |
| `handler`      | [_LibP2PHandlerFunction_](modules.md#libp2phandlerfunction) | -             |
| `includeReply` | _boolean_                                                   | false         |

**Returns:** _void_

Defined in: [libp2p/index.ts:251](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/index.ts#L251)

---

### limitConcurrency

▸ **limitConcurrency**<T\>(`maxConcurrency`: _number_, `exitCond`: () => _boolean_, `createPromise`: () => _Promise_<T\>, `maxIterations?`: _number_): _Promise_<T[]\>

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name             | Type                | Default value |
| :--------------- | :------------------ | :------------ |
| `maxConcurrency` | _number_            | -             |
| `exitCond`       | () => _boolean_     | -             |
| `createPromise`  | () => _Promise_<T\> | -             |
| `maxIterations`  | _number_            | 1e3           |

**Returns:** _Promise_<T[]\>

Defined in: [collection/promise-pool.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/promise-pool.ts#L1)

---

### moveDecimalPoint

▸ **moveDecimalPoint**(`amount`: BigNumber \| _string_ \| _number_, `position`: _number_): _string_

#### Parameters

| Name       | Type                              |
| :--------- | :-------------------------------- |
| `amount`   | BigNumber \| _string_ \| _number_ |
| `position` | _number_                          |

**Returns:** _string_

Defined in: [math/moveDecimalPoint.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/math/moveDecimalPoint.ts#L3)

---

### oneAtATime

▸ **oneAtATime**(): _function_

**Returns:** (`cb`: () => _Promise_<void\>) => _Promise_<void\>

Defined in: [concurrency.ts:1](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/concurrency.ts#L1)

---

### parseHosts

▸ **parseHosts**(): [_Hosts_](modules.md#hosts)

**Returns:** [_Hosts_](modules.md#hosts)

Defined in: [hosts.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/hosts.ts#L11)

---

### parseJSON

▸ **parseJSON**(`str`: _string_): _object_

Parse JSON while recovering all Buffer elements

#### Parameters

| Name  | Type     | Description |
| :---- | :------- | :---------- |
| `str` | _string_ | JSON string |

**Returns:** _object_

Defined in: [parseJSON.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/parseJSON.ts#L5)

---

### preVerify

▸ **preVerify**(`secret`: Uint8Array, `porBytes`: Uint8Array, `challenge`: [_Address_](classes/address.md)): ValidOutput \| InvalidOutput

Verifies that an incoming packet contains all values that
are necessary to reconstruct the response to redeem the
incentive for relaying the packet

#### Parameters

| Name        | Type                            | Description                                  |
| :---------- | :------------------------------ | :------------------------------------------- |
| `secret`    | Uint8Array                      | shared secret with the creator of the packet |
| `porBytes`  | Uint8Array                      | PoR bitstring as included within the packet  |
| `challenge` | [_Address_](classes/address.md) | ticket challenge of the incoming ticket      |

**Returns:** ValidOutput \| InvalidOutput

whether the challenge is derivable, if yes, it returns
the keyShare of the relayer as well as the secret that is used
to create it and the challenge for the next relayer.

Defined in: [crypto/por/index.ts:83](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/por/index.ts#L83)

---

### privKeyToPeerId

▸ **privKeyToPeerId**(`privKey`: Uint8Array \| _string_): PeerId

Converts a plain compressed ECDSA private key over the curve `secp256k1`
to a peerId in order to use it with libp2p.
It equips the generated peerId with private key and public key.

#### Parameters

| Name      | Type                   | Description           |
| :-------- | :--------------------- | :-------------------- |
| `privKey` | Uint8Array \| _string_ | the plain private key |

**Returns:** PeerId

Defined in: [libp2p/privKeyToPeerId.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/privKeyToPeerId.ts#L18)

---

### pubKeyToPeerId

▸ **pubKeyToPeerId**(`pubKey`: Uint8Array \| _string_): PeerId

Converts a plain compressed ECDSA public key over the curve `secp256k1`
to a peerId in order to use it with libp2p.

**`notice`** Libp2p stores the keys in format that is derived from `protobuf`.
Using `libsecp256k1` directly does not work.

#### Parameters

| Name     | Type                   | Description          |
| :------- | :--------------------- | :------------------- |
| `pubKey` | Uint8Array \| _string_ | the plain public key |

**Returns:** PeerId

Defined in: [libp2p/pubKeyToPeerId.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/libp2p/pubKeyToPeerId.ts#L17)

---

### randomChoice

▸ **randomChoice**<T\>(`collection`: T[]): T

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name         | Type |
| :----------- | :--- |
| `collection` | T[]  |

**Returns:** T

Defined in: [crypto/randomInteger.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/randomInteger.ts#L85)

---

### randomFloat

▸ **randomFloat**(): _number_

**Returns:** _number_

Defined in: [crypto/randomFloat.ts:3](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/randomFloat.ts#L3)

---

### randomInteger

▸ **randomInteger**(`start`: _number_, `end?`: _number_, `_seed?`: Uint8Array): _number_

Returns a random value between `start` and `end`.

**`example`**

```
randomInteger(3) // result in { 0, 1, 2, 3 }
randomInteger(0, 3) // result in { 0, 1, 2, 3 }
randomInteger(7, 9) // result in { 7, 8, 9 }
randomInteger(8, 8) == 8
```

#### Parameters

| Name     | Type       | Description                   |
| :------- | :--------- | :---------------------------- |
| `start`  | _number_   | start of the interval         |
| `end?`   | _number_   | end of the interval inclusive |
| `_seed?` | Uint8Array | -                             |

**Returns:** _number_

random number between @param start and @param end

Defined in: [crypto/randomInteger.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/randomInteger.ts#L18)

---

### randomPermutation

▸ **randomPermutation**<T\>(`array`: T[]): T[]

Return a random permutation of the given `array`
by using the (optimized) Fisher-Yates shuffling algorithm.

**`example`**

```javascript
randomPermutation([1, 2, 3, 4])
// first run: [2,4,1,2]
// second run: [3,1,2,4]
// ...
```

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name    | Type | Description            |
| :------ | :--- | :--------------------- |
| `array` | T[]  | the array to permutate |

**Returns:** T[]

Defined in: [collection/randomPermutation.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/randomPermutation.ts#L18)

---

### randomSubset

▸ **randomSubset**<T\>(`array`: T[], `subsetSize`: _number_, `filter?`: (`candidate`: T) => _boolean_): T[]

Picks @param subsetSize elements at random from @param array .
The order of the picked elements does not coincide with their
order in @param array

**`notice`** If less than @param subsetSize elements pass the test,
the result will contain less than @param subsetSize elements.

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name         | Type                          | Description                                                                                   |
| :----------- | :---------------------------- | :-------------------------------------------------------------------------------------------- |
| `array`      | T[]                           | the array to pick the elements from                                                           |
| `subsetSize` | _number_                      | the requested size of the subset                                                              |
| `filter?`    | (`candidate`: T) => _boolean_ | called with `(peerInfo)` and should return `true` for every node that should be in the subset |

**Returns:** T[]

array with at most @param subsetSize elements
that pass the test.

Defined in: [collection/randomSubset.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/collection/randomSubset.ts#L20)

---

### recoverIteratedHash

▸ **recoverIteratedHash**(`hashValue`: Uint8Array, `hashFunc`: (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array, `hint`: (`index`: _number_) => _Promise_<Uint8Array\>, `maxIterations`: _number_, `stepSize?`: _number_, `indexHint?`: _number_): _Promise_<[_Intermediate_](interfaces/intermediate.md) \| undefined\>

#### Parameters

| Name            | Type                                                             |
| :-------------- | :--------------------------------------------------------------- |
| `hashValue`     | Uint8Array                                                       |
| `hashFunc`      | (`preImage`: Uint8Array) => _Promise_<Uint8Array\> \| Uint8Array |
| `hint`          | (`index`: _number_) => _Promise_<Uint8Array\>                    |
| `maxIterations` | _number_                                                         |
| `stepSize?`     | _number_                                                         |
| `indexHint?`    | _number_                                                         |

**Returns:** _Promise_<[_Intermediate_](interfaces/intermediate.md) \| undefined\>

Defined in: [crypto/hashIterator.ts:55](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/hashIterator.ts#L55)

---

### sampleGroupElement

▸ **sampleGroupElement**(`compressed?`: _boolean_): [exponent: Uint8Array, groupElement: Uint8Array]

Samples a valid exponent and returns the exponent
and the product of exponent and base-point.

**`dev`** can be used to derive a secp256k1 keypair

#### Parameters

| Name         | Type      | Default value |
| :----------- | :-------- | :------------ |
| `compressed` | _boolean_ | false         |

**Returns:** [exponent: Uint8Array, groupElement: Uint8Array]

Defined in: [crypto/sampleGroupElement.ts:11](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/sampleGroupElement.ts#L11)

---

### serializeToU8a

▸ **serializeToU8a**(`items`: [_U8aAndSize_](modules.md#u8aandsize)[]): Uint8Array

#### Parameters

| Name    | Type                                    |
| :------ | :-------------------------------------- |
| `items` | [_U8aAndSize_](modules.md#u8aandsize)[] |

**Returns:** Uint8Array

Defined in: [u8a/index.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/index.ts#L22)

---

### stringToU8a

▸ **stringToU8a**(`str`: _string_, `length?`: _number_): Uint8Array

Converts a **HEX** string to a Uint8Array and optionally adds some padding to match
the desired size.

**`example`**
stringToU8a('0xDEadBeeF') // Uint8Array [ 222, 173, 190, 239 ]

**`notice`** Throws an error in case a length was provided and the result does not fit.

#### Parameters

| Name      | Type     | Description                      |
| :-------- | :------- | :------------------------------- |
| `str`     | _string_ | string to convert                |
| `length?` | _number_ | desired length of the Uint8Array |

**Returns:** Uint8Array

Defined in: [u8a/toU8a.ts:60](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/toU8a.ts#L60)

---

### timeoutAfter

▸ **timeoutAfter**<T\>(`body`: (`abortSignal`: AbortSignal) => _Promise_<T\>, `timeout`: _number_): _Promise_<T\>

#### Type parameters

| Name |
| :--- |
| `T`  |

#### Parameters

| Name      | Type                                          |
| :-------- | :-------------------------------------------- |
| `body`    | (`abortSignal`: AbortSignal) => _Promise_<T\> |
| `timeout` | _number_                                      |

**Returns:** _Promise_<T\>

Defined in: [timeout.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/timeout.ts#L5)

---

### toLengthPrefixedU8a

▸ **toLengthPrefixedU8a**(`arg`: Uint8Array, `additionalPadding?`: Uint8Array, `length?`: _number_): _Uint8Array_

Adds a length-prefix to a Uint8Array

#### Parameters

| Name                 | Type       | Description                                                          |
| :------------------- | :--------- | :------------------------------------------------------------------- |
| `arg`                | Uint8Array | data to add padding                                                  |
| `additionalPadding?` | Uint8Array | optional additional padding that is inserted between length and data |
| `length?`            | _number_   | optional target length                                               |

**Returns:** _Uint8Array_

Defined in: [u8a/toLengthPrefixedU8a.ts:12](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/toLengthPrefixedU8a.ts#L12)

---

### toU8a

▸ **toU8a**(`arg`: _number_, `length?`: _number_): Uint8Array

Converts a number to a Uint8Array and optionally adds some padding to match
the desired size.

#### Parameters

| Name      | Type     | Description                      |
| :-------- | :------- | :------------------------------- |
| `arg`     | _number_ | to convert to Uint8Array         |
| `length?` | _number_ | desired length of the Uint8Array |

**Returns:** Uint8Array

Defined in: [u8a/toU8a.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/toU8a.ts#L7)

---

### u8aAdd

▸ **u8aAdd**(`inplace`: _boolean_, `a`: Uint8Array, `b`: Uint8Array): Uint8Array

Adds the contents of two arrays together while ignoring the final overflow.
Computes `a + b % ( 2 ** (8 * a.length) - 1)`

**`example`**
u8aAdd(false, new Uint8Array([1], new Uint8Array([2])) // Uint8Array([3])
u8aAdd(false, new Uint8Array([1], new Uint8Array([255])) // Uint8Array([0])
u8aAdd(false, new Uint8Array([0, 1], new Uint8Array([0, 255])) // Uint8Array([1, 0])

#### Parameters

| Name      | Type       | Description                          |
| :-------- | :--------- | :----------------------------------- |
| `inplace` | _boolean_  | result is stored in a if set to true |
| `a`       | Uint8Array | first array                          |
| `b`       | Uint8Array | second array                         |

**Returns:** Uint8Array

Defined in: [u8a/u8aAdd.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/u8aAdd.ts#L13)

---

### u8aAllocate

▸ **u8aAllocate**(`__namedParameters`: MemoryPage, ...`list`: (Uint8Array \| _undefined_)[]): Uint8Array

Writes to the provided mempage the data on a given list of u8a on a given offset

**`export`**

#### Parameters

| Name                | Type                          |
| :------------------ | :---------------------------- |
| `__namedParameters` | MemoryPage                    |
| `...list`           | (Uint8Array \| _undefined_)[] |

**Returns:** Uint8Array

Defined in: [u8a/allocate.ts:14](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/allocate.ts#L14)

---

### u8aCompare

▸ **u8aCompare**(`a`: Uint8Array, `b`: Uint8Array): _number_

#### Parameters

| Name | Type       |
| :--- | :--------- |
| `a`  | Uint8Array |
| `b`  | Uint8Array |

**Returns:** _number_

Defined in: [u8a/u8aCompare.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/u8aCompare.ts#L5)

---

### u8aConcat

▸ **u8aConcat**(...`list`: (Uint8Array \| _undefined_)[]): Uint8Array

Concatenates the input arrays into a single `UInt8Array`.

**`example`**
u8aConcat(
new Uint8Array([1, 1, 1]),
new Uint8Array([2, 2, 2])
); // Uint8Arrau([1, 1, 1, 2, 2, 2])

- u8aConcat(
  new Uint8Array([1, 1, 1]),
  undefined
  new Uint8Array([2, 2, 2])
  ); // Uint8Arrau([1, 1, 1, 2, 2, 2])

#### Parameters

| Name      | Type                          |
| :-------- | :---------------------------- |
| `...list` | (Uint8Array \| _undefined_)[] |

**Returns:** Uint8Array

Defined in: [u8a/concat.ts:15](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/concat.ts#L15)

---

### u8aEquals

▸ **u8aEquals**(`a`: Uint8Array, `b`: Uint8Array, ...`arrays`: Uint8Array[]): _boolean_

Checks if the contents of the given Uint8Arrays are equal. Returns once at least
one different entry is found.

#### Parameters

| Name        | Type         | Description       |
| :---------- | :----------- | :---------------- |
| `a`         | Uint8Array   | first array       |
| `b`         | Uint8Array   | second array      |
| `...arrays` | Uint8Array[] | additional arrays |

**Returns:** _boolean_

Defined in: [u8a/equals.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/equals.ts#L8)

---

### u8aSplit

▸ **u8aSplit**(`u8a`: Uint8Array, `sizes`: _number_[]): Uint8Array[]

#### Parameters

| Name    | Type       |
| :------ | :--------- |
| `u8a`   | Uint8Array |
| `sizes` | _number_[] |

**Returns:** Uint8Array[]

Defined in: [u8a/index.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/index.ts#L36)

---

### u8aToHex

▸ **u8aToHex**(`arr?`: Uint8Array, `prefixed?`: _boolean_): _string_

Converts a Uint8Array to a hex string.

**`notice`** Mainly used for debugging.

#### Parameters

| Name       | Type       | Default value | Description                           |
| :--------- | :--------- | :------------ | :------------------------------------ |
| `arr?`     | Uint8Array | -             | Uint8Array                            |
| `prefixed` | _boolean_  | true          | if `true` add a `0x` in the beginning |

**Returns:** _string_

Defined in: [u8a/toHex.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/toHex.ts#L8)

---

### u8aToNumber

▸ **u8aToNumber**(`arr`: Uint8Array): _number_ \| _bigint_

Converts a Uint8Array to number.

#### Parameters

| Name  | Type       | Description                     |
| :---- | :--------- | :------------------------------ |
| `arr` | Uint8Array | Uint8Array to convert to number |

**Returns:** _number_ \| _bigint_

Defined in: [u8a/u8aToNumber.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/u8aToNumber.ts#L5)

---

### u8aXOR

▸ **u8aXOR**(`inPlace?`: _boolean_, ...`list`: Uint8Array[]): Uint8Array

Apply an XOR on a list of arrays.

#### Parameters

| Name      | Type         | Default value | Description                                 |
| :-------- | :----------- | :------------ | :------------------------------------------ |
| `inPlace` | _boolean_    | false         | if `true` overwrite first Array with result |
| `...list` | Uint8Array[] | -             | arrays to XOR                               |

**Returns:** Uint8Array

Defined in: [u8a/xor.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/u8a/xor.ts#L7)

---

### validateAcknowledgement

▸ **validateAcknowledgement**(`ownKey`: Uint8Array \| _undefined_, `ack`: Uint8Array \| _undefined_, `challenge`: [_Address_](classes/address.md), `ownShare?`: Uint8Array, `response?`: Uint8Array): { `response`: Uint8Array ; `valid`: `true` } \| { `valid`: `false` }

Takes an the second key share and reconstructs the secret
that is necessary to redeem the incentive for relaying the
packet.

#### Parameters

| Name        | Type                            | Description                                                               |
| :---------- | :------------------------------ | :------------------------------------------------------------------------ |
| `ownKey`    | Uint8Array \| _undefined_       | key that as derived from the shared secret with the creator of the packet |
| `ack`       | Uint8Array \| _undefined_       | second key share as given by the acknowledgement                          |
| `challenge` | [_Address_](classes/address.md) | challenge of the ticket                                                   |
| `ownShare?` | Uint8Array                      | own key share as computed from the packet                                 |
| `response?` | Uint8Array                      | -                                                                         |

**Returns:** { `response`: Uint8Array ; `valid`: `true` } \| { `valid`: `false` }

whether the input values led to a valid response that
can be used to redeem the incentive

Defined in: [crypto/por/index.ts:123](https://github.com/hoprnet/hoprnet/blob/master/packages/utils/src/crypto/por/index.ts#L123)
