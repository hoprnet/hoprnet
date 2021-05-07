[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprChannels](../modules/contracts_hoprchannels.md) / HoprChannels

# Class: HoprChannels

[contracts/HoprChannels](../modules/contracts_hoprchannels.md).HoprChannels

## Hierarchy

- _Contract_

  ↳ **HoprChannels**

## Table of contents

### Constructors

- [constructor](contracts_hoprchannels.hoprchannels.md#constructor)

### Properties

- [\_deployedPromise](contracts_hoprchannels.hoprchannels.md#_deployedpromise)
- [\_runningEvents](contracts_hoprchannels.hoprchannels.md#_runningevents)
- [\_wrappedEmits](contracts_hoprchannels.hoprchannels.md#_wrappedemits)
- [address](contracts_hoprchannels.hoprchannels.md#address)
- [callStatic](contracts_hoprchannels.hoprchannels.md#callstatic)
- [deployTransaction](contracts_hoprchannels.hoprchannels.md#deploytransaction)
- [estimateGas](contracts_hoprchannels.hoprchannels.md#estimategas)
- [filters](contracts_hoprchannels.hoprchannels.md#filters)
- [functions](contracts_hoprchannels.hoprchannels.md#functions)
- [interface](contracts_hoprchannels.hoprchannels.md#interface)
- [populateTransaction](contracts_hoprchannels.hoprchannels.md#populatetransaction)
- [provider](contracts_hoprchannels.hoprchannels.md#provider)
- [resolvedAddress](contracts_hoprchannels.hoprchannels.md#resolvedaddress)
- [signer](contracts_hoprchannels.hoprchannels.md#signer)

### Methods

- [FUND_CHANNEL_MULTI_SIZE](contracts_hoprchannels.hoprchannels.md#fund_channel_multi_size)
- [FUND_CHANNEL_MULTI_SIZE()](<contracts_hoprchannels.hoprchannels.md#fund_channel_multi_size()>)
- [TOKENS_RECIPIENT_INTERFACE_HASH](contracts_hoprchannels.hoprchannels.md#tokens_recipient_interface_hash)
- [TOKENS_RECIPIENT_INTERFACE_HASH()](<contracts_hoprchannels.hoprchannels.md#tokens_recipient_interface_hash()>)
- [\_checkRunningEvents](contracts_hoprchannels.hoprchannels.md#_checkrunningevents)
- [\_deployed](contracts_hoprchannels.hoprchannels.md#_deployed)
- [\_wrapEvent](contracts_hoprchannels.hoprchannels.md#_wrapevent)
- [announce](contracts_hoprchannels.hoprchannels.md#announce)
- [announce(bytes)](<contracts_hoprchannels.hoprchannels.md#announce(bytes)>)
- [attach](contracts_hoprchannels.hoprchannels.md#attach)
- [bumpChannel](contracts_hoprchannels.hoprchannels.md#bumpchannel)
- [bumpChannel(address,bytes32)](<contracts_hoprchannels.hoprchannels.md#bumpchannel(address,bytes32)>)
- [canImplementInterfaceForAddress](contracts_hoprchannels.hoprchannels.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](<contracts_hoprchannels.hoprchannels.md#canimplementinterfaceforaddress(bytes32,address)>)
- [channels](contracts_hoprchannels.hoprchannels.md#channels)
- [channels(bytes32)](<contracts_hoprchannels.hoprchannels.md#channels(bytes32)>)
- [computeChallenge](contracts_hoprchannels.hoprchannels.md#computechallenge)
- [computeChallenge(bytes32)](<contracts_hoprchannels.hoprchannels.md#computechallenge(bytes32)>)
- [connect](contracts_hoprchannels.hoprchannels.md#connect)
- [deployed](contracts_hoprchannels.hoprchannels.md#deployed)
- [emit](contracts_hoprchannels.hoprchannels.md#emit)
- [fallback](contracts_hoprchannels.hoprchannels.md#fallback)
- [finalizeChannelClosure](contracts_hoprchannels.hoprchannels.md#finalizechannelclosure)
- [finalizeChannelClosure(address)](<contracts_hoprchannels.hoprchannels.md#finalizechannelclosure(address)>)
- [fundChannelMulti](contracts_hoprchannels.hoprchannels.md#fundchannelmulti)
- [fundChannelMulti(address,address,uint256,uint256)](<contracts_hoprchannels.hoprchannels.md#fundchannelmulti(address,address,uint256,uint256)>)
- [initiateChannelClosure](contracts_hoprchannels.hoprchannels.md#initiatechannelclosure)
- [initiateChannelClosure(address)](<contracts_hoprchannels.hoprchannels.md#initiatechannelclosure(address)>)
- [listenerCount](contracts_hoprchannels.hoprchannels.md#listenercount)
- [listeners](contracts_hoprchannels.hoprchannels.md#listeners)
- [off](contracts_hoprchannels.hoprchannels.md#off)
- [on](contracts_hoprchannels.hoprchannels.md#on)
- [once](contracts_hoprchannels.hoprchannels.md#once)
- [queryFilter](contracts_hoprchannels.hoprchannels.md#queryfilter)
- [redeemTicket](contracts_hoprchannels.hoprchannels.md#redeemticket)
- [redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)](<contracts_hoprchannels.hoprchannels.md#redeemticket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)>)
- [removeAllListeners](contracts_hoprchannels.hoprchannels.md#removealllisteners)
- [removeListener](contracts_hoprchannels.hoprchannels.md#removelistener)
- [secsClosure](contracts_hoprchannels.hoprchannels.md#secsclosure)
- [secsClosure()](<contracts_hoprchannels.hoprchannels.md#secsclosure()>)
- [token](contracts_hoprchannels.hoprchannels.md#token)
- [token()](<contracts_hoprchannels.hoprchannels.md#token()>)
- [tokensReceived](contracts_hoprchannels.hoprchannels.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](<contracts_hoprchannels.hoprchannels.md#tokensreceived(address,address,address,uint256,bytes,bytes)>)
- [getContractAddress](contracts_hoprchannels.hoprchannels.md#getcontractaddress)
- [getInterface](contracts_hoprchannels.hoprchannels.md#getinterface)
- [isIndexed](contracts_hoprchannels.hoprchannels.md#isindexed)

## Constructors

### constructor

\+ **new HoprChannels**(`addressOrName`: _string_, `contractInterface`: ContractInterface, `signerOrProvider?`: _Provider_ \| _Signer_): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name                | Type                   |
| :------------------ | :--------------------- |
| `addressOrName`     | _string_               |
| `contractInterface` | ContractInterface      |
| `signerOrProvider?` | _Provider_ \| _Signer_ |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Inherited from: Contract.constructor

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:98

## Properties

### \_deployedPromise

• **\_deployedPromise**: _Promise_<Contract\>

Inherited from: Contract.\_deployedPromise

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:92

---

### \_runningEvents

• **\_runningEvents**: _object_

#### Type declaration

Inherited from: Contract.\_runningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:93

---

### \_wrappedEmits

• **\_wrappedEmits**: _object_

#### Type declaration

Inherited from: Contract.\_wrappedEmits

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:96

---

### address

• `Readonly` **address**: _string_

Inherited from: Contract.address

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:71

---

### callStatic

• **callStatic**: _object_

#### Type declaration

| Name                                                                          | Type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| :---------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `FUND_CHANNEL_MULTI_SIZE`                                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `FUND_CHANNEL_MULTI_SIZE()`                                                   | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                                             | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                                           | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `announce`                                                                    | (`multiaddr`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `announce(bytes)`                                                             | (`multiaddr`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `bumpChannel`                                                                 | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `bumpChannel(address,bytes32)`                                                | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `canImplementInterfaceForAddress`                                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `canImplementInterfaceForAddress(bytes32,address)`                            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `channels`                                                                    | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\> |
| `channels(bytes32)`                                                           | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\> |
| `computeChallenge`                                                            | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `computeChallenge(bytes32)`                                                   | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `finalizeChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `finalizeChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `fundChannelMulti`                                                            | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `fundChannelMulti(address,address,uint256,uint256)`                           | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `initiateChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `initiateChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `redeemTicket`                                                                | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                           |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                           |
| `secsClosure`                                                                 | (`overrides?`: CallOverrides) => _Promise_<number\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `secsClosure()`                                                               | (`overrides?`: CallOverrides) => _Promise_<number\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `token`                                                                       | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `token()`                                                                     | (`overrides?`: CallOverrides) => _Promise_<string\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `tokensReceived`                                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `tokensReceived(address,address,address,uint256,bytes,bytes)`                 | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => _Promise_<void\>                                                                                                                                                                                                                                                                                                                                                                                                                    |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:619

---

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

---

### estimateGas

• **estimateGas**: _object_

#### Type declaration

| Name                                                                          | Type                                                                                                                                                                                                                                                                                                                        |
| :---------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `FUND_CHANNEL_MULTI_SIZE`                                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `FUND_CHANNEL_MULTI_SIZE()`                                                   | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                                             | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                                           | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `announce`                                                                    | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                                                    |
| `announce(bytes)`                                                             | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                                                    |
| `bumpChannel`                                                                 | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                      |
| `bumpChannel(address,bytes32)`                                                | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                      |
| `canImplementInterfaceForAddress`                                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                     |
| `canImplementInterfaceForAddress(bytes32,address)`                            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                     |
| `channels`                                                                    | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                   |
| `channels(bytes32)`                                                           | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                   |
| `computeChallenge`                                                            | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                               |
| `computeChallenge(bytes32)`                                                   | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                               |
| `finalizeChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                                                  |
| `finalizeChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                                                  |
| `fundChannelMulti`                                                            | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                              |
| `fundChannelMulti(address,address,uint256,uint256)`                           | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                              |
| `initiateChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                                                  |
| `initiateChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                                                                                                                                  |
| `redeemTicket`                                                                | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\> |
| `secsClosure`                                                                 | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `secsClosure()`                                                               | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `token`                                                                       | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `token()`                                                                     | (`overrides?`: CallOverrides) => _Promise_<BigNumber\>                                                                                                                                                                                                                                                                      |
| `tokensReceived`                                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                          |
| `tokensReceived(address,address,address,uint256,bytes,bytes)`                 | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<BigNumber\>                                                                                          |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:905

---

### filters

• **filters**: _object_

#### Type declaration

| Name            | Type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| :-------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Announcement`  | (`account`: _string_, `multiaddr`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `account`: _string_ ; `multiaddr`: _string_ }\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `ChannelUpdate` | (`partyA`: _string_, `partyB`: _string_, `newState`: `null`) => [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<[_string_, _string_, [*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }], { `newState`: [*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ } ; `partyA`: _string_ ; `partyB`: _string_ }\> |

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:826

---

### functions

• **functions**: _object_

#### Type declaration

| Name                                                                          | Type                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| :---------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `FUND_CHANNEL_MULTI_SIZE`                                                     | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `FUND_CHANNEL_MULTI_SIZE()`                                                   | (`overrides?`: CallOverrides) => _Promise_<[*BigNumber*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                                             | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                                           | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `announce`                                                                    | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `announce(bytes)`                                                             | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `bumpChannel`                                                                 | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `bumpChannel(address,bytes32)`                                                | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `canImplementInterfaceForAddress`                                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `canImplementInterfaceForAddress(bytes32,address)`                            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `channels`                                                                    | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\> |
| `channels(bytes32)`                                                           | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\> |
| `computeChallenge`                                                            | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `computeChallenge(bytes32)`                                                   | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `finalizeChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `finalizeChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `fundChannelMulti`                                                            | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                               |
| `fundChannelMulti(address,address,uint256,uint256)`                           | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                               |
| `initiateChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `initiateChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `redeemTicket`                                                                | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                  |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                  |
| `secsClosure`                                                                 | (`overrides?`: CallOverrides) => _Promise_<[*number*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `secsClosure()`                                                               | (`overrides?`: CallOverrides) => _Promise_<[*number*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `token`                                                                       | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `token()`                                                                     | (`overrides?`: CallOverrides) => _Promise_<[*string*]\>                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `tokensReceived`                                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                           |
| `tokensReceived(address,address,address,uint256,bytes,bytes)`                 | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<ContractTransaction\>                                                                                                                                                                                                                                                                                                                                                           |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:197

---

### interface

• **interface**: [_HoprChannelsInterface_](../interfaces/contracts_hoprchannels.hoprchannelsinterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:195

---

### populateTransaction

• **populateTransaction**: _object_

#### Type declaration

| Name                                                                          | Type                                                                                                                                                                                                                                                                                                                                   |
| :---------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `FUND_CHANNEL_MULTI_SIZE`                                                     | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `FUND_CHANNEL_MULTI_SIZE()`                                                   | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `TOKENS_RECIPIENT_INTERFACE_HASH`                                             | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `TOKENS_RECIPIENT_INTERFACE_HASH()`                                           | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `announce`                                                                    | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                    |
| `announce(bytes)`                                                             | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                    |
| `bumpChannel`                                                                 | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                      |
| `bumpChannel(address,bytes32)`                                                | (`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                      |
| `canImplementInterfaceForAddress`                                             | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                     |
| `canImplementInterfaceForAddress(bytes32,address)`                            | (`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                     |
| `channels`                                                                    | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                   |
| `channels(bytes32)`                                                           | (`arg0`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                   |
| `computeChallenge`                                                            | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                               |
| `computeChallenge(bytes32)`                                                   | (`response`: BytesLike, `overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                               |
| `finalizeChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                  |
| `finalizeChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                  |
| `fundChannelMulti`                                                            | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                              |
| `fundChannelMulti(address,address,uint256,uint256)`                           | (`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                              |
| `initiateChannelClosure`                                                      | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                  |
| `initiateChannelClosure(address)`                                             | (`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                  |
| `redeemTicket`                                                                | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\> |
| `secsClosure`                                                                 | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `secsClosure()`                                                               | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `token`                                                                       | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `token()`                                                                     | (`overrides?`: CallOverrides) => _Promise_<PopulatedTransaction\>                                                                                                                                                                                                                                                                      |
| `tokensReceived`                                                              | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                          |
| `tokensReceived(address,address,address,uint256,bytes,bytes)`                 | (`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }) => _Promise_<PopulatedTransaction\>                                                                                          |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:1058

---

### provider

• `Readonly` **provider**: _Provider_

Inherited from: Contract.provider

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:74

---

### resolvedAddress

• `Readonly` **resolvedAddress**: _Promise_<string\>

Inherited from: Contract.resolvedAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:90

---

### signer

• `Readonly` **signer**: _Signer_

Inherited from: Contract.signer

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:73

## Methods

### FUND_CHANNEL_MULTI_SIZE

▸ **FUND_CHANNEL_MULTI_SIZE**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:411

---

### FUND_CHANNEL_MULTI_SIZE()

▸ **FUND_CHANNEL_MULTI_SIZE()**(`overrides?`: CallOverrides): _Promise_<BigNumber\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:411

---

### TOKENS_RECIPIENT_INTERFACE_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:415

---

### TOKENS_RECIPIENT_INTERFACE_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:415

---

### \_checkRunningEvents

▸ **\_checkRunningEvents**(`runningEvent`: _RunningEvent_): _void_

#### Parameters

| Name           | Type           |
| :------------- | :------------- |
| `runningEvent` | _RunningEvent_ |

**Returns:** _void_

Inherited from: Contract.\_checkRunningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:113

---

### \_deployed

▸ **\_deployed**(`blockTag?`: BlockTag): _Promise_<Contract\>

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `blockTag?` | BlockTag |

**Returns:** _Promise_<Contract\>

Inherited from: Contract.\_deployed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:106

---

### \_wrapEvent

▸ **\_wrapEvent**(`runningEvent`: _RunningEvent_, `log`: Log, `listener`: Listener): Event

#### Parameters

| Name           | Type           |
| :------------- | :------------- |
| `runningEvent` | _RunningEvent_ |
| `log`          | Log            |
| `listener`     | Listener       |

**Returns:** Event

Inherited from: Contract.\_wrapEvent

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:114

---

### announce

▸ **announce**(`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `multiaddr`  | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:421

---

### announce(bytes)

▸ **announce(bytes)**(`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `multiaddr`  | BytesLike                                               |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:424

---

### attach

▸ **attach**(`addressOrName`: _string_): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name            | Type     |
| :-------------- | :------- |
| `addressOrName` | _string_ |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:156

---

### bumpChannel

▸ **bumpChannel**(`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name            | Type                                                    |
| :-------------- | :------------------------------------------------------ |
| `counterparty`  | _string_                                                |
| `newCommitment` | BytesLike                                               |
| `overrides?`    | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:431

---

### bumpChannel(address,bytes32)

▸ **bumpChannel(address,bytes32)**(`counterparty`: _string_, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name            | Type                                                    |
| :-------------- | :------------------------------------------------------ |
| `counterparty`  | _string_                                                |
| `newCommitment` | BytesLike                                               |
| `overrides?`    | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:435

---

### canImplementInterfaceForAddress

▸ **canImplementInterfaceForAddress**(`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name            | Type          |
| :-------------- | :------------ |
| `interfaceHash` | BytesLike     |
| `account`       | _string_      |
| `overrides?`    | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:443

---

### canImplementInterfaceForAddress(bytes32,address)

▸ **canImplementInterfaceForAddress(bytes32,address)**(`interfaceHash`: BytesLike, `account`: _string_, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name            | Type          |
| :-------------- | :------------ |
| `interfaceHash` | BytesLike     |
| `account`       | _string_      |
| `overrides?`    | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:447

---

### channels

▸ **channels**(`arg0`: BytesLike, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:455

---

### channels(bytes32)

▸ **channels(bytes32)**(`arg0`: BytesLike, `overrides?`: CallOverrides): _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `arg0`       | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: _BigNumber_ ; `closureByPartyA`: _boolean_ ; `closureTime`: _number_ ; `partyABalance`: _BigNumber_ ; `partyACommitment`: _string_ ; `partyATicketEpoch`: _BigNumber_ ; `partyATicketIndex`: _BigNumber_ ; `partyBBalance`: _BigNumber_ ; `partyBCommitment`: _string_ ; `partyBTicketEpoch`: _BigNumber_ ; `partyBTicketIndex`: _BigNumber_ ; `status`: _number_ }\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:486

---

### computeChallenge

▸ **computeChallenge**(`response`: BytesLike, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `response`   | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:521

---

### computeChallenge(bytes32)

▸ **computeChallenge(bytes32)**(`response`: BytesLike, `overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `response`   | BytesLike     |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:524

---

### connect

▸ **connect**(`signerOrProvider`: _string_ \| _Provider_ \| _Signer_): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name               | Type                               |
| :----------------- | :--------------------------------- |
| `signerOrProvider` | _string_ \| _Provider_ \| _Signer_ |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:155

---

### deployed

▸ **deployed**(): _Promise_<[_HoprChannels_](contracts_hoprchannels.hoprchannels.md)\>

**Returns:** _Promise_<[_HoprChannels_](contracts_hoprchannels.hoprchannels.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:157

---

### emit

▸ **emit**(`eventName`: _string_ \| EventFilter, ...`args`: _any_[]): _boolean_

#### Parameters

| Name        | Type                    |
| :---------- | :---------------------- |
| `eventName` | _string_ \| EventFilter |
| `...args`   | _any_[]                 |

**Returns:** _boolean_

Inherited from: Contract.emit

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:119

---

### fallback

▸ **fallback**(`overrides?`: TransactionRequest): _Promise_<TransactionResponse\>

#### Parameters

| Name         | Type               |
| :----------- | :----------------- |
| `overrides?` | TransactionRequest |

**Returns:** _Promise_<TransactionResponse\>

Inherited from: Contract.fallback

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:107

---

### finalizeChannelClosure

▸ **finalizeChannelClosure**(`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `counterparty` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:531

---

### finalizeChannelClosure(address)

▸ **finalizeChannelClosure(address)**(`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `counterparty` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:534

---

### fundChannelMulti

▸ **fundChannelMulti**(`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `account1`   | _string_                                                |
| `account2`   | _string_                                                |
| `amount1`    | BigNumberish                                            |
| `amount2`    | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:541

---

### fundChannelMulti(address,address,uint256,uint256)

▸ **fundChannelMulti(address,address,uint256,uint256)**(`account1`: _string_, `account2`: _string_, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name         | Type                                                    |
| :----------- | :------------------------------------------------------ |
| `account1`   | _string_                                                |
| `account2`   | _string_                                                |
| `amount1`    | BigNumberish                                            |
| `amount2`    | BigNumberish                                            |
| `overrides?` | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:547

---

### initiateChannelClosure

▸ **initiateChannelClosure**(`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `counterparty` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:557

---

### initiateChannelClosure(address)

▸ **initiateChannelClosure(address)**(`counterparty`: _string_, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `counterparty` | _string_                                                |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:560

---

### listenerCount

▸ **listenerCount**(`eventName?`: _string_ \| EventFilter): _number_

#### Parameters

| Name         | Type                    |
| :----------- | :---------------------- |
| `eventName?` | _string_ \| EventFilter |

**Returns:** _number_

Inherited from: Contract.listenerCount

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:120

---

### listeners

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name           | Type                                                                                                        |
| :------------- | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter?` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:159

▸ **listeners**(`eventName?`: _string_): Listener[]

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:182

---

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:162

▸ **off**(`eventName`: _string_, `listener`: Listener): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:183

---

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:166

▸ **on**(`eventName`: _string_, `listener`: Listener): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:184

---

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:170

▸ **once**(`eventName`: _string_, `listener`: Listener): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:185

---

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `fromBlockOrBlockhash?`: _string_ \| _number_, `toBlock?`: _string_ \| _number_): _Promise_<[_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name                    | Type                                                                                                        |
| :---------------------- | :---------------------------------------------------------------------------------------------------------- |
| `event`                 | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | _string_ \| _number_                                                                                        |
| `toBlock?`              | _string_ \| _number_                                                                                        |

**Returns:** _Promise_<[_TypedEvent_](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

Overrides: Contract.queryFilter

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:189

---

### redeemTicket

▸ **redeemTicket**(`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name                 | Type                                                    |
| :------------------- | :------------------------------------------------------ |
| `counterparty`       | _string_                                                |
| `nextCommitment`     | BytesLike                                               |
| `ticketEpoch`        | BigNumberish                                            |
| `ticketIndex`        | BigNumberish                                            |
| `proofOfRelaySecret` | BytesLike                                               |
| `amount`             | BigNumberish                                            |
| `winProb`            | BigNumberish                                            |
| `signature`          | BytesLike                                               |
| `overrides?`         | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:567

---

### redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)

▸ **redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)**(`counterparty`: _string_, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name                 | Type                                                    |
| :------------------- | :------------------------------------------------------ |
| `counterparty`       | _string_                                                |
| `nextCommitment`     | BytesLike                                               |
| `ticketEpoch`        | BigNumberish                                            |
| `ticketIndex`        | BigNumberish                                            |
| `proofOfRelaySecret` | BytesLike                                               |
| `amount`             | BigNumberish                                            |
| `winProb`            | BigNumberish                                            |
| `signature`          | BytesLike                                               |
| `overrides?`         | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:577

---

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:178

▸ **removeAllListeners**(`eventName?`: _string_): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name         | Type     |
| :----------- | :------- |
| `eventName?` | _string_ |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:187

---

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name              | Type    |
| :---------------- | :------ |
| `EventArgsArray`  | _any_[] |
| `EventArgsObject` | -       |

#### Parameters

| Name          | Type                                                                                                        |
| :------------ | :---------------------------------------------------------------------------------------------------------- |
| `eventFilter` | [_TypedEventFilter_](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener`    | [_TypedListener_](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>          |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:174

▸ **removeListener**(`eventName`: _string_, `listener`: Listener): [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name        | Type     |
| :---------- | :------- |
| `eventName` | _string_ |
| `listener`  | Listener |

**Returns:** [_HoprChannels_](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:186

---

### secsClosure

▸ **secsClosure**(`overrides?`: CallOverrides): _Promise_<number\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<number\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:591

---

### secsClosure()

▸ **secsClosure()**(`overrides?`: CallOverrides): _Promise_<number\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<number\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:591

---

### token

▸ **token**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:595

---

### token()

▸ **token()**(`overrides?`: CallOverrides): _Promise_<string\>

#### Parameters

| Name         | Type          |
| :----------- | :------------ |
| `overrides?` | CallOverrides |

**Returns:** _Promise_<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:595

---

### tokensReceived

▸ **tokensReceived**(`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `operator`     | _string_                                                |
| `from`         | _string_                                                |
| `to`           | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:599

---

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`: _string_, `from`: _string_, `to`: _string_, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: _string_ \| _Promise_<string\> }): _Promise_<ContractTransaction\>

#### Parameters

| Name           | Type                                                    |
| :------------- | :------------------------------------------------------ |
| `operator`     | _string_                                                |
| `from`         | _string_                                                |
| `to`           | _string_                                                |
| `amount`       | BigNumberish                                            |
| `userData`     | BytesLike                                               |
| `operatorData` | BytesLike                                               |
| `overrides?`   | Overrides & { `from?`: _string_ \| _Promise_<string\> } |

**Returns:** _Promise_<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:607

---

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`: { `from`: _string_ ; `nonce`: BigNumberish }): _string_

#### Parameters

| Name                | Type         |
| :------------------ | :----------- |
| `transaction`       | _object_     |
| `transaction.from`  | _string_     |
| `transaction.nonce` | BigNumberish |

**Returns:** _string_

Inherited from: Contract.getContractAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:100

---

### getInterface

▸ `Static` **getInterface**(`contractInterface`: ContractInterface): _Interface_

#### Parameters

| Name                | Type              |
| :------------------ | :---------------- |
| `contractInterface` | ContractInterface |

**Returns:** _Interface_

Inherited from: Contract.getInterface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:104

---

### isIndexed

▸ `Static` **isIndexed**(`value`: _any_): value is Indexed

#### Parameters

| Name    | Type  |
| :------ | :---- |
| `value` | _any_ |

**Returns:** value is Indexed

Inherited from: Contract.isIndexed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:110
