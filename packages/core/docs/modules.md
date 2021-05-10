[@hoprnet/hopr-core](README.md) / Exports

# @hoprnet/hopr-core

## Table of contents

### Namespaces

- [LibP2P](modules/libp2p.md)

### Classes

- [LibP2P](classes/libp2p.md)
- [default](classes/default.md)

### Type aliases

- [ChannelStrategyNames](modules.md#channelstrategynames)
- [HoprOptions](modules.md#hoproptions)
- [NodeStatus](modules.md#nodestatus)

### Variables

- [CHECK\_TIMEOUT](modules.md#check_timeout)
- [DEFAULT\_STUN\_PORT](modules.md#default_stun_port)
- [FULL\_VERSION](modules.md#full_version)
- [HEARTBEAT\_INTERVAL](modules.md#heartbeat_interval)
- [HEARTBEAT\_INTERVAL\_VARIANCE](modules.md#heartbeat_interval_variance)
- [HEARTBEAT\_TIMEOUT](modules.md#heartbeat_timeout)
- [MAX\_AUTO\_CHANNELS](modules.md#max_auto_channels)
- [MAX\_HOPS](modules.md#max_hops)
- [MAX\_NEW\_CHANNELS\_PER\_TICK](modules.md#max_new_channels_per_tick)
- [MAX\_PACKET\_DELAY](modules.md#max_packet_delay)
- [MAX\_PARALLEL\_CONNECTIONS](modules.md#max_parallel_connections)
- [MAX\_PATH\_ITERATIONS](modules.md#max_path_iterations)
- [MINIMUM\_REASONABLE\_CHANNEL\_STAKE](modules.md#minimum_reasonable_channel_stake)
- [MIN\_NATIVE\_BALANCE](modules.md#min_native_balance)
- [NAME](modules.md#name)
- [NETWORK\_QUALITY\_THRESHOLD](modules.md#network_quality_threshold)
- [PACKET\_SIZE](modules.md#packet_size)
- [PATH\_RANDOMNESS](modules.md#path_randomness)
- [PROTOCOL\_ACKNOWLEDGEMENT](modules.md#protocol_acknowledgement)
- [PROTOCOL\_HEARTBEAT](modules.md#protocol_heartbeat)
- [PROTOCOL\_ONCHAIN\_KEY](modules.md#protocol_onchain_key)
- [PROTOCOL\_PAYMENT\_CHANNEL](modules.md#protocol_payment_channel)
- [PROTOCOL\_STRING](modules.md#protocol_string)
- [SUGGESTED\_BALANCE](modules.md#suggested_balance)
- [SUGGESTED\_NATIVE\_BALANCE](modules.md#suggested_native_balance)
- [TICKET\_AMOUNT](modules.md#ticket_amount)
- [TICKET\_WIN\_PROB](modules.md#ticket_win_prob)
- [VERSION](modules.md#version)

## Type aliases

### ChannelStrategyNames

Ƭ **ChannelStrategyNames**: ``"passive"`` \| ``"promiscuous"``

Defined in: [packages/core/src/index.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L66)

___

### HoprOptions

Ƭ **HoprOptions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `announce?` | *boolean* |
| `connector?` | HoprCoreEthereum |
| `createDbIfNotExist?` | *boolean* |
| `dbPath?` | *string* |
| `hosts?` | *object* |
| `hosts.ip4?` | NetOptions |
| `hosts.ip6?` | NetOptions |
| `network` | *string* |
| `password?` | *string* |
| `provider` | *string* |
| `strategy?` | [*ChannelStrategyNames*](modules.md#channelstrategynames) |
| `ticketAmount?` | *number* |
| `ticketWinProb?` | *number* |

Defined in: [packages/core/src/index.ts:68](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L68)

___

### NodeStatus

Ƭ **NodeStatus**: ``"UNINITIALIZED"`` \| ``"INITIALIZING"`` \| ``"RUNNING"`` \| ``"DESTROYED"``

Defined in: [packages/core/src/index.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L85)

## Variables

### CHECK\_TIMEOUT

• `Const` **CHECK\_TIMEOUT**: ``10000``= 10000

Defined in: [packages/core/src/constants.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L46)

___

### DEFAULT\_STUN\_PORT

• `Const` **DEFAULT\_STUN\_PORT**: ``3478``= 3478

Defined in: [packages/core/src/constants.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L22)

___

### FULL\_VERSION

• `Const` **FULL\_VERSION**: *any*

Defined in: [packages/core/src/constants.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L10)

___

### HEARTBEAT\_INTERVAL

• `Const` **HEARTBEAT\_INTERVAL**: ``3000``= 3000

Defined in: [packages/core/src/constants.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L24)

___

### HEARTBEAT\_INTERVAL\_VARIANCE

• `Const` **HEARTBEAT\_INTERVAL\_VARIANCE**: ``2000``= 2000

Defined in: [packages/core/src/constants.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L25)

___

### HEARTBEAT\_TIMEOUT

• `Const` **HEARTBEAT\_TIMEOUT**: ``4000``= 4000

Defined in: [packages/core/src/constants.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L29)

___

### MAX\_AUTO\_CHANNELS

• `Const` **MAX\_AUTO\_CHANNELS**: ``5``= 5

Defined in: [packages/core/src/constants.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L39)

___

### MAX\_HOPS

• `Const` **MAX\_HOPS**: ``2``= 2

Defined in: [packages/core/src/constants.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L33)

___

### MAX\_NEW\_CHANNELS\_PER\_TICK

• `Const` **MAX\_NEW\_CHANNELS\_PER\_TICK**: ``5``= 5

Defined in: [packages/core/src/constants.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L38)

___

### MAX\_PACKET\_DELAY

• `Const` **MAX\_PACKET\_DELAY**: ``200``= 200

Defined in: [packages/core/src/constants.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L31)

___

### MAX\_PARALLEL\_CONNECTIONS

• `Const` **MAX\_PARALLEL\_CONNECTIONS**: ``5``= 5

Defined in: [packages/core/src/constants.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L27)

___

### MAX\_PATH\_ITERATIONS

• `Const` **MAX\_PATH\_ITERATIONS**: ``100``= 100

Defined in: [packages/core/src/constants.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L35)

___

### MINIMUM\_REASONABLE\_CHANNEL\_STAKE

• `Const` **MINIMUM\_REASONABLE\_CHANNEL\_STAKE**: *BN*

Defined in: [packages/core/src/constants.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L37)

___

### MIN\_NATIVE\_BALANCE

• `Const` **MIN\_NATIVE\_BALANCE**: *BN*

Defined in: [packages/core/src/constants.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L40)

___

### NAME

• `Const` **NAME**: ``"ipfs"``= 'ipfs'

Defined in: [packages/core/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L8)

___

### NETWORK\_QUALITY\_THRESHOLD

• `Const` **NETWORK\_QUALITY\_THRESHOLD**: ``0.5``= 0.5

Defined in: [packages/core/src/constants.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L36)

___

### PACKET\_SIZE

• `Const` **PACKET\_SIZE**: ``500``= 500

Defined in: [packages/core/src/constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L7)

___

### PATH\_RANDOMNESS

• `Const` **PATH\_RANDOMNESS**: ``0.1``= 0.1

Defined in: [packages/core/src/constants.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L34)

___

### PROTOCOL\_ACKNOWLEDGEMENT

• `Const` **PROTOCOL\_ACKNOWLEDGEMENT**: *string*

Defined in: [packages/core/src/constants.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L18)

___

### PROTOCOL\_HEARTBEAT

• `Const` **PROTOCOL\_HEARTBEAT**: *string*

Defined in: [packages/core/src/constants.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L21)

___

### PROTOCOL\_ONCHAIN\_KEY

• `Const` **PROTOCOL\_ONCHAIN\_KEY**: *string*

Defined in: [packages/core/src/constants.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L20)

___

### PROTOCOL\_PAYMENT\_CHANNEL

• `Const` **PROTOCOL\_PAYMENT\_CHANNEL**: *string*

Defined in: [packages/core/src/constants.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L19)

___

### PROTOCOL\_STRING

• `Const` **PROTOCOL\_STRING**: *string*

Defined in: [packages/core/src/constants.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L17)

___

### SUGGESTED\_BALANCE

• `Const` **SUGGESTED\_BALANCE**: *BN*

Defined in: [packages/core/src/constants.ts:43](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L43)

___

### SUGGESTED\_NATIVE\_BALANCE

• `Const` **SUGGESTED\_NATIVE\_BALANCE**: *BN*

Defined in: [packages/core/src/constants.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L44)

___

### TICKET\_AMOUNT

• `Const` **TICKET\_AMOUNT**: ``"10000000000000000"``= '10000000000000000'

Defined in: [packages/core/src/constants.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L5)

___

### TICKET\_WIN\_PROB

• `Const` **TICKET\_WIN\_PROB**: ``1``= 1

Defined in: [packages/core/src/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L6)

___

### VERSION

• `Const` **VERSION**: *string*

Defined in: [packages/core/src/constants.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L13)
