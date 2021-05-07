[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/utils/displayHelp

# Module: commands/utils/displayHelp

## Table of contents

### Variables

- [CHALK\_STRINGS](commands_utils_displayhelp.md#chalk_strings)

### Functions

- [displayHelp](commands_utils_displayhelp.md#displayhelp)
- [getOptions](commands_utils_displayhelp.md#getoptions)
- [getPaddingLength](commands_utils_displayhelp.md#getpaddinglength)
- [styleValue](commands_utils_displayhelp.md#stylevalue)

## Variables

### CHALK\_STRINGS

• `Const` **CHALK\_STRINGS**: *object*

#### Type declaration

| Name | Type |
| :------ | :------ |
| `no` | *string* |
| `yes` | *string* |

Defined in: [commands/utils/displayHelp.ts:78](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L78)

## Functions

### displayHelp

▸ **displayHelp**(): *void*

**Returns:** *void*

Defined in: [commands/utils/displayHelp.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L6)

___

### getOptions

▸ **getOptions**(`options`: { `description?`: *string* ; `value`: *any*  }[], `style?`: ``"compact"`` \| ``"vertical"``): *string*[]

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `options` | { `description?`: *string* ; `value`: *any*  }[] | - |
| `style` | ``"compact"`` \| ``"vertical"`` | 'compact' |

**Returns:** *string*[]

Defined in: [commands/utils/displayHelp.ts:87](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L87)

___

### getPaddingLength

▸ **getPaddingLength**(`items`: *string*[], `addExtraPadding?`: *boolean*): *number*

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `items` | *string*[] | - |
| `addExtraPadding` | *boolean* | true |

**Returns:** *number*

Defined in: [commands/utils/displayHelp.ts:74](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L74)

___

### styleValue

▸ **styleValue**(`value`: *any*, `_type?`: *any*): *string*

#### Parameters

| Name | Type |
| :------ | :------ |
| `value` | *any* |
| `_type?` | *any* |

**Returns:** *string*

Defined in: [commands/utils/displayHelp.ts:83](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L83)
