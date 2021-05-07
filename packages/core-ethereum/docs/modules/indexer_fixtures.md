[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / indexer/fixtures

# Module: indexer/fixtures

## Table of contents

### Variables

- [FUNDED_CHANNEL](indexer_fixtures.md#funded_channel)
- [FUNDED_EVENT](indexer_fixtures.md#funded_event)
- [OPENED_CHANNEL](indexer_fixtures.md#opened_channel)
- [OPENED_EVENT](indexer_fixtures.md#opened_event)
- [PARTY_A_INITIALIZED_ACCOUNT](indexer_fixtures.md#party_a_initialized_account)
- [PARTY_A_INITIALIZED_EVENT](indexer_fixtures.md#party_a_initialized_event)
- [partyA](indexer_fixtures.md#partya)
- [partyAMultiAddr](indexer_fixtures.md#partyamultiaddr)
- [partyB](indexer_fixtures.md#partyb)
- [secret1](indexer_fixtures.md#secret1)
- [secret2](indexer_fixtures.md#secret2)

### Functions

- [expectAccountsToBeEqual](indexer_fixtures.md#expectaccountstobeequal)
- [expectChannelsToBeEqual](indexer_fixtures.md#expectchannelstobeequal)

## Variables

### FUNDED_CHANNEL

• `Const` **FUNDED_CHANNEL**: _ChannelEntry_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L95)

---

### FUNDED_EVENT

• `Const` **FUNDED_EVENT**: [_Event_](indexer_types.md#event)<`"ChannelUpdate"`\>

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:69](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L69)

---

### OPENED_CHANNEL

• `Const` **OPENED_CHANNEL**: _ChannelEntry_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:123](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L123)

---

### OPENED_EVENT

• `Const` **OPENED_EVENT**: [_Event_](indexer_types.md#event)<`"ChannelUpdate"`\>

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:97](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L97)

---

### PARTY_A_INITIALIZED_ACCOUNT

• `Const` **PARTY_A_INITIALIZED_ACCOUNT**: _AccountEntry_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:67](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L67)

---

### PARTY_A_INITIALIZED_EVENT

• `Const` **PARTY_A_INITIALIZED_EVENT**: [_Event_](indexer_types.md#event)<`"Announcement"`\>

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L55)

---

### partyA

• `Const` **partyA**: _PublicKey_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L9)

---

### partyAMultiAddr

• `Const` **partyAMultiAddr**: _Multiaddr_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L13)

---

### partyB

• `Const` **partyB**: _PublicKey_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L10)

---

### secret1

• `Const` **secret1**: _Hash_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L11)

---

### secret2

• `Const` **secret2**: _Hash_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L12)

## Functions

### expectAccountsToBeEqual

▸ `Const` **expectAccountsToBeEqual**(`actual`: _AccountEntry_, `expected`: _AccountEntry_): _void_

#### Parameters

| Name       | Type           |
| :--------- | :------------- |
| `actual`   | _AccountEntry_ |
| `expected` | _AccountEntry_ |

**Returns:** _void_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L17)

---

### expectChannelsToBeEqual

▸ `Const` **expectChannelsToBeEqual**(`actual`: _ChannelEntry_, `expected`: _ChannelEntry_): _void_

#### Parameters

| Name       | Type           |
| :--------- | :------------- |
| `actual`   | _ChannelEntry_ |
| `expected` | _ChannelEntry_ |

**Returns:** _void_

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L22)
