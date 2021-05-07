[@hoprnet/hopr-core-ethereum](../README.md) / [Exports](../modules.md) / [contracts/HoprChannels](../modules/contracts_hoprchannels.md) / HoprChannels

# Class: HoprChannels

[contracts/HoprChannels](../modules/contracts_hoprchannels.md).HoprChannels

## Hierarchy

- *Contract*

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

- [FUND\_CHANNEL\_MULTI\_SIZE](contracts_hoprchannels.hoprchannels.md#fund_channel_multi_size)
- [FUND\_CHANNEL\_MULTI\_SIZE()](contracts_hoprchannels.hoprchannels.md#fund_channel_multi_size())
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH](contracts_hoprchannels.hoprchannels.md#tokens_recipient_interface_hash)
- [TOKENS\_RECIPIENT\_INTERFACE\_HASH()](contracts_hoprchannels.hoprchannels.md#tokens_recipient_interface_hash())
- [\_checkRunningEvents](contracts_hoprchannels.hoprchannels.md#_checkrunningevents)
- [\_deployed](contracts_hoprchannels.hoprchannels.md#_deployed)
- [\_wrapEvent](contracts_hoprchannels.hoprchannels.md#_wrapevent)
- [announce](contracts_hoprchannels.hoprchannels.md#announce)
- [announce(bytes)](contracts_hoprchannels.hoprchannels.md#announce(bytes))
- [attach](contracts_hoprchannels.hoprchannels.md#attach)
- [bumpChannel](contracts_hoprchannels.hoprchannels.md#bumpchannel)
- [bumpChannel(address,bytes32)](contracts_hoprchannels.hoprchannels.md#bumpchannel(address,bytes32))
- [canImplementInterfaceForAddress](contracts_hoprchannels.hoprchannels.md#canimplementinterfaceforaddress)
- [canImplementInterfaceForAddress(bytes32,address)](contracts_hoprchannels.hoprchannels.md#canimplementinterfaceforaddress(bytes32,address))
- [channels](contracts_hoprchannels.hoprchannels.md#channels)
- [channels(bytes32)](contracts_hoprchannels.hoprchannels.md#channels(bytes32))
- [computeChallenge](contracts_hoprchannels.hoprchannels.md#computechallenge)
- [computeChallenge(bytes32)](contracts_hoprchannels.hoprchannels.md#computechallenge(bytes32))
- [connect](contracts_hoprchannels.hoprchannels.md#connect)
- [deployed](contracts_hoprchannels.hoprchannels.md#deployed)
- [emit](contracts_hoprchannels.hoprchannels.md#emit)
- [fallback](contracts_hoprchannels.hoprchannels.md#fallback)
- [finalizeChannelClosure](contracts_hoprchannels.hoprchannels.md#finalizechannelclosure)
- [finalizeChannelClosure(address)](contracts_hoprchannels.hoprchannels.md#finalizechannelclosure(address))
- [fundChannelMulti](contracts_hoprchannels.hoprchannels.md#fundchannelmulti)
- [fundChannelMulti(address,address,uint256,uint256)](contracts_hoprchannels.hoprchannels.md#fundchannelmulti(address,address,uint256,uint256))
- [initiateChannelClosure](contracts_hoprchannels.hoprchannels.md#initiatechannelclosure)
- [initiateChannelClosure(address)](contracts_hoprchannels.hoprchannels.md#initiatechannelclosure(address))
- [listenerCount](contracts_hoprchannels.hoprchannels.md#listenercount)
- [listeners](contracts_hoprchannels.hoprchannels.md#listeners)
- [off](contracts_hoprchannels.hoprchannels.md#off)
- [on](contracts_hoprchannels.hoprchannels.md#on)
- [once](contracts_hoprchannels.hoprchannels.md#once)
- [queryFilter](contracts_hoprchannels.hoprchannels.md#queryfilter)
- [redeemTicket](contracts_hoprchannels.hoprchannels.md#redeemticket)
- [redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)](contracts_hoprchannels.hoprchannels.md#redeemticket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes))
- [removeAllListeners](contracts_hoprchannels.hoprchannels.md#removealllisteners)
- [removeListener](contracts_hoprchannels.hoprchannels.md#removelistener)
- [secsClosure](contracts_hoprchannels.hoprchannels.md#secsclosure)
- [secsClosure()](contracts_hoprchannels.hoprchannels.md#secsclosure())
- [token](contracts_hoprchannels.hoprchannels.md#token)
- [token()](contracts_hoprchannels.hoprchannels.md#token())
- [tokensReceived](contracts_hoprchannels.hoprchannels.md#tokensreceived)
- [tokensReceived(address,address,address,uint256,bytes,bytes)](contracts_hoprchannels.hoprchannels.md#tokensreceived(address,address,address,uint256,bytes,bytes))
- [getContractAddress](contracts_hoprchannels.hoprchannels.md#getcontractaddress)
- [getInterface](contracts_hoprchannels.hoprchannels.md#getinterface)
- [isIndexed](contracts_hoprchannels.hoprchannels.md#isindexed)

## Constructors

### constructor

\+ **new HoprChannels**(`addressOrName`: *string*, `contractInterface`: ContractInterface, `signerOrProvider?`: *Provider* \| *Signer*): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |
| `contractInterface` | ContractInterface |
| `signerOrProvider?` | *Provider* \| *Signer* |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Inherited from: Contract.constructor

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:98

## Properties

### \_deployedPromise

• **\_deployedPromise**: *Promise*<Contract\>

Inherited from: Contract.\_deployedPromise

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:92

___

### \_runningEvents

• **\_runningEvents**: *object*

#### Type declaration

Inherited from: Contract.\_runningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:93

___

### \_wrappedEmits

• **\_wrappedEmits**: *object*

#### Type declaration

Inherited from: Contract.\_wrappedEmits

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:96

___

### address

• `Readonly` **address**: *string*

Inherited from: Contract.address

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:71

___

### callStatic

• **callStatic**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `announce` | (`multiaddr`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `announce(bytes)` | (`multiaddr`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `bumpChannel` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<string\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<string\> |
| `channels` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\> |
| `channels(bytes32)` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\> |
| `computeChallenge` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<string\> |
| `computeChallenge(bytes32)` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<string\> |
| `finalizeChannelClosure` | (`counterparty`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `finalizeChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `fundChannelMulti` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: CallOverrides) => *Promise*<void\> |
| `initiateChannelClosure` | (`counterparty`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `initiateChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: CallOverrides) => *Promise*<void\> |
| `redeemTicket` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `secsClosure` | (`overrides?`: CallOverrides) => *Promise*<number\> |
| `secsClosure()` | (`overrides?`: CallOverrides) => *Promise*<number\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<string\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: CallOverrides) => *Promise*<void\> |

Overrides: Contract.callStatic

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:619

___

### deployTransaction

• `Readonly` **deployTransaction**: TransactionResponse

Inherited from: Contract.deployTransaction

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:91

___

### estimateGas

• **estimateGas**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `announce` | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `announce(bytes)` | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `bumpChannel` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `channels` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `channels(bytes32)` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `computeChallenge` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `computeChallenge(bytes32)` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `finalizeChannelClosure` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `finalizeChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `fundChannelMulti` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `initiateChannelClosure` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `initiateChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `redeemTicket` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `secsClosure` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `secsClosure()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<BigNumber\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<BigNumber\> |

Overrides: Contract.estimateGas

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:905

___

### filters

• **filters**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `Announcement` | (`account`: *string*, `multiaddr`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*], { `account`: *string* ; `multiaddr`: *string*  }\> |
| `ChannelUpdate` | (`partyA`: *string*, `partyB`: *string*, `newState`: ``null``) => [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<[*string*, *string*, [*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }], { `newState`: [*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  } ; `partyA`: *string* ; `partyB`: *string*  }\> |

Overrides: Contract.filters

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:826

___

### functions

• **functions**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: CallOverrides) => *Promise*<[*BigNumber*]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `announce` | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `announce(bytes)` | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `bumpChannel` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `channels` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\> |
| `channels(bytes32)` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\> |
| `computeChallenge` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `computeChallenge(bytes32)` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `finalizeChannelClosure` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `finalizeChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `fundChannelMulti` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `initiateChannelClosure` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `initiateChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `redeemTicket` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `secsClosure` | (`overrides?`: CallOverrides) => *Promise*<[*number*]\> |
| `secsClosure()` | (`overrides?`: CallOverrides) => *Promise*<[*number*]\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<[*string*]\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<ContractTransaction\> |

Overrides: Contract.functions

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:197

___

### interface

• **interface**: [*HoprChannelsInterface*](../interfaces/contracts_hoprchannels.hoprchannelsinterface.md)

Overrides: Contract.interface

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:195

___

### populateTransaction

• **populateTransaction**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `FUND_CHANNEL_MULTI_SIZE` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `FUND_CHANNEL_MULTI_SIZE()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `TOKENS_RECIPIENT_INTERFACE_HASH()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `announce` | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `announce(bytes)` | (`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `bumpChannel` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `bumpChannel(address,bytes32)` | (`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `canImplementInterfaceForAddress` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `canImplementInterfaceForAddress(bytes32,address)` | (`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `channels` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `channels(bytes32)` | (`arg0`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `computeChallenge` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `computeChallenge(bytes32)` | (`response`: BytesLike, `overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `finalizeChannelClosure` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `finalizeChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `fundChannelMulti` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `fundChannelMulti(address,address,uint256,uint256)` | (`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `initiateChannelClosure` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `initiateChannelClosure(address)` | (`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `redeemTicket` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)` | (`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `secsClosure` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `secsClosure()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `token` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `token()` | (`overrides?`: CallOverrides) => *Promise*<PopulatedTransaction\> |
| `tokensReceived` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |
| `tokensReceived(address,address,address,uint256,bytes,bytes)` | (`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }) => *Promise*<PopulatedTransaction\> |

Overrides: Contract.populateTransaction

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:1058

___

### provider

• `Readonly` **provider**: *Provider*

Inherited from: Contract.provider

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:74

___

### resolvedAddress

• `Readonly` **resolvedAddress**: *Promise*<string\>

Inherited from: Contract.resolvedAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:90

___

### signer

• `Readonly` **signer**: *Signer*

Inherited from: Contract.signer

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:73

## Methods

### FUND\_CHANNEL\_MULTI\_SIZE

▸ **FUND_CHANNEL_MULTI_SIZE**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:411

___

### FUND\_CHANNEL\_MULTI\_SIZE()

▸ **FUND_CHANNEL_MULTI_SIZE()**(`overrides?`: CallOverrides): *Promise*<BigNumber\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<BigNumber\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:411

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH

▸ **TOKENS_RECIPIENT_INTERFACE_HASH**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:415

___

### TOKENS\_RECIPIENT\_INTERFACE\_HASH()

▸ **TOKENS_RECIPIENT_INTERFACE_HASH()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:415

___

### \_checkRunningEvents

▸ **_checkRunningEvents**(`runningEvent`: *RunningEvent*): *void*

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | *RunningEvent* |

**Returns:** *void*

Inherited from: Contract.\_checkRunningEvents

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:113

___

### \_deployed

▸ **_deployed**(`blockTag?`: BlockTag): *Promise*<Contract\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `blockTag?` | BlockTag |

**Returns:** *Promise*<Contract\>

Inherited from: Contract.\_deployed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:106

___

### \_wrapEvent

▸ **_wrapEvent**(`runningEvent`: *RunningEvent*, `log`: Log, `listener`: Listener): Event

#### Parameters

| Name | Type |
| :------ | :------ |
| `runningEvent` | *RunningEvent* |
| `log` | Log |
| `listener` | Listener |

**Returns:** Event

Inherited from: Contract.\_wrapEvent

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:114

___

### announce

▸ **announce**(`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:421

___

### announce(bytes)

▸ **announce(bytes)**(`multiaddr`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `multiaddr` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:424

___

### attach

▸ **attach**(`addressOrName`: *string*): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `addressOrName` | *string* |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.attach

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:156

___

### bumpChannel

▸ **bumpChannel**(`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `newCommitment` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:431

___

### bumpChannel(address,bytes32)

▸ **bumpChannel(address,bytes32)**(`counterparty`: *string*, `newCommitment`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `newCommitment` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:435

___

### canImplementInterfaceForAddress

▸ **canImplementInterfaceForAddress**(`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | BytesLike |
| `account` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:443

___

### canImplementInterfaceForAddress(bytes32,address)

▸ **canImplementInterfaceForAddress(bytes32,address)**(`interfaceHash`: BytesLike, `account`: *string*, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `interfaceHash` | BytesLike |
| `account` | *string* |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:447

___

### channels

▸ **channels**(`arg0`: BytesLike, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:455

___

### channels(bytes32)

▸ **channels(bytes32)**(`arg0`: BytesLike, `overrides?`: CallOverrides): *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `arg0` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<[*BigNumber*, *BigNumber*, *string*, *string*, *BigNumber*, *BigNumber*, *BigNumber*, *BigNumber*, *number*, *BigNumber*, *number*, *boolean*] & { `channelEpoch`: *BigNumber* ; `closureByPartyA`: *boolean* ; `closureTime`: *number* ; `partyABalance`: *BigNumber* ; `partyACommitment`: *string* ; `partyATicketEpoch`: *BigNumber* ; `partyATicketIndex`: *BigNumber* ; `partyBBalance`: *BigNumber* ; `partyBCommitment`: *string* ; `partyBTicketEpoch`: *BigNumber* ; `partyBTicketIndex`: *BigNumber* ; `status`: *number*  }\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:486

___

### computeChallenge

▸ **computeChallenge**(`response`: BytesLike, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `response` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:521

___

### computeChallenge(bytes32)

▸ **computeChallenge(bytes32)**(`response`: BytesLike, `overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `response` | BytesLike |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:524

___

### connect

▸ **connect**(`signerOrProvider`: *string* \| *Provider* \| *Signer*): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signerOrProvider` | *string* \| *Provider* \| *Signer* |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.connect

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:155

___

### deployed

▸ **deployed**(): *Promise*<[*HoprChannels*](contracts_hoprchannels.hoprchannels.md)\>

**Returns:** *Promise*<[*HoprChannels*](contracts_hoprchannels.hoprchannels.md)\>

Overrides: Contract.deployed

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:157

___

### emit

▸ **emit**(`eventName`: *string* \| EventFilter, ...`args`: *any*[]): *boolean*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* \| EventFilter |
| `...args` | *any*[] |

**Returns:** *boolean*

Inherited from: Contract.emit

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:119

___

### fallback

▸ **fallback**(`overrides?`: TransactionRequest): *Promise*<TransactionResponse\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | TransactionRequest |

**Returns:** *Promise*<TransactionResponse\>

Inherited from: Contract.fallback

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:107

___

### finalizeChannelClosure

▸ **finalizeChannelClosure**(`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:531

___

### finalizeChannelClosure(address)

▸ **finalizeChannelClosure(address)**(`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:534

___

### fundChannelMulti

▸ **fundChannelMulti**(`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | *string* |
| `account2` | *string* |
| `amount1` | BigNumberish |
| `amount2` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:541

___

### fundChannelMulti(address,address,uint256,uint256)

▸ **fundChannelMulti(address,address,uint256,uint256)**(`account1`: *string*, `account2`: *string*, `amount1`: BigNumberish, `amount2`: BigNumberish, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `account1` | *string* |
| `account2` | *string* |
| `amount1` | BigNumberish |
| `amount2` | BigNumberish |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:547

___

### initiateChannelClosure

▸ **initiateChannelClosure**(`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:557

___

### initiateChannelClosure(address)

▸ **initiateChannelClosure(address)**(`counterparty`: *string*, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:560

___

### listenerCount

▸ **listenerCount**(`eventName?`: *string* \| EventFilter): *number*

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* \| EventFilter |

**Returns:** *number*

Inherited from: Contract.listenerCount

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:120

___

### listeners

▸ **listeners**<EventArgsArray, EventArgsObject\>(`eventFilter?`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter?` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:159

▸ **listeners**(`eventName?`: *string*): Listener[]

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** Listener[]

Overrides: Contract.listeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:182

___

### off

▸ **off**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:162

▸ **off**(`eventName`: *string*, `listener`: Listener): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.off

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:183

___

### on

▸ **on**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:166

▸ **on**(`eventName`: *string*, `listener`: Listener): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.on

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:184

___

### once

▸ **once**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:170

▸ **once**(`eventName`: *string*, `listener`: Listener): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.once

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:185

___

### queryFilter

▸ **queryFilter**<EventArgsArray, EventArgsObject\>(`event`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `fromBlockOrBlockhash?`: *string* \| *number*, `toBlock?`: *string* \| *number*): *Promise*<[*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `event` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `fromBlockOrBlockhash?` | *string* \| *number* |
| `toBlock?` | *string* \| *number* |

**Returns:** *Promise*<[*TypedEvent*](../interfaces/contracts_commons.typedevent.md)<EventArgsArray & EventArgsObject\>[]\>

Overrides: Contract.queryFilter

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:189

___

### redeemTicket

▸ **redeemTicket**(`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `nextCommitment` | BytesLike |
| `ticketEpoch` | BigNumberish |
| `ticketIndex` | BigNumberish |
| `proofOfRelaySecret` | BytesLike |
| `amount` | BigNumberish |
| `winProb` | BigNumberish |
| `signature` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:567

___

### redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)

▸ **redeemTicket(address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)**(`counterparty`: *string*, `nextCommitment`: BytesLike, `ticketEpoch`: BigNumberish, `ticketIndex`: BigNumberish, `proofOfRelaySecret`: BytesLike, `amount`: BigNumberish, `winProb`: BigNumberish, `signature`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `counterparty` | *string* |
| `nextCommitment` | BytesLike |
| `ticketEpoch` | BigNumberish |
| `ticketIndex` | BigNumberish |
| `proofOfRelaySecret` | BytesLike |
| `amount` | BigNumberish |
| `winProb` | BigNumberish |
| `signature` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:577

___

### removeAllListeners

▸ **removeAllListeners**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:178

▸ **removeAllListeners**(`eventName?`: *string*): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName?` | *string* |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeAllListeners

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:187

___

### removeListener

▸ **removeListener**<EventArgsArray, EventArgsObject\>(`eventFilter`: [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\>, `listener`: [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\>): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Type parameters

| Name | Type |
| :------ | :------ |
| `EventArgsArray` | *any*[] |
| `EventArgsObject` | - |

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventFilter` | [*TypedEventFilter*](../interfaces/contracts_commons.typedeventfilter.md)<EventArgsArray, EventArgsObject\> |
| `listener` | [*TypedListener*](../modules/contracts_commons.md#typedlistener)<EventArgsArray, EventArgsObject\> |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:174

▸ **removeListener**(`eventName`: *string*, `listener`: Listener): [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `eventName` | *string* |
| `listener` | Listener |

**Returns:** [*HoprChannels*](contracts_hoprchannels.hoprchannels.md)

Overrides: Contract.removeListener

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:186

___

### secsClosure

▸ **secsClosure**(`overrides?`: CallOverrides): *Promise*<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<number\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:591

___

### secsClosure()

▸ **secsClosure()**(`overrides?`: CallOverrides): *Promise*<number\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<number\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:591

___

### token

▸ **token**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:595

___

### token()

▸ **token()**(`overrides?`: CallOverrides): *Promise*<string\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `overrides?` | CallOverrides |

**Returns:** *Promise*<string\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:595

___

### tokensReceived

▸ **tokensReceived**(`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `from` | *string* |
| `to` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:599

___

### tokensReceived(address,address,address,uint256,bytes,bytes)

▸ **tokensReceived(address,address,address,uint256,bytes,bytes)**(`operator`: *string*, `from`: *string*, `to`: *string*, `amount`: BigNumberish, `userData`: BytesLike, `operatorData`: BytesLike, `overrides?`: Overrides & { `from?`: *string* \| *Promise*<string\>  }): *Promise*<ContractTransaction\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `operator` | *string* |
| `from` | *string* |
| `to` | *string* |
| `amount` | BigNumberish |
| `userData` | BytesLike |
| `operatorData` | BytesLike |
| `overrides?` | Overrides & { `from?`: *string* \| *Promise*<string\>  } |

**Returns:** *Promise*<ContractTransaction\>

Defined in: packages/core-ethereum/src/contracts/HoprChannels.d.ts:607

___

### getContractAddress

▸ `Static` **getContractAddress**(`transaction`: { `from`: *string* ; `nonce`: BigNumberish  }): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `transaction` | *object* |
| `transaction.from` | *string* |
| `transaction.nonce` | BigNumberish |

**Returns:** *string*

Inherited from: Contract.getContractAddress

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:100

___

### getInterface

▸ `Static` **getInterface**(`contractInterface`: ContractInterface): *Interface*

#### Parameters

| Name | Type |
| :------ | :------ |
| `contractInterface` | ContractInterface |

**Returns:** *Interface*

Inherited from: Contract.getInterface

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:104

___

### isIndexed

▸ `Static` **isIndexed**(`value`: *any*): value is Indexed

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | *any* |

**Returns:** value is Indexed

Inherited from: Contract.isIndexed

Defined in: node_modules/@ethersproject/contracts/lib/index.d.ts:110
