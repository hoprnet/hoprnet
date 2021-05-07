[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / indexer/fixtures

# Module: indexer/fixtures

## Table of contents

### Variables

- [FUNDED\_CHANNEL](indexer_fixtures.md#funded_channel)
- [FUNDED\_EVENT](indexer_fixtures.md#funded_event)
- [OPENED\_CHANNEL](indexer_fixtures.md#opened_channel)
- [OPENED\_EVENT](indexer_fixtures.md#opened_event)
- [PARTY\_A\_INITIALIZED\_ACCOUNT](indexer_fixtures.md#party_a_initialized_account)
- [PARTY\_A\_INITIALIZED\_EVENT](indexer_fixtures.md#party_a_initialized_event)
- [partyA](indexer_fixtures.md#partya)
- [partyAMultiAddr](indexer_fixtures.md#partyamultiaddr)
- [partyB](indexer_fixtures.md#partyb)
- [secret1](indexer_fixtures.md#secret1)
- [secret2](indexer_fixtures.md#secret2)

### Functions

- [expectAccountsToBeEqual](indexer_fixtures.md#expectaccountstobeequal)
- [expectChannelsToBeEqual](indexer_fixtures.md#expectchannelstobeequal)

## Variables

### FUNDED\_CHANNEL

• `Const` **FUNDED\_CHANNEL**: *ChannelEntry*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:95](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L95)

___

### FUNDED\_EVENT

• `Const` **FUNDED\_EVENT**: [*Event*](indexer_types.md#event)<``"ChannelUpdate"``\>

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:69](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L69)

___

### OPENED\_CHANNEL

• `Const` **OPENED\_CHANNEL**: *ChannelEntry*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:123](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L123)

___

### OPENED\_EVENT

• `Const` **OPENED\_EVENT**: [*Event*](indexer_types.md#event)<``"ChannelUpdate"``\>

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:97](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L97)

___

### PARTY\_A\_INITIALIZED\_ACCOUNT

• `Const` **PARTY\_A\_INITIALIZED\_ACCOUNT**: *AccountEntry*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:67](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L67)

___

### PARTY\_A\_INITIALIZED\_EVENT

• `Const` **PARTY\_A\_INITIALIZED\_EVENT**: [*Event*](indexer_types.md#event)<``"Announcement"``\>

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:55](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L55)

___

### partyA

• `Const` **partyA**: *PublicKey*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L9)

___

### partyAMultiAddr

• `Const` **partyAMultiAddr**: *Multiaddr*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:13](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L13)

___

### partyB

• `Const` **partyB**: *PublicKey*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:10](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L10)

___

### secret1

• `Const` **secret1**: *Hash*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:11](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L11)

___

### secret2

• `Const` **secret2**: *Hash*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:12](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L12)

## Functions

### expectAccountsToBeEqual

▸ `Const` **expectAccountsToBeEqual**(`actual`: *AccountEntry*, `expected`: *AccountEntry*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `actual` | *AccountEntry* |
| `expected` | *AccountEntry* |

**Returns:** *void*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:17](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L17)

___

### expectChannelsToBeEqual

▸ `Const` **expectChannelsToBeEqual**(`actual`: *ChannelEntry*, `expected`: *ChannelEntry*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `actual` | *ChannelEntry* |
| `expected` | *ChannelEntry* |

**Returns:** *void*

Defined in: [packages/core-ethereum/src/indexer/fixtures.ts:22](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/core-ethereum/src/indexer/fixtures.ts#L22)
