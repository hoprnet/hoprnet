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

- [CHECK_TIMEOUT](modules.md#check_timeout)
- [DEFAULT_STUN_PORT](modules.md#default_stun_port)
- [FULL_VERSION](modules.md#full_version)
- [HEARTBEAT_INTERVAL](modules.md#heartbeat_interval)
- [HEARTBEAT_INTERVAL_VARIANCE](modules.md#heartbeat_interval_variance)
- [HEARTBEAT_TIMEOUT](modules.md#heartbeat_timeout)
- [MAX_AUTO_CHANNELS](modules.md#max_auto_channels)
- [MAX_HOPS](modules.md#max_hops)
- [MAX_NEW_CHANNELS_PER_TICK](modules.md#max_new_channels_per_tick)
- [MAX_PACKET_DELAY](modules.md#max_packet_delay)
- [MAX_PARALLEL_CONNECTIONS](modules.md#max_parallel_connections)
- [MAX_PATH_ITERATIONS](modules.md#max_path_iterations)
- [MINIMUM_REASONABLE_CHANNEL_STAKE](modules.md#minimum_reasonable_channel_stake)
- [MIN_NATIVE_BALANCE](modules.md#min_native_balance)
- [NAME](modules.md#name)
- [NETWORK_QUALITY_THRESHOLD](modules.md#network_quality_threshold)
- [PACKET_SIZE](modules.md#packet_size)
- [PATH_RANDOMNESS](modules.md#path_randomness)
- [PROTOCOL_ACKNOWLEDGEMENT](modules.md#protocol_acknowledgement)
- [PROTOCOL_HEARTBEAT](modules.md#protocol_heartbeat)
- [PROTOCOL_ONCHAIN_KEY](modules.md#protocol_onchain_key)
- [PROTOCOL_PAYMENT_CHANNEL](modules.md#protocol_payment_channel)
- [PROTOCOL_STRING](modules.md#protocol_string)
- [SUGGESTED_BALANCE](modules.md#suggested_balance)
- [SUGGESTED_NATIVE_BALANCE](modules.md#suggested_native_balance)
- [TICKET_AMOUNT](modules.md#ticket_amount)
- [TICKET_WIN_PROB](modules.md#ticket_win_prob)
- [VERSION](modules.md#version)

## Type aliases

### ChannelStrategyNames

Ƭ **ChannelStrategyNames**: `"passive"` \| `"promiscuous"`

Defined in: [packages/core/src/index.ts:66](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L66)

---

### HoprOptions

Ƭ **HoprOptions**: _object_

#### Type declaration

| Name                  | Type                                                      |
| :-------------------- | :-------------------------------------------------------- |
| `announce?`           | _boolean_                                                 |
| `connector?`          | HoprCoreEthereum                                          |
| `createDbIfNotExist?` | _boolean_                                                 |
| `dbPath?`             | _string_                                                  |
| `hosts?`              | _object_                                                  |
| `hosts.ip4?`          | NetOptions                                                |
| `hosts.ip6?`          | NetOptions                                                |
| `network`             | _string_                                                  |
| `password?`           | _string_                                                  |
| `provider`            | _string_                                                  |
| `strategy?`           | [_ChannelStrategyNames_](modules.md#channelstrategynames) |
| `ticketAmount?`       | _number_                                                  |
| `ticketWinProb?`      | _number_                                                  |

Defined in: [packages/core/src/index.ts:68](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L68)

---

### NodeStatus

Ƭ **NodeStatus**: `"UNINITIALIZED"` \| `"INITIALIZING"` \| `"RUNNING"` \| `"DESTROYED"`

Defined in: [packages/core/src/index.ts:85](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L85)

## Variables

### CHECK_TIMEOUT

• `Const` **CHECK_TIMEOUT**: `10000`= 10000

Defined in: [packages/core/src/constants.ts:46](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L46)

---

### DEFAULT_STUN_PORT

• `Const` **DEFAULT_STUN_PORT**: `3478`= 3478

Defined in: [packages/core/src/constants.ts:22](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L22)

---

### FULL_VERSION

• `Const` **FULL_VERSION**: _any_

Defined in: [packages/core/src/constants.ts:10](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L10)

---

### HEARTBEAT_INTERVAL

• `Const` **HEARTBEAT_INTERVAL**: `3000`= 3000

Defined in: [packages/core/src/constants.ts:24](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L24)

---

### HEARTBEAT_INTERVAL_VARIANCE

• `Const` **HEARTBEAT_INTERVAL_VARIANCE**: `2000`= 2000

Defined in: [packages/core/src/constants.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L25)

---

### HEARTBEAT_TIMEOUT

• `Const` **HEARTBEAT_TIMEOUT**: `4000`= 4000

Defined in: [packages/core/src/constants.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L29)

---

### MAX_AUTO_CHANNELS

• `Const` **MAX_AUTO_CHANNELS**: `5`= 5

Defined in: [packages/core/src/constants.ts:39](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L39)

---

### MAX_HOPS

• `Const` **MAX_HOPS**: `2`= 2

Defined in: [packages/core/src/constants.ts:33](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L33)

---

### MAX_NEW_CHANNELS_PER_TICK

• `Const` **MAX_NEW_CHANNELS_PER_TICK**: `5`= 5

Defined in: [packages/core/src/constants.ts:38](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L38)

---

### MAX_PACKET_DELAY

• `Const` **MAX_PACKET_DELAY**: `200`= 200

Defined in: [packages/core/src/constants.ts:31](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L31)

---

### MAX_PARALLEL_CONNECTIONS

• `Const` **MAX_PARALLEL_CONNECTIONS**: `5`= 5

Defined in: [packages/core/src/constants.ts:27](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L27)

---

### MAX_PATH_ITERATIONS

• `Const` **MAX_PATH_ITERATIONS**: `100`= 100

Defined in: [packages/core/src/constants.ts:35](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L35)

---

### MINIMUM_REASONABLE_CHANNEL_STAKE

• `Const` **MINIMUM_REASONABLE_CHANNEL_STAKE**: _BN_

Defined in: [packages/core/src/constants.ts:37](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L37)

---

### MIN_NATIVE_BALANCE

• `Const` **MIN_NATIVE_BALANCE**: _BN_

Defined in: [packages/core/src/constants.ts:40](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L40)

---

### NAME

• `Const` **NAME**: `"ipfs"`= 'ipfs'

Defined in: [packages/core/src/constants.ts:8](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L8)

---

### NETWORK_QUALITY_THRESHOLD

• `Const` **NETWORK_QUALITY_THRESHOLD**: `0.5`= 0.5

Defined in: [packages/core/src/constants.ts:36](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L36)

---

### PACKET_SIZE

• `Const` **PACKET_SIZE**: `500`= 500

Defined in: [packages/core/src/constants.ts:7](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L7)

---

### PATH_RANDOMNESS

• `Const` **PATH_RANDOMNESS**: `0.1`= 0.1

Defined in: [packages/core/src/constants.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L34)

---

### PROTOCOL_ACKNOWLEDGEMENT

• `Const` **PROTOCOL_ACKNOWLEDGEMENT**: _string_

Defined in: [packages/core/src/constants.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L18)

---

### PROTOCOL_HEARTBEAT

• `Const` **PROTOCOL_HEARTBEAT**: _string_

Defined in: [packages/core/src/constants.ts:21](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L21)

---

### PROTOCOL_ONCHAIN_KEY

• `Const` **PROTOCOL_ONCHAIN_KEY**: _string_

Defined in: [packages/core/src/constants.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L20)

---

### PROTOCOL_PAYMENT_CHANNEL

• `Const` **PROTOCOL_PAYMENT_CHANNEL**: _string_

Defined in: [packages/core/src/constants.ts:19](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L19)

---

### PROTOCOL_STRING

• `Const` **PROTOCOL_STRING**: _string_

Defined in: [packages/core/src/constants.ts:17](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L17)

---

### SUGGESTED_BALANCE

• `Const` **SUGGESTED_BALANCE**: _BN_

Defined in: [packages/core/src/constants.ts:43](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L43)

---

### SUGGESTED_NATIVE_BALANCE

• `Const` **SUGGESTED_NATIVE_BALANCE**: _BN_

Defined in: [packages/core/src/constants.ts:44](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L44)

---

### TICKET_AMOUNT

• `Const` **TICKET_AMOUNT**: `"10000000000000000"`= '10000000000000000'

Defined in: [packages/core/src/constants.ts:5](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L5)

---

### TICKET_WIN_PROB

• `Const` **TICKET_WIN_PROB**: `1`= 1

Defined in: [packages/core/src/constants.ts:6](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L6)

---

### VERSION

• `Const` **VERSION**: _string_

Defined in: [packages/core/src/constants.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/constants.ts#L13)
