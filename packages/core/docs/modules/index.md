[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / index

# Module: index

## Table of contents

### References

- [CHECK_TIMEOUT](index.md#check_timeout)
- [DEFAULT_STUN_PORT](index.md#default_stun_port)
- [FULL_VERSION](index.md#full_version)
- [HEARTBEAT_INTERVAL](index.md#heartbeat_interval)
- [HEARTBEAT_INTERVAL_VARIANCE](index.md#heartbeat_interval_variance)
- [HEARTBEAT_TIMEOUT](index.md#heartbeat_timeout)
- [MAX_AUTO_CHANNELS](index.md#max_auto_channels)
- [MAX_HOPS](index.md#max_hops)
- [MAX_NEW_CHANNELS_PER_TICK](index.md#max_new_channels_per_tick)
- [MAX_PACKET_DELAY](index.md#max_packet_delay)
- [MAX_PARALLEL_CONNECTIONS](index.md#max_parallel_connections)
- [MAX_PATH_ITERATIONS](index.md#max_path_iterations)
- [MINIMUM_REASONABLE_CHANNEL_STAKE](index.md#minimum_reasonable_channel_stake)
- [MIN_NATIVE_BALANCE](index.md#min_native_balance)
- [NAME](index.md#name)
- [NETWORK_QUALITY_THRESHOLD](index.md#network_quality_threshold)
- [PACKET_SIZE](index.md#packet_size)
- [PATH_RANDOMNESS](index.md#path_randomness)
- [PROTOCOL_ACKNOWLEDGEMENT](index.md#protocol_acknowledgement)
- [PROTOCOL_HEARTBEAT](index.md#protocol_heartbeat)
- [PROTOCOL_ONCHAIN_KEY](index.md#protocol_onchain_key)
- [PROTOCOL_PAYMENT_CHANNEL](index.md#protocol_payment_channel)
- [PROTOCOL_STRING](index.md#protocol_string)
- [SUGGESTED_BALANCE](index.md#suggested_balance)
- [SUGGESTED_NATIVE_BALANCE](index.md#suggested_native_balance)
- [TICKET_AMOUNT](index.md#ticket_amount)
- [TICKET_WIN_PROB](index.md#ticket_win_prob)
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

### CHECK_TIMEOUT

Re-exports: [CHECK_TIMEOUT](constants.md#check_timeout)

---

### DEFAULT_STUN_PORT

Re-exports: [DEFAULT_STUN_PORT](constants.md#default_stun_port)

---

### FULL_VERSION

Re-exports: [FULL_VERSION](constants.md#full_version)

---

### HEARTBEAT_INTERVAL

Re-exports: [HEARTBEAT_INTERVAL](constants.md#heartbeat_interval)

---

### HEARTBEAT_INTERVAL_VARIANCE

Re-exports: [HEARTBEAT_INTERVAL_VARIANCE](constants.md#heartbeat_interval_variance)

---

### HEARTBEAT_TIMEOUT

Re-exports: [HEARTBEAT_TIMEOUT](constants.md#heartbeat_timeout)

---

### MAX_AUTO_CHANNELS

Re-exports: [MAX_AUTO_CHANNELS](constants.md#max_auto_channels)

---

### MAX_HOPS

Re-exports: [MAX_HOPS](constants.md#max_hops)

---

### MAX_NEW_CHANNELS_PER_TICK

Re-exports: [MAX_NEW_CHANNELS_PER_TICK](constants.md#max_new_channels_per_tick)

---

### MAX_PACKET_DELAY

Re-exports: [MAX_PACKET_DELAY](constants.md#max_packet_delay)

---

### MAX_PARALLEL_CONNECTIONS

Re-exports: [MAX_PARALLEL_CONNECTIONS](constants.md#max_parallel_connections)

---

### MAX_PATH_ITERATIONS

Re-exports: [MAX_PATH_ITERATIONS](constants.md#max_path_iterations)

---

### MINIMUM_REASONABLE_CHANNEL_STAKE

Re-exports: [MINIMUM_REASONABLE_CHANNEL_STAKE](constants.md#minimum_reasonable_channel_stake)

---

### MIN_NATIVE_BALANCE

Re-exports: [MIN_NATIVE_BALANCE](constants.md#min_native_balance)

---

### NAME

Re-exports: [NAME](constants.md#name)

---

### NETWORK_QUALITY_THRESHOLD

Re-exports: [NETWORK_QUALITY_THRESHOLD](constants.md#network_quality_threshold)

---

### PACKET_SIZE

Re-exports: [PACKET_SIZE](constants.md#packet_size)

---

### PATH_RANDOMNESS

Re-exports: [PATH_RANDOMNESS](constants.md#path_randomness)

---

### PROTOCOL_ACKNOWLEDGEMENT

Re-exports: [PROTOCOL_ACKNOWLEDGEMENT](constants.md#protocol_acknowledgement)

---

### PROTOCOL_HEARTBEAT

Re-exports: [PROTOCOL_HEARTBEAT](constants.md#protocol_heartbeat)

---

### PROTOCOL_ONCHAIN_KEY

Re-exports: [PROTOCOL_ONCHAIN_KEY](constants.md#protocol_onchain_key)

---

### PROTOCOL_PAYMENT_CHANNEL

Re-exports: [PROTOCOL_PAYMENT_CHANNEL](constants.md#protocol_payment_channel)

---

### PROTOCOL_STRING

Re-exports: [PROTOCOL_STRING](constants.md#protocol_string)

---

### SUGGESTED_BALANCE

Re-exports: [SUGGESTED_BALANCE](constants.md#suggested_balance)

---

### SUGGESTED_NATIVE_BALANCE

Re-exports: [SUGGESTED_NATIVE_BALANCE](constants.md#suggested_native_balance)

---

### TICKET_AMOUNT

Re-exports: [TICKET_AMOUNT](constants.md#ticket_amount)

---

### TICKET_WIN_PROB

Re-exports: [TICKET_WIN_PROB](constants.md#ticket_win_prob)

---

### VERSION

Re-exports: [VERSION](constants.md#version)

## Type aliases

### ChannelStrategyNames

Ƭ **ChannelStrategyNames**: `"passive"` \| `"promiscuous"`

Defined in: [packages/core/src/index.ts:66](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L66)

---

### HoprOptions

Ƭ **HoprOptions**: _object_

#### Type declaration

| Name                  | Type                                                    |
| :-------------------- | :------------------------------------------------------ |
| `announce?`           | _boolean_                                               |
| `connector?`          | HoprCoreEthereum                                        |
| `createDbIfNotExist?` | _boolean_                                               |
| `dbPath?`             | _string_                                                |
| `hosts?`              | _object_                                                |
| `hosts.ip4?`          | NetOptions                                              |
| `hosts.ip6?`          | NetOptions                                              |
| `network`             | _string_                                                |
| `password?`           | _string_                                                |
| `provider`            | _string_                                                |
| `strategy?`           | [_ChannelStrategyNames_](index.md#channelstrategynames) |
| `ticketAmount?`       | _number_                                                |
| `ticketWinProb?`      | _number_                                                |

Defined in: [packages/core/src/index.ts:68](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L68)

---

### NodeStatus

Ƭ **NodeStatus**: `"UNINITIALIZED"` \| `"INITIALIZING"` \| `"RUNNING"` \| `"DESTROYED"`

Defined in: [packages/core/src/index.ts:85](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core/src/index.ts#L85)
