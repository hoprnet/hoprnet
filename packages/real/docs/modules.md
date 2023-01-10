[@hoprnet/hopr-real](README.md) / Exports

# @hoprnet/hopr-real

## Table of contents

### Functions

- [coerce\_version](modules.md#coerce_version)
- [dummy\_get\_one](modules.md#dummy_get_one)
- [read\_file](modules.md#read_file)
- [satisfies](modules.md#satisfies)
- [write\_file](modules.md#write_file)

## Functions

### coerce\_version

▸ **coerce_version**(`version`, `options?`): `string`

Wrapper for semver `semver.coerce`

Coerces a string to SemVer if possible

#### Parameters

| Name | Type |
| :------ | :------ |
| `version` | `string` \| `number` \| `SemVer` |
| `options?` | `CoerceOptions` |

#### Returns

`string`

#### Defined in

[src/semver.ts:13](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/semver.ts#L13)

___

### dummy\_get\_one

▸ **dummy_get_one**(): `string`

Dummy function to test WASM.

#### Returns

`string`

#### Defined in

lib/real_base.d.ts:7

___

### read\_file

▸ **read_file**(`file`): `Uint8Array`

Wrapper for reading file via WASM

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `file` | `string` | File path |

#### Returns

`Uint8Array`

#### Defined in

[src/io.ts:9](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/io.ts#L9)

___

### satisfies

▸ **satisfies**(`version`, `range`, `optionsOrLoose?`): `boolean`

Wrapper for `semver.satisfies`

Return true if the version satisfies the range.

#### Parameters

| Name | Type |
| :------ | :------ |
| `version` | `string` \| `SemVer` |
| `range` | `string` \| `Range` |
| `optionsOrLoose?` | `boolean` \| `RangeOptions` |

#### Returns

`boolean`

#### Defined in

[src/semver.ts:26](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/semver.ts#L26)

___

### write\_file

▸ **write_file**(`file`, `data`): `void`

Wrapper for reading file via WASM.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `file` | `string` | File path |
| `data` | `Uint8Array` | Data to write to the file |

#### Returns

`void`

#### Defined in

[src/io.ts:18](https://github.com/hoprnet/hoprnet/blob/master/packages/real/src/io.ts#L18)
