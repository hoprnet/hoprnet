[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / libp2p

# Module: libp2p

## Table of contents

### References

- [privKeyToPeerId](libp2p.md#privkeytopeerid)
- [pubKeyToPeerId](libp2p.md#pubkeytopeerid)

### Type aliases

- [DialOpts](libp2p.md#dialopts)
- [DialResponse](libp2p.md#dialresponse)
- [LibP2PHandlerArgs](libp2p.md#libp2phandlerargs)
- [LibP2PHandlerFunction](libp2p.md#libp2phandlerfunction)

### Variables

- [b58StringRegex](libp2p.md#b58stringregex)

### Functions

- [convertPubKeyFromB58String](libp2p.md#convertpubkeyfromb58string)
- [convertPubKeyFromPeerId](libp2p.md#convertpubkeyfrompeerid)
- [dial](libp2p.md#dial)
- [getB58String](libp2p.md#getb58string)
- [hasB58String](libp2p.md#hasb58string)
- [libp2pSendMessage](libp2p.md#libp2psendmessage)
- [libp2pSendMessageAndExpectResponse](libp2p.md#libp2psendmessageandexpectresponse)
- [libp2pSubscribe](libp2p.md#libp2psubscribe)

## References

### privKeyToPeerId

Re-exports: [privKeyToPeerId](libp2p_privkeytopeerid.md#privkeytopeerid)

---

### pubKeyToPeerId

Re-exports: [pubKeyToPeerId](libp2p_pubkeytopeerid.md#pubkeytopeerid)

## Type aliases

### DialOpts

Ƭ **DialOpts**: _object_

#### Type declaration

| Name      | Type     |
| :-------- | :------- |
| `timeout` | _number_ |

Defined in: [libp2p/index.ts:82](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L82)

---

### DialResponse

Ƭ **DialResponse**: { `resp`: { `protocol`: _string_ ; `stream`: MuxedStream } ; `status`: `"SUCCESS"` } \| { `status`: `"E_TIMEOUT"` } \| { `dht`: _boolean_ ; `error`: Error ; `status`: `"E_DIAL"` } \| { `error`: Error ; `query`: PeerId ; `status`: `"E_DHT_QUERY"` }

Defined in: [libp2p/index.ts:86](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L86)

---

### LibP2PHandlerArgs

Ƭ **LibP2PHandlerArgs**: _object_

#### Type declaration

| Name         | Type        |
| :----------- | :---------- |
| `connection` | Connection  |
| `protocol`   | _string_    |
| `stream`     | MuxedStream |

Defined in: [libp2p/index.ts:231](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L231)

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

Defined in: [libp2p/index.ts:232](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L232)

## Variables

### b58StringRegex

• `Const` **b58StringRegex**: _RegExp_

Regular expresion used to match b58Strings

Defined in: [libp2p/index.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L22)

## Functions

### convertPubKeyFromB58String

▸ **convertPubKeyFromB58String**(`b58string`: _string_): _Promise_<PublicKey\>

Takes a B58String and converts them to a PublicKey

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `b58string` | _string_ |

**Returns:** _Promise_<PublicKey\>

Defined in: [libp2p/index.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L39)

---

### convertPubKeyFromPeerId

▸ **convertPubKeyFromPeerId**(`peerId`: PeerId): _Promise_<PublicKey\>

Takes a peerId and returns its corresponding public key.

#### Parameters

| Name     | Type   | Description                              |
| :------- | :----- | :--------------------------------------- |
| `peerId` | PeerId | the PeerId used to generate a public key |

**Returns:** _Promise_<PublicKey\>

Defined in: [libp2p/index.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L29)

---

### dial

▸ **dial**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: _string_, `opts?`: [_DialOpts_](libp2p.md#dialopts)): _Promise_<[_DialResponse_](libp2p.md#dialresponse)\>

Combines libp2p methods such as dialProtocol and peerRouting.findPeer
to establish a connection.
Contains a baseline protection against dialing same addresses twice.

#### Parameters

| Name          | Type                             | Description               |
| :------------ | :------------------------------- | :------------------------ |
| `libp2p`      | LibP2P                           | a libp2p instance         |
| `destination` | PeerId                           | PeerId of the destination |
| `protocol`    | _string_                         | -                         |
| `opts?`       | [_DialOpts_](libp2p.md#dialopts) |                           |

**Returns:** _Promise_<[_DialResponse_](libp2p.md#dialresponse)\>

Defined in: [libp2p/index.ts:114](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L114)

---

### getB58String

▸ **getB58String**(`content`: _string_): _string_

Returns the b58String within a given content. Returns empty string if none is found.

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `content` | _string_ |

**Returns:** _string_

Defined in: [libp2p/index.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L66)

---

### hasB58String

▸ **hasB58String**(`content`: _string_): Boolean

Returns true or false if given string does not contain a b58string

#### Parameters

| Name      | Type     |
| :-------- | :------- |
| `content` | _string_ |

**Returns:** Boolean

Defined in: [libp2p/index.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L49)

---

### libp2pSendMessage

▸ **libp2pSendMessage**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: _string_, `message`: Uint8Array, `opts?`: [_DialOpts_](libp2p.md#dialopts)): _Promise_<void\>

#### Parameters

| Name          | Type                             |
| :------------ | :------------------------------- |
| `libp2p`      | LibP2P                           |
| `destination` | PeerId                           |
| `protocol`    | _string_                         |
| `message`     | Uint8Array                       |
| `opts?`       | [_DialOpts_](libp2p.md#dialopts) |

**Returns:** _Promise_<void\>

Defined in: [libp2p/index.ts:195](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L195)

---

### libp2pSendMessageAndExpectResponse

▸ **libp2pSendMessageAndExpectResponse**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: _string_, `message`: Uint8Array, `opts?`: [_DialOpts_](libp2p.md#dialopts)): _Promise_<Uint8Array\>

#### Parameters

| Name          | Type                             |
| :------------ | :------------------------------- |
| `libp2p`      | LibP2P                           |
| `destination` | PeerId                           |
| `protocol`    | _string_                         |
| `message`     | Uint8Array                       |
| `opts?`       | [_DialOpts_](libp2p.md#dialopts) |

**Returns:** _Promise_<Uint8Array\>

Defined in: [libp2p/index.ts:211](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L211)

---

### libp2pSubscribe

▸ **libp2pSubscribe**(`libp2p`: LibP2P, `protocol`: _string_, `handler`: [_LibP2PHandlerFunction_](libp2p.md#libp2phandlerfunction), `includeReply?`: _boolean_): _void_

#### Parameters

| Name           | Type                                                       | Default value |
| :------------- | :--------------------------------------------------------- | :------------ |
| `libp2p`       | LibP2P                                                     | -             |
| `protocol`     | _string_                                                   | -             |
| `handler`      | [_LibP2PHandlerFunction_](libp2p.md#libp2phandlerfunction) | -             |
| `includeReply` | _boolean_                                                  | false         |

**Returns:** _void_

Defined in: [libp2p/index.ts:251](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L251)
