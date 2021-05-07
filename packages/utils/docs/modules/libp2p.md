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

___

### pubKeyToPeerId

Re-exports: [pubKeyToPeerId](libp2p_pubkeytopeerid.md#pubkeytopeerid)

## Type aliases

### DialOpts

Ƭ **DialOpts**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `timeout` | *number* |

Defined in: [libp2p/index.ts:82](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L82)

___

### DialResponse

Ƭ **DialResponse**: { `resp`: { `protocol`: *string* ; `stream`: MuxedStream  } ; `status`: ``"SUCCESS"``  } \| { `status`: ``"E_TIMEOUT"``  } \| { `dht`: *boolean* ; `error`: Error ; `status`: ``"E_DIAL"``  } \| { `error`: Error ; `query`: PeerId ; `status`: ``"E_DHT_QUERY"``  }

Defined in: [libp2p/index.ts:86](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L86)

___

### LibP2PHandlerArgs

Ƭ **LibP2PHandlerArgs**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `connection` | Connection |
| `protocol` | *string* |
| `stream` | MuxedStream |

Defined in: [libp2p/index.ts:231](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L231)

___

### LibP2PHandlerFunction

Ƭ **LibP2PHandlerFunction**: (`msg`: Uint8Array, `remotePeer`: PeerId) => *any*

#### Type declaration

▸ (`msg`: Uint8Array, `remotePeer`: PeerId): *any*

#### Parameters

| Name | Type |
| :------ | :------ |
| `msg` | Uint8Array |
| `remotePeer` | PeerId |

**Returns:** *any*

Defined in: [libp2p/index.ts:232](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L232)

## Variables

### b58StringRegex

• `Const` **b58StringRegex**: *RegExp*

Regular expresion used to match b58Strings

Defined in: [libp2p/index.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L22)

## Functions

### convertPubKeyFromB58String

▸ **convertPubKeyFromB58String**(`b58string`: *string*): *Promise*<PublicKey\>

Takes a B58String and converts them to a PublicKey

#### Parameters

| Name | Type |
| :------ | :------ |
| `b58string` | *string* |

**Returns:** *Promise*<PublicKey\>

Defined in: [libp2p/index.ts:39](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L39)

___

### convertPubKeyFromPeerId

▸ **convertPubKeyFromPeerId**(`peerId`: PeerId): *Promise*<PublicKey\>

Takes a peerId and returns its corresponding public key.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `peerId` | PeerId | the PeerId used to generate a public key |

**Returns:** *Promise*<PublicKey\>

Defined in: [libp2p/index.ts:29](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L29)

___

### dial

▸ **dial**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: *string*, `opts?`: [*DialOpts*](libp2p.md#dialopts)): *Promise*<[*DialResponse*](libp2p.md#dialresponse)\>

Combines libp2p methods such as dialProtocol and peerRouting.findPeer
to establish a connection.
Contains a baseline protection against dialing same addresses twice.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `libp2p` | LibP2P | a libp2p instance |
| `destination` | PeerId | PeerId of the destination |
| `protocol` | *string* | - |
| `opts?` | [*DialOpts*](libp2p.md#dialopts) |  |

**Returns:** *Promise*<[*DialResponse*](libp2p.md#dialresponse)\>

Defined in: [libp2p/index.ts:114](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L114)

___

### getB58String

▸ **getB58String**(`content`: *string*): *string*

Returns the b58String within a given content. Returns empty string if none is found.

#### Parameters

| Name | Type |
| :------ | :------ |
| `content` | *string* |

**Returns:** *string*

Defined in: [libp2p/index.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L66)

___

### hasB58String

▸ **hasB58String**(`content`: *string*): Boolean

Returns true or false if given string does not contain a b58string

#### Parameters

| Name | Type |
| :------ | :------ |
| `content` | *string* |

**Returns:** Boolean

Defined in: [libp2p/index.ts:49](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L49)

___

### libp2pSendMessage

▸ **libp2pSendMessage**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: *string*, `message`: Uint8Array, `opts?`: [*DialOpts*](libp2p.md#dialopts)): *Promise*<void\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `libp2p` | LibP2P |
| `destination` | PeerId |
| `protocol` | *string* |
| `message` | Uint8Array |
| `opts?` | [*DialOpts*](libp2p.md#dialopts) |

**Returns:** *Promise*<void\>

Defined in: [libp2p/index.ts:195](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L195)

___

### libp2pSendMessageAndExpectResponse

▸ **libp2pSendMessageAndExpectResponse**(`libp2p`: LibP2P, `destination`: PeerId, `protocol`: *string*, `message`: Uint8Array, `opts?`: [*DialOpts*](libp2p.md#dialopts)): *Promise*<Uint8Array\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `libp2p` | LibP2P |
| `destination` | PeerId |
| `protocol` | *string* |
| `message` | Uint8Array |
| `opts?` | [*DialOpts*](libp2p.md#dialopts) |

**Returns:** *Promise*<Uint8Array\>

Defined in: [libp2p/index.ts:211](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L211)

___

### libp2pSubscribe

▸ **libp2pSubscribe**(`libp2p`: LibP2P, `protocol`: *string*, `handler`: [*LibP2PHandlerFunction*](libp2p.md#libp2phandlerfunction), `includeReply?`: *boolean*): *void*

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `libp2p` | LibP2P | - |
| `protocol` | *string* | - |
| `handler` | [*LibP2PHandlerFunction*](libp2p.md#libp2phandlerfunction) | - |
| `includeReply` | *boolean* | false |

**Returns:** *void*

Defined in: [libp2p/index.ts:251](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/utils/src/libp2p/index.ts#L251)
