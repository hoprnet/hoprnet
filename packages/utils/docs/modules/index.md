[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / index

# Module: index

## Table of contents

### References

- [ADDRESS_LENGTH](index.md#address_length)
- [A_EQUALS_B](index.md#a_equals_b)
- [A_STRICLY_LESS_THAN_B](index.md#a_stricly_less_than_b)
- [A_STRICTLY_GREATER_THAN_B](index.md#a_strictly_greater_than_b)
- [AccountEntry](index.md#accountentry)
- [AcknowledgedTicket](index.md#acknowledgedticket)
- [Address](index.md#address)
- [Balance](index.md#balance)
- [ChannelEntry](index.md#channelentry)
- [ChannelStatus](index.md#channelstatus)
- [DialOpts](index.md#dialopts)
- [DialResponse](index.md#dialresponse)
- [HASH_LENGTH](index.md#hash_length)
- [Hash](index.md#hash)
- [HoprDB](index.md#hoprdb)
- [Hosts](index.md#hosts)
- [Intermediate](index.md#intermediate)
- [LENGTH_PREFIX_LENGTH](index.md#length_prefix_length)
- [LibP2PHandlerArgs](index.md#libp2phandlerargs)
- [LibP2PHandlerFunction](index.md#libp2phandlerfunction)
- [MULTI_ADDR_MAX_LENGTH](index.md#multi_addr_max_length)
- [NativeBalance](index.md#nativebalance)
- [NetOptions](index.md#netoptions)
- [POR_STRING_LENGTH](index.md#por_string_length)
- [PRG](index.md#prg)
- [PRGParameters](index.md#prgparameters)
- [PRG_COUNTER_LENGTH](index.md#prg_counter_length)
- [PRG_IV_LENGTH](index.md#prg_iv_length)
- [PRG_KEY_LENGTH](index.md#prg_key_length)
- [PRIVATE_KEY_LENGTH](index.md#private_key_length)
- [PRP](index.md#prp)
- [PRPParameters](index.md#prpparameters)
- [PRP_IV_LENGTH](index.md#prp_iv_length)
- [PRP_KEY_LENGTH](index.md#prp_key_length)
- [PRP_MIN_LENGTH](index.md#prp_min_length)
- [PUBLIC_KEY_LENGTH](index.md#public_key_length)
- [PromiseValue](index.md#promisevalue)
- [PublicKey](index.md#publickey)
- [SECP256K1_CONSTANTS](index.md#secp256k1_constants)
- [SIGNATURE_LENGTH](index.md#signature_length)
- [SIGNATURE_RECOVERY_LENGTH](index.md#signature_recovery_length)
- [Signature](index.md#signature)
- [Snapshot](index.md#snapshot)
- [Ticket](index.md#ticket)
- [U8aAndSize](index.md#u8aandsize)
- [UINT256](index.md#uint256)
- [UNCOMPRESSED_PUBLIC_KEY_LENGTH](index.md#uncompressed_public_key_length)
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

### ADDRESS_LENGTH

Re-exports: [ADDRESS_LENGTH](constants.md#address_length)

---

### A_EQUALS_B

Re-exports: [A_EQUALS_B](u8a_u8acompare.md#a_equals_b)

---

### A_STRICLY_LESS_THAN_B

Re-exports: [A_STRICLY_LESS_THAN_B](u8a_u8acompare.md#a_stricly_less_than_b)

---

### A_STRICTLY_GREATER_THAN_B

Re-exports: [A_STRICTLY_GREATER_THAN_B](u8a_u8acompare.md#a_strictly_greater_than_b)

---

### AccountEntry

Re-exports: [AccountEntry](../classes/types_accountentry.accountentry.md)

---

### AcknowledgedTicket

Re-exports: [AcknowledgedTicket](../classes/types_acknowledged.acknowledgedticket.md)

---

### Address

Re-exports: [Address](../classes/types_primitives.address.md)

---

### Balance

Re-exports: [Balance](../classes/types_primitives.balance.md)

---

### ChannelEntry

Re-exports: [ChannelEntry](../classes/types_channelentry.channelentry.md)

---

### ChannelStatus

Re-exports: [ChannelStatus](types_channelentry.md#channelstatus)

---

### DialOpts

Re-exports: [DialOpts](libp2p.md#dialopts)

---

### DialResponse

Re-exports: [DialResponse](libp2p.md#dialresponse)

---

### HASH_LENGTH

Re-exports: [HASH_LENGTH](constants.md#hash_length)

---

### Hash

Re-exports: [Hash](../classes/types_primitives.hash.md)

---

### HoprDB

Re-exports: [HoprDB](../classes/db.hoprdb.md)

---

### Hosts

Re-exports: [Hosts](hosts.md#hosts)

---

### Intermediate

Re-exports: [Intermediate](../interfaces/crypto_hashiterator.intermediate.md)

---

### LENGTH_PREFIX_LENGTH

Re-exports: [LENGTH_PREFIX_LENGTH](u8a_constants.md#length_prefix_length)

---

### LibP2PHandlerArgs

Re-exports: [LibP2PHandlerArgs](libp2p.md#libp2phandlerargs)

---

### LibP2PHandlerFunction

Re-exports: [LibP2PHandlerFunction](libp2p.md#libp2phandlerfunction)

---

### MULTI_ADDR_MAX_LENGTH

Re-exports: [MULTI_ADDR_MAX_LENGTH](constants.md#multi_addr_max_length)

---

### NativeBalance

Re-exports: [NativeBalance](../classes/types_primitives.nativebalance.md)

---

### NetOptions

Re-exports: [NetOptions](../interfaces/hosts.netoptions.md)

---

### POR_STRING_LENGTH

Re-exports: [POR_STRING_LENGTH](crypto_por.md#por_string_length)

---

### PRG

Re-exports: [PRG](../classes/crypto_prg.prg.md)

---

### PRGParameters

Re-exports: [PRGParameters](crypto_prg.md#prgparameters)

---

### PRG_COUNTER_LENGTH

Re-exports: [PRG_COUNTER_LENGTH](crypto_prg.md#prg_counter_length)

---

### PRG_IV_LENGTH

Re-exports: [PRG_IV_LENGTH](crypto_prg.md#prg_iv_length)

---

### PRG_KEY_LENGTH

Re-exports: [PRG_KEY_LENGTH](crypto_prg.md#prg_key_length)

---

### PRIVATE_KEY_LENGTH

Re-exports: [PRIVATE_KEY_LENGTH](constants.md#private_key_length)

---

### PRP

Re-exports: [PRP](../classes/crypto_prp.prp.md)

---

### PRPParameters

Re-exports: [PRPParameters](crypto_prp.md#prpparameters)

---

### PRP_IV_LENGTH

Re-exports: [PRP_IV_LENGTH](crypto_prp.md#prp_iv_length)

---

### PRP_KEY_LENGTH

Re-exports: [PRP_KEY_LENGTH](crypto_prp.md#prp_key_length)

---

### PRP_MIN_LENGTH

Re-exports: [PRP_MIN_LENGTH](crypto_prp.md#prp_min_length)

---

### PUBLIC_KEY_LENGTH

Re-exports: [PUBLIC_KEY_LENGTH](constants.md#public_key_length)

---

### PromiseValue

Re-exports: [PromiseValue](typescript.md#promisevalue)

---

### PublicKey

Re-exports: [PublicKey](../classes/types_primitives.publickey.md)

---

### SECP256K1_CONSTANTS

Re-exports: [SECP256K1_CONSTANTS](crypto_constants.md#secp256k1_constants)

---

### SIGNATURE_LENGTH

Re-exports: [SIGNATURE_LENGTH](constants.md#signature_length)

---

### SIGNATURE_RECOVERY_LENGTH

Re-exports: [SIGNATURE_RECOVERY_LENGTH](constants.md#signature_recovery_length)

---

### Signature

Re-exports: [Signature](../classes/types_primitives.signature.md)

---

### Snapshot

Re-exports: [Snapshot](../classes/types_snapshot.snapshot.md)

---

### Ticket

Re-exports: [Ticket](../classes/types_ticket.ticket.md)

---

### U8aAndSize

Re-exports: [U8aAndSize](u8a.md#u8aandsize)

---

### UINT256

Re-exports: [UINT256](../classes/types_solidity.uint256.md)

---

### UNCOMPRESSED_PUBLIC_KEY_LENGTH

Re-exports: [UNCOMPRESSED_PUBLIC_KEY_LENGTH](constants.md#uncompressed_public_key_length)

---

### UnAcknowledgedTickets

Re-exports: [UnAcknowledgedTickets](db.md#unacknowledgedtickets)

---

### UnacknowledgedTicket

Re-exports: [UnacknowledgedTicket](../classes/types_unacknowledgedticket.unacknowledgedticket.md)

---

### b58StringRegex

Re-exports: [b58StringRegex](libp2p.md#b58stringregex)

---

### cacheNoArgAsyncFunction

Re-exports: [cacheNoArgAsyncFunction](cache.md#cachenoargasyncfunction)

---

### convertPubKeyFromB58String

Re-exports: [convertPubKeyFromB58String](libp2p.md#convertpubkeyfromb58string)

---

### convertPubKeyFromPeerId

Re-exports: [convertPubKeyFromPeerId](libp2p.md#convertpubkeyfrompeerid)

---

### createFirstChallenge

Re-exports: [createFirstChallenge](crypto_por.md#createfirstchallenge)

---

### createPacket

Re-exports: [createPacket](crypto_packet.md#createpacket)

---

### createPoRString

Re-exports: [createPoRString](crypto_por.md#createporstring)

---

### deriveAckKeyShare

Re-exports: [deriveAckKeyShare](crypto_por_keyderivation.md#deriveackkeyshare)

---

### dial

Re-exports: [dial](libp2p.md#dial)

---

### durations

Re-exports: [durations](time.md#durations)

---

### forwardTransform

Re-exports: [forwardTransform](crypto_packet.md#forwardtransform)

---

### gcd

Re-exports: [gcd](math_gcd.md#gcd)

---

### generateChannelId

Re-exports: [generateChannelId](types_channelentry.md#generatechannelid)

---

### generateKeyShares

Re-exports: [generateKeyShares](crypto_packet_keyshares.md#generatekeyshares)

---

### getB58String

Re-exports: [getB58String](libp2p.md#getb58string)

---

### getHeaderLength

Re-exports: [getHeaderLength](crypto_packet.md#getheaderlength)

---

### getPacketLength

Re-exports: [getPacketLength](crypto_packet.md#getpacketlength)

---

### hasB58String

Re-exports: [hasB58String](libp2p.md#hasb58string)

---

### isExpired

Re-exports: [isExpired](time.md#isexpired)

---

### iterateHash

Re-exports: [iterateHash](crypto_hashiterator.md#iteratehash)

---

### lengthPrefixedToU8a

Re-exports: [lengthPrefixedToU8a](u8a_lengthprefixedtou8a.md#lengthprefixedtou8a)

---

### libp2pSendMessage

Re-exports: [libp2pSendMessage](libp2p.md#libp2psendmessage)

---

### libp2pSendMessageAndExpectResponse

Re-exports: [libp2pSendMessageAndExpectResponse](libp2p.md#libp2psendmessageandexpectresponse)

---

### libp2pSubscribe

Re-exports: [libp2pSubscribe](libp2p.md#libp2psubscribe)

---

### limitConcurrency

Re-exports: [limitConcurrency](collection_promise_pool.md#limitconcurrency)

---

### moveDecimalPoint

Re-exports: [moveDecimalPoint](math_movedecimalpoint.md#movedecimalpoint)

---

### oneAtATime

Re-exports: [oneAtATime](concurrency.md#oneatatime)

---

### parseHosts

Re-exports: [parseHosts](hosts.md#parsehosts)

---

### parseJSON

Re-exports: [parseJSON](parsejson.md#parsejson)

---

### preVerify

Re-exports: [preVerify](crypto_por.md#preverify)

---

### privKeyToPeerId

Re-exports: [privKeyToPeerId](libp2p_privkeytopeerid.md#privkeytopeerid)

---

### pubKeyToPeerId

Re-exports: [pubKeyToPeerId](libp2p_pubkeytopeerid.md#pubkeytopeerid)

---

### randomChoice

Re-exports: [randomChoice](crypto_randominteger.md#randomchoice)

---

### randomFloat

Re-exports: [randomFloat](crypto_randomfloat.md#randomfloat)

---

### randomInteger

Re-exports: [randomInteger](crypto_randominteger.md#randominteger)

---

### randomPermutation

Re-exports: [randomPermutation](collection_randompermutation.md#randompermutation)

---

### randomSubset

Re-exports: [randomSubset](collection_randomsubset.md#randomsubset)

---

### recoverIteratedHash

Re-exports: [recoverIteratedHash](crypto_hashiterator.md#recoveriteratedhash)

---

### sampleGroupElement

Re-exports: [sampleGroupElement](crypto_samplegroupelement.md#samplegroupelement)

---

### serializeToU8a

Re-exports: [serializeToU8a](u8a.md#serializetou8a)

---

### stringToU8a

Re-exports: [stringToU8a](u8a_tou8a.md#stringtou8a)

---

### timeoutAfter

Re-exports: [timeoutAfter](timeout.md#timeoutafter)

---

### toLengthPrefixedU8a

Re-exports: [toLengthPrefixedU8a](u8a_tolengthprefixedu8a.md#tolengthprefixedu8a)

---

### toU8a

Re-exports: [toU8a](u8a_tou8a.md#tou8a)

---

### u8aAdd

Re-exports: [u8aAdd](u8a_u8aadd.md#u8aadd)

---

### u8aAllocate

Re-exports: [u8aAllocate](u8a_allocate.md#u8aallocate)

---

### u8aCompare

Re-exports: [u8aCompare](u8a_u8acompare.md#u8acompare)

---

### u8aConcat

Re-exports: [u8aConcat](u8a_concat.md#u8aconcat)

---

### u8aEquals

Re-exports: [u8aEquals](u8a_equals.md#u8aequals)

---

### u8aSplit

Re-exports: [u8aSplit](u8a.md#u8asplit)

---

### u8aToHex

Re-exports: [u8aToHex](u8a_tohex.md#u8atohex)

---

### u8aToNumber

Re-exports: [u8aToNumber](u8a_u8atonumber.md#u8atonumber)

---

### u8aXOR

Re-exports: [u8aXOR](u8a_xor.md#u8axor)

---

### validateAcknowledgement

Re-exports: [validateAcknowledgement](crypto_por.md#validateacknowledgement)
