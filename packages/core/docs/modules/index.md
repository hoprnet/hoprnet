[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / index

# Module: index

## Table of contents

### References

- [CHECK\_TIMEOUT](index.md#check_timeout)
- [DEFAULT\_STUN\_PORT](index.md#default_stun_port)
- [FULL\_VERSION](index.md#full_version)
- [HEARTBEAT\_INTERVAL](index.md#heartbeat_interval)
- [HEARTBEAT\_INTERVAL\_VARIANCE](index.md#heartbeat_interval_variance)
- [HEARTBEAT\_TIMEOUT](index.md#heartbeat_timeout)
- [MAX\_AUTO\_CHANNELS](index.md#max_auto_channels)
- [MAX\_HOPS](index.md#max_hops)
- [MAX\_NEW\_CHANNELS\_PER\_TICK](index.md#max_new_channels_per_tick)
- [MAX\_PACKET\_DELAY](index.md#max_packet_delay)
- [MAX\_PARALLEL\_CONNECTIONS](index.md#max_parallel_connections)
- [MAX\_PATH\_ITERATIONS](index.md#max_path_iterations)
- [MINIMUM\_REASONABLE\_CHANNEL\_STAKE](index.md#minimum_reasonable_channel_stake)
- [MIN\_NATIVE\_BALANCE](index.md#min_native_balance)
- [NAME](index.md#name)
- [NETWORK\_QUALITY\_THRESHOLD](index.md#network_quality_threshold)
- [PACKET\_SIZE](index.md#packet_size)
- [PATH\_RANDOMNESS](index.md#path_randomness)
- [PROTOCOL\_ACKNOWLEDGEMENT](index.md#protocol_acknowledgement)
- [PROTOCOL\_HEARTBEAT](index.md#protocol_heartbeat)
- [PROTOCOL\_ONCHAIN\_KEY](index.md#protocol_onchain_key)
- [PROTOCOL\_PAYMENT\_CHANNEL](index.md#protocol_payment_channel)
- [PROTOCOL\_STRING](index.md#protocol_string)
- [SUGGESTED\_BALANCE](index.md#suggested_balance)
- [SUGGESTED\_NATIVE\_BALANCE](index.md#suggested_native_balance)
- [TICKET\_AMOUNT](index.md#ticket_amount)
- [TICKET\_WIN\_PROB](index.md#ticket_win_prob)
- [VERSION](index.md#version)

### Namespaces

- [LibP2P](index.libp2p.md)

### Classes

- [LibP2P](../classes/index.libp2p-1.md)
- [default](../classes/index.default.md)

### Type aliases

- [ChannelStrategyNames](index.md#channelstrategynames)
- [HoprOptions](index.md#hoproptions)
- [NodeStatus](index.md#nodestatus)

## References

### CHECK\_TIMEOUT

Re-exports: [CHECK\_TIMEOUT](constants.md#check_timeout)

___

### DEFAULT\_STUN\_PORT

Re-exports: [DEFAULT\_STUN\_PORT](constants.md#default_stun_port)

___

### FULL\_VERSION

Re-exports: [FULL\_VERSION](constants.md#full_version)

___

### HEARTBEAT\_INTERVAL

Re-exports: [HEARTBEAT\_INTERVAL](constants.md#heartbeat_interval)

___

### HEARTBEAT\_INTERVAL\_VARIANCE

Re-exports: [HEARTBEAT\_INTERVAL\_VARIANCE](constants.md#heartbeat_interval_variance)

___

### HEARTBEAT\_TIMEOUT

Re-exports: [HEARTBEAT\_TIMEOUT](constants.md#heartbeat_timeout)

___

### MAX\_AUTO\_CHANNELS

Re-exports: [MAX\_AUTO\_CHANNELS](constants.md#max_auto_channels)

___

### MAX\_HOPS

Re-exports: [MAX\_HOPS](constants.md#max_hops)

___

### MAX\_NEW\_CHANNELS\_PER\_TICK

Re-exports: [MAX\_NEW\_CHANNELS\_PER\_TICK](constants.md#max_new_channels_per_tick)

___

### MAX\_PACKET\_DELAY

Re-exports: [MAX\_PACKET\_DELAY](constants.md#max_packet_delay)

___

### MAX\_PARALLEL\_CONNECTIONS

Re-exports: [MAX\_PARALLEL\_CONNECTIONS](constants.md#max_parallel_connections)

___

### MAX\_PATH\_ITERATIONS

Re-exports: [MAX\_PATH\_ITERATIONS](constants.md#max_path_iterations)

___

### MINIMUM\_REASONABLE\_CHANNEL\_STAKE

Re-exports: [MINIMUM\_REASONABLE\_CHANNEL\_STAKE](constants.md#minimum_reasonable_channel_stake)

___

### MIN\_NATIVE\_BALANCE

Re-exports: [MIN\_NATIVE\_BALANCE](constants.md#min_native_balance)

___

### NAME

Re-exports: [NAME](constants.md#name)

___

### NETWORK\_QUALITY\_THRESHOLD

Re-exports: [NETWORK\_QUALITY\_THRESHOLD](constants.md#network_quality_threshold)

___

### PACKET\_SIZE

Re-exports: [PACKET\_SIZE](constants.md#packet_size)

___

### PATH\_RANDOMNESS

Re-exports: [PATH\_RANDOMNESS](constants.md#path_randomness)

___

### PROTOCOL\_ACKNOWLEDGEMENT

Re-exports: [PROTOCOL\_ACKNOWLEDGEMENT](constants.md#protocol_acknowledgement)

___

### PROTOCOL\_HEARTBEAT

Re-exports: [PROTOCOL\_HEARTBEAT](constants.md#protocol_heartbeat)

___

### PROTOCOL\_ONCHAIN\_KEY

Re-exports: [PROTOCOL\_ONCHAIN\_KEY](constants.md#protocol_onchain_key)

___

### PROTOCOL\_PAYMENT\_CHANNEL

Re-exports: [PROTOCOL\_PAYMENT\_CHANNEL](constants.md#protocol_payment_channel)

___

### PROTOCOL\_STRING

Re-exports: [PROTOCOL\_STRING](constants.md#protocol_string)

___

### SUGGESTED\_BALANCE

Re-exports: [SUGGESTED\_BALANCE](constants.md#suggested_balance)

___

### SUGGESTED\_NATIVE\_BALANCE

Re-exports: [SUGGESTED\_NATIVE\_BALANCE](constants.md#suggested_native_balance)

___

### TICKET\_AMOUNT

Re-exports: [TICKET\_AMOUNT](constants.md#ticket_amount)

___

### TICKET\_WIN\_PROB

Re-exports: [TICKET\_WIN\_PROB](constants.md#ticket_win_prob)

___

### VERSION

Re-exports: [VERSION](constants.md#version)

## Type aliases

### ChannelStrategyNames

Ƭ **ChannelStrategyNames**: ``"passive"`` \| ``"promiscuous"``

Defined in: [packages/core/src/index.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L66)

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
| `strategy?` | [*ChannelStrategyNames*](index.md#channelstrategynames) |
| `ticketAmount?` | *number* |
| `ticketWinProb?` | *number* |

Defined in: [packages/core/src/index.ts:68](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L68)

___

### NodeStatus

Ƭ **NodeStatus**: ``"UNINITIALIZED"`` \| ``"INITIALIZING"`` \| ``"RUNNING"`` \| ``"DESTROYED"``

Defined in: [packages/core/src/index.ts:85](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L85)
