[@hoprnet/hopr-real](README.md) / Exports

# @hoprnet/hopr-real

## Table of contents

### Functions

- [dummy\_get\_one](modules.md#dummy_get_one)
- [read\_file](modules.md#read_file)
- [write\_file](modules.md#write_file)

## Functions

### dummy\_get\_one

▸ **dummy_get_one**(): `string`

Dummy function to test WASM.

#### Returns

`string`

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
