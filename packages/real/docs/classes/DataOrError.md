[@hoprnet/hopr-real](../README.md) / [Exports](../modules.md) / DataOrError

# Class: DataOrError

Class wrapping Uint8Array or error.

## Hierarchy

- `XOrError`<`Uint8Array`\>

  ↳ **`DataOrError`**

## Table of contents

### Constructors

- [constructor](DataOrError.md#constructor)

### Accessors

- [data](DataOrError.md#data)
- [error](DataOrError.md#error)

### Methods

- [hasError](DataOrError.md#haserror)

## Constructors

### constructor

• **new DataOrError**()

#### Inherited from

XOrError<Uint8Array\>.constructor

## Accessors

### data

• `get` **data**(): `X`

#### Returns

`X`

#### Inherited from

XOrError.data

#### Defined in

[src/io.ts:16](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/io.ts#L16)

• `set` **data**(`val`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `val` | `X` |

#### Returns

`void`

#### Inherited from

XOrError.data

#### Defined in

[src/io.ts:20](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/io.ts#L20)

___

### error

• `get` **error**(): `any`

#### Returns

`any`

#### Inherited from

XOrError.error

#### Defined in

[src/io.ts:25](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/io.ts#L25)

• `set` **error**(`val`): `void`

#### Parameters

| Name | Type |
| :------ | :------ |
| `val` | `any` |

#### Returns

`void`

#### Inherited from

XOrError.error

#### Defined in

[src/io.ts:29](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/io.ts#L29)

## Methods

### hasError

▸ **hasError**(): `boolean`

#### Returns

`boolean`

#### Inherited from

XOrError.hasError

#### Defined in

[src/io.ts:34](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/io.ts#L34)
