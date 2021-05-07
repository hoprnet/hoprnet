[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / index

# Module: index

## Table of contents

### References

- [ADDRESS\_LENGTH](index.md#address_length)
- [A\_EQUALS\_B](index.md#a_equals_b)
- [A\_STRICLY\_LESS\_THAN\_B](index.md#a_stricly_less_than_b)
- [A\_STRICTLY\_GREATER\_THAN\_B](index.md#a_strictly_greater_than_b)
- [AccountEntry](index.md#accountentry)
- [AcknowledgedTicket](index.md#acknowledgedticket)
- [Address](index.md#address)
- [Balance](index.md#balance)
- [ChannelEntry](index.md#channelentry)
- [ChannelStatus](index.md#channelstatus)
- [DialOpts](index.md#dialopts)
- [DialResponse](index.md#dialresponse)
- [HASH\_LENGTH](index.md#hash_length)
- [Hash](index.md#hash)
- [HoprDB](index.md#hoprdb)
- [Hosts](index.md#hosts)
- [Intermediate](index.md#intermediate)
- [LENGTH\_PREFIX\_LENGTH](index.md#length_prefix_length)
- [LibP2PHandlerArgs](index.md#libp2phandlerargs)
- [LibP2PHandlerFunction](index.md#libp2phandlerfunction)
- [MULTI\_ADDR\_MAX\_LENGTH](index.md#multi_addr_max_length)
- [NativeBalance](index.md#nativebalance)
- [NetOptions](index.md#netoptions)
- [POR\_STRING\_LENGTH](index.md#por_string_length)
- [PRG](index.md#prg)
- [PRGParameters](index.md#prgparameters)
- [PRG\_COUNTER\_LENGTH](index.md#prg_counter_length)
- [PRG\_IV\_LENGTH](index.md#prg_iv_length)
- [PRG\_KEY\_LENGTH](index.md#prg_key_length)
- [PRIVATE\_KEY\_LENGTH](index.md#private_key_length)
- [PRP](index.md#prp)
- [PRPParameters](index.md#prpparameters)
- [PRP\_IV\_LENGTH](index.md#prp_iv_length)
- [PRP\_KEY\_LENGTH](index.md#prp_key_length)
- [PRP\_MIN\_LENGTH](index.md#prp_min_length)
- [PUBLIC\_KEY\_LENGTH](index.md#public_key_length)
- [PromiseValue](index.md#promisevalue)
- [PublicKey](index.md#publickey)
- [SECP256K1\_CONSTANTS](index.md#secp256k1_constants)
- [SIGNATURE\_LENGTH](index.md#signature_length)
- [SIGNATURE\_RECOVERY\_LENGTH](index.md#signature_recovery_length)
- [Signature](index.md#signature)
- [Snapshot](index.md#snapshot)
- [Ticket](index.md#ticket)
- [U8aAndSize](index.md#u8aandsize)
- [UINT256](index.md#uint256)
- [UNCOMPRESSED\_PUBLIC\_KEY\_LENGTH](index.md#uncompressed_public_key_length)
- [UnAcknowledgedTickets](index.md#unacknowledgedtickets)
- [UnacknowledgedTicket](index.md#unacknowledgedticket)
- [b58StringRegex](index.md#b58stringregex)
- [cacheNoArgAsyncFunction](index.md#cachenoargasyncfunction)
- [convertPubKeyFromB58String](index.md#convertpubkeyfromb58string)
- [convertPubKeyFromPeerId](index.md#convertpubkeyfrompeerid)
- [createFirstChallenge](index.md#createfirstchallenge)
- [createPacket](index.md#createpacket)
- [createPoRString](index.md#createporstring)
- [deriveAckKeyShare](index.md#deriveackkeyshare)
- [dial](index.md#dial)
- [durations](index.md#durations)
- [forwardTransform](index.md#forwardtransform)
- [gcd](index.md#gcd)
- [generateChannelId](index.md#generatechannelid)
- [generateKeyShares](index.md#generatekeyshares)
- [getB58String](index.md#getb58string)
- [getHeaderLength](index.md#getheaderlength)
- [getPacketLength](index.md#getpacketlength)
- [hasB58String](index.md#hasb58string)
- [isExpired](index.md#isexpired)
- [iterateHash](index.md#iteratehash)
- [lengthPrefixedToU8a](index.md#lengthprefixedtou8a)
- [libp2pSendMessage](index.md#libp2psendmessage)
- [libp2pSendMessageAndExpectResponse](index.md#libp2psendmessageandexpectresponse)
- [libp2pSubscribe](index.md#libp2psubscribe)
- [limitConcurrency](index.md#limitconcurrency)
- [moveDecimalPoint](index.md#movedecimalpoint)
- [oneAtATime](index.md#oneatatime)
- [parseHosts](index.md#parsehosts)
- [parseJSON](index.md#parsejson)
- [preVerify](index.md#preverify)
- [privKeyToPeerId](index.md#privkeytopeerid)
- [pubKeyToPeerId](index.md#pubkeytopeerid)
- [randomChoice](index.md#randomchoice)
- [randomFloat](index.md#randomfloat)
- [randomInteger](index.md#randominteger)
- [randomPermutation](index.md#randompermutation)
- [randomSubset](index.md#randomsubset)
- [recoverIteratedHash](index.md#recoveriteratedhash)
- [sampleGroupElement](index.md#samplegroupelement)
- [serializeToU8a](index.md#serializetou8a)
- [stringToU8a](index.md#stringtou8a)
- [timeoutAfter](index.md#timeoutafter)
- [toLengthPrefixedU8a](index.md#tolengthprefixedu8a)
- [toU8a](index.md#tou8a)
- [u8aAdd](index.md#u8aadd)
- [u8aAllocate](index.md#u8aallocate)
- [u8aCompare](index.md#u8acompare)
- [u8aConcat](index.md#u8aconcat)
- [u8aEquals](index.md#u8aequals)
- [u8aSplit](index.md#u8asplit)
- [u8aToHex](index.md#u8atohex)
- [u8aToNumber](index.md#u8atonumber)
- [u8aXOR](index.md#u8axor)
- [validateAcknowledgement](index.md#validateacknowledgement)

## References

### ADDRESS\_LENGTH

Re-exports: [ADDRESS\_LENGTH](constants.md#address_length)

___

### A\_EQUALS\_B

Re-exports: [A\_EQUALS\_B](u8a_u8acompare.md#a_equals_b)

___

### A\_STRICLY\_LESS\_THAN\_B

Re-exports: [A\_STRICLY\_LESS\_THAN\_B](u8a_u8acompare.md#a_stricly_less_than_b)

___

### A\_STRICTLY\_GREATER\_THAN\_B

Re-exports: [A\_STRICTLY\_GREATER\_THAN\_B](u8a_u8acompare.md#a_strictly_greater_than_b)

___

### AccountEntry

Re-exports: [AccountEntry](../classes/types_accountentry.accountentry.md)

___

### AcknowledgedTicket

Re-exports: [AcknowledgedTicket](../classes/types_acknowledged.acknowledgedticket.md)

___

### Address

Re-exports: [Address](../classes/types_primitives.address.md)

___

### Balance

Re-exports: [Balance](../classes/types_primitives.balance.md)

___

### ChannelEntry

Re-exports: [ChannelEntry](../classes/types_channelentry.channelentry.md)

___

### ChannelStatus

Re-exports: [ChannelStatus](types_channelentry.md#channelstatus)

___

### DialOpts

Re-exports: [DialOpts](libp2p.md#dialopts)

___

### DialResponse

Re-exports: [DialResponse](libp2p.md#dialresponse)

___

### HASH\_LENGTH

Re-exports: [HASH\_LENGTH](constants.md#hash_length)

___

### Hash

Re-exports: [Hash](../classes/types_primitives.hash.md)

___

### HoprDB

Re-exports: [HoprDB](../classes/db.hoprdb.md)

___

### Hosts

Re-exports: [Hosts](hosts.md#hosts)

___

### Intermediate

Re-exports: [Intermediate](../interfaces/crypto_hashiterator.intermediate.md)

___

### LENGTH\_PREFIX\_LENGTH

Re-exports: [LENGTH\_PREFIX\_LENGTH](u8a_constants.md#length_prefix_length)

___

### LibP2PHandlerArgs

Re-exports: [LibP2PHandlerArgs](libp2p.md#libp2phandlerargs)

___

### LibP2PHandlerFunction

Re-exports: [LibP2PHandlerFunction](libp2p.md#libp2phandlerfunction)

___

### MULTI\_ADDR\_MAX\_LENGTH

Re-exports: [MULTI\_ADDR\_MAX\_LENGTH](constants.md#multi_addr_max_length)

___

### NativeBalance

Re-exports: [NativeBalance](../classes/types_primitives.nativebalance.md)

___

### NetOptions

Re-exports: [NetOptions](../interfaces/hosts.netoptions.md)

___

### POR\_STRING\_LENGTH

Re-exports: [POR\_STRING\_LENGTH](crypto_por.md#por_string_length)

___

### PRG

Re-exports: [PRG](../classes/crypto_prg.prg.md)

___

### PRGParameters

Re-exports: [PRGParameters](crypto_prg.md#prgparameters)

___

### PRG\_COUNTER\_LENGTH

Re-exports: [PRG\_COUNTER\_LENGTH](crypto_prg.md#prg_counter_length)

___

### PRG\_IV\_LENGTH

Re-exports: [PRG\_IV\_LENGTH](crypto_prg.md#prg_iv_length)

___

### PRG\_KEY\_LENGTH

Re-exports: [PRG\_KEY\_LENGTH](crypto_prg.md#prg_key_length)

___

### PRIVATE\_KEY\_LENGTH

Re-exports: [PRIVATE\_KEY\_LENGTH](constants.md#private_key_length)

___

### PRP

Re-exports: [PRP](../classes/crypto_prp.prp.md)

___

### PRPParameters

Re-exports: [PRPParameters](crypto_prp.md#prpparameters)

___

### PRP\_IV\_LENGTH

Re-exports: [PRP\_IV\_LENGTH](crypto_prp.md#prp_iv_length)

___

### PRP\_KEY\_LENGTH

Re-exports: [PRP\_KEY\_LENGTH](crypto_prp.md#prp_key_length)

___

### PRP\_MIN\_LENGTH

Re-exports: [PRP\_MIN\_LENGTH](crypto_prp.md#prp_min_length)

___

### PUBLIC\_KEY\_LENGTH

Re-exports: [PUBLIC\_KEY\_LENGTH](constants.md#public_key_length)

___

### PromiseValue

Re-exports: [PromiseValue](typescript.md#promisevalue)

___

### PublicKey

Re-exports: [PublicKey](../classes/types_primitives.publickey.md)

___

### SECP256K1\_CONSTANTS

Re-exports: [SECP256K1\_CONSTANTS](crypto_constants.md#secp256k1_constants)

___

### SIGNATURE\_LENGTH

Re-exports: [SIGNATURE\_LENGTH](constants.md#signature_length)

___

### SIGNATURE\_RECOVERY\_LENGTH

Re-exports: [SIGNATURE\_RECOVERY\_LENGTH](constants.md#signature_recovery_length)

___

### Signature

Re-exports: [Signature](../classes/types_primitives.signature.md)

___

### Snapshot

Re-exports: [Snapshot](../classes/types_snapshot.snapshot.md)

___

### Ticket

Re-exports: [Ticket](../classes/types_ticket.ticket.md)

___

### U8aAndSize

Re-exports: [U8aAndSize](u8a.md#u8aandsize)

___

### UINT256

Re-exports: [UINT256](../classes/types_solidity.uint256.md)

___

### UNCOMPRESSED\_PUBLIC\_KEY\_LENGTH

Re-exports: [UNCOMPRESSED\_PUBLIC\_KEY\_LENGTH](constants.md#uncompressed_public_key_length)

___

### UnAcknowledgedTickets

Re-exports: [UnAcknowledgedTickets](db.md#unacknowledgedtickets)

___

### UnacknowledgedTicket

Re-exports: [UnacknowledgedTicket](../classes/types_unacknowledgedticket.unacknowledgedticket.md)

___

### b58StringRegex

Re-exports: [b58StringRegex](libp2p.md#b58stringregex)

___

### cacheNoArgAsyncFunction

Re-exports: [cacheNoArgAsyncFunction](cache.md#cachenoargasyncfunction)

___

### convertPubKeyFromB58String

Re-exports: [convertPubKeyFromB58String](libp2p.md#convertpubkeyfromb58string)

___

### convertPubKeyFromPeerId

Re-exports: [convertPubKeyFromPeerId](libp2p.md#convertpubkeyfrompeerid)

___

### createFirstChallenge

Re-exports: [createFirstChallenge](crypto_por.md#createfirstchallenge)

___

### createPacket

Re-exports: [createPacket](crypto_packet.md#createpacket)

___

### createPoRString

Re-exports: [createPoRString](crypto_por.md#createporstring)

___

### deriveAckKeyShare

Re-exports: [deriveAckKeyShare](crypto_por_keyderivation.md#deriveackkeyshare)

___

### dial

Re-exports: [dial](libp2p.md#dial)

___

### durations

Re-exports: [durations](time.md#durations)

___

### forwardTransform

Re-exports: [forwardTransform](crypto_packet.md#forwardtransform)

___

### gcd

Re-exports: [gcd](math_gcd.md#gcd)

___

### generateChannelId

Re-exports: [generateChannelId](types_channelentry.md#generatechannelid)

___

### generateKeyShares

Re-exports: [generateKeyShares](crypto_packet_keyshares.md#generatekeyshares)

___

### getB58String

Re-exports: [getB58String](libp2p.md#getb58string)

___

### getHeaderLength

Re-exports: [getHeaderLength](crypto_packet.md#getheaderlength)

___

### getPacketLength

Re-exports: [getPacketLength](crypto_packet.md#getpacketlength)

___

### hasB58String

Re-exports: [hasB58String](libp2p.md#hasb58string)

___

### isExpired

Re-exports: [isExpired](time.md#isexpired)

___

### iterateHash

Re-exports: [iterateHash](crypto_hashiterator.md#iteratehash)

___

### lengthPrefixedToU8a

Re-exports: [lengthPrefixedToU8a](u8a_lengthprefixedtou8a.md#lengthprefixedtou8a)

___

### libp2pSendMessage

Re-exports: [libp2pSendMessage](libp2p.md#libp2psendmessage)

___

### libp2pSendMessageAndExpectResponse

Re-exports: [libp2pSendMessageAndExpectResponse](libp2p.md#libp2psendmessageandexpectresponse)

___

### libp2pSubscribe

Re-exports: [libp2pSubscribe](libp2p.md#libp2psubscribe)

___

### limitConcurrency

Re-exports: [limitConcurrency](collection_promise_pool.md#limitconcurrency)

___

### moveDecimalPoint

Re-exports: [moveDecimalPoint](math_movedecimalpoint.md#movedecimalpoint)

___

### oneAtATime

Re-exports: [oneAtATime](concurrency.md#oneatatime)

___

### parseHosts

Re-exports: [parseHosts](hosts.md#parsehosts)

___

### parseJSON

Re-exports: [parseJSON](parsejson.md#parsejson)

___

### preVerify

Re-exports: [preVerify](crypto_por.md#preverify)

___

### privKeyToPeerId

Re-exports: [privKeyToPeerId](libp2p_privkeytopeerid.md#privkeytopeerid)

___

### pubKeyToPeerId

Re-exports: [pubKeyToPeerId](libp2p_pubkeytopeerid.md#pubkeytopeerid)

___

### randomChoice

Re-exports: [randomChoice](crypto_randominteger.md#randomchoice)

___

### randomFloat

Re-exports: [randomFloat](crypto_randomfloat.md#randomfloat)

___

### randomInteger

Re-exports: [randomInteger](crypto_randominteger.md#randominteger)

___

### randomPermutation

Re-exports: [randomPermutation](collection_randompermutation.md#randompermutation)

___

### randomSubset

Re-exports: [randomSubset](collection_randomsubset.md#randomsubset)

___

### recoverIteratedHash

Re-exports: [recoverIteratedHash](crypto_hashiterator.md#recoveriteratedhash)

___

### sampleGroupElement

Re-exports: [sampleGroupElement](crypto_samplegroupelement.md#samplegroupelement)

___

### serializeToU8a

Re-exports: [serializeToU8a](u8a.md#serializetou8a)

___

### stringToU8a

Re-exports: [stringToU8a](u8a_tou8a.md#stringtou8a)

___

### timeoutAfter

Re-exports: [timeoutAfter](timeout.md#timeoutafter)

___

### toLengthPrefixedU8a

Re-exports: [toLengthPrefixedU8a](u8a_tolengthprefixedu8a.md#tolengthprefixedu8a)

___

### toU8a

Re-exports: [toU8a](u8a_tou8a.md#tou8a)

___

### u8aAdd

Re-exports: [u8aAdd](u8a_u8aadd.md#u8aadd)

___

### u8aAllocate

Re-exports: [u8aAllocate](u8a_allocate.md#u8aallocate)

___

### u8aCompare

Re-exports: [u8aCompare](u8a_u8acompare.md#u8acompare)

___

### u8aConcat

Re-exports: [u8aConcat](u8a_concat.md#u8aconcat)

___

### u8aEquals

Re-exports: [u8aEquals](u8a_equals.md#u8aequals)

___

### u8aSplit

Re-exports: [u8aSplit](u8a.md#u8asplit)

___

### u8aToHex

Re-exports: [u8aToHex](u8a_tohex.md#u8atohex)

___

### u8aToNumber

Re-exports: [u8aToNumber](u8a_u8atonumber.md#u8atonumber)

___

### u8aXOR

Re-exports: [u8aXOR](u8a_xor.md#u8axor)

___

### validateAcknowledgement

Re-exports: [validateAcknowledgement](crypto_por.md#validateacknowledgement)
