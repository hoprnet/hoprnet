[@hoprnet/hopr-core](../README.md) / [Exports](../modules.md) / HoprOptions

# Class: HoprOptions

## Table of contents

### Constructors

- [constructor](HoprOptions.md#constructor)

### Properties

- [announce](HoprOptions.md#announce)
- [announceLocalAddresses](HoprOptions.md#announcelocaladdresses)
- [connector](HoprOptions.md#connector)
- [createDbIfNotExist](HoprOptions.md#createdbifnotexist)
- [dbPath](HoprOptions.md#dbpath)
- [environment](HoprOptions.md#environment)
- [forceCreateDB](HoprOptions.md#forcecreatedb)
- [hosts](HoprOptions.md#hosts)
- [password](HoprOptions.md#password)
- [preferLocalAddresses](HoprOptions.md#preferlocaladdresses)
- [strategy](HoprOptions.md#strategy)

## Constructors

### constructor

• **new HoprOptions**(`environment`, `announce?`, `dbPath?`, `createDbIfNotExist?`, `forceCreateDB?`, `password?`, `connector?`, `strategy?`, `hosts?`, `announceLocalAddresses?`, `preferLocalAddresses?`)

#### Parameters

| Name | Type |
| :------ | :------ |
| `environment` | [`ResolvedEnvironment`](../modules.md#resolvedenvironment) |
| `announce?` | `boolean` |
| `dbPath?` | `string` |
| `createDbIfNotExist?` | `boolean` |
| `forceCreateDB?` | `boolean` |
| `password?` | `string` |
| `connector?` | `default` |
| `strategy?` | `ChannelStrategy` |
| `hosts?` | `Object` |
| `hosts.ip4?` | `NetOptions` |
| `hosts.ip6?` | `NetOptions` |
| `announceLocalAddresses?` | `boolean` |
| `preferLocalAddresses?` | `boolean` |

#### Defined in

[packages/core/src/index.ts:75](https://github.com/hoprnet/hoprnet/blob/master/packages/core/src/index.ts#L75)

## Properties

### announce

• `Optional` **announce**: `boolean`

___

### announceLocalAddresses

• `Optional` **announceLocalAddresses**: `boolean`

___

### connector

• `Optional` **connector**: `default`

___

### createDbIfNotExist

• `Optional` **createDbIfNotExist**: `boolean`

___

### dbPath

• `Optional` **dbPath**: `string`

___

### environment

• **environment**: [`ResolvedEnvironment`](../modules.md#resolvedenvironment)

___

### forceCreateDB

• `Optional` **forceCreateDB**: `boolean`

___

### hosts

• `Optional` **hosts**: `Object`

#### Type declaration

| Name | Type |
| :------ | :------ |
| `ip4?` | `NetOptions` |
| `ip6?` | `NetOptions` |

___

### password

• `Optional` **password**: `string`

___

### preferLocalAddresses

• `Optional` **preferLocalAddresses**: `boolean`

___

### strategy

• `Optional` **strategy**: `ChannelStrategy`
