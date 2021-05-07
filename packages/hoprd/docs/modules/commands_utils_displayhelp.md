[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/utils/displayHelp

# Module: commands/utils/displayHelp

## Table of contents

### Variables

- [CHALK_STRINGS](commands_utils_displayhelp.md#chalk_strings)

### Functions

- [displayHelp](commands_utils_displayhelp.md#displayhelp)
- [getOptions](commands_utils_displayhelp.md#getoptions)
- [getPaddingLength](commands_utils_displayhelp.md#getpaddinglength)
- [styleValue](commands_utils_displayhelp.md#stylevalue)

## Variables

### CHALK_STRINGS

• `Const` **CHALK_STRINGS**: _object_

#### Type declaration

| Name  | Type     |
| :---- | :------- |
| `no`  | _string_ |
| `yes` | _string_ |

Defined in: [commands/utils/displayHelp.ts:78](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L78)

## Functions

### displayHelp

▸ **displayHelp**(): _void_

**Returns:** _void_

Defined in: [commands/utils/displayHelp.ts:6](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L6)

---

### getOptions

▸ **getOptions**(`options`: { `description?`: _string_ ; `value`: _any_ }[], `style?`: `"compact"` \| `"vertical"`): _string_[]

#### Parameters

| Name      | Type                                            | Default value |
| :-------- | :---------------------------------------------- | :------------ |
| `options` | { `description?`: _string_ ; `value`: _any_ }[] | -             |
| `style`   | `"compact"` \| `"vertical"`                     | 'compact'     |

**Returns:** _string_[]

Defined in: [commands/utils/displayHelp.ts:87](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L87)

---

### getPaddingLength

▸ **getPaddingLength**(`items`: _string_[], `addExtraPadding?`: _boolean_): _number_

#### Parameters

| Name              | Type       | Default value |
| :---------------- | :--------- | :------------ |
| `items`           | _string_[] | -             |
| `addExtraPadding` | _boolean_  | true          |

**Returns:** _number_

Defined in: [commands/utils/displayHelp.ts:74](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L74)

---

### styleValue

▸ **styleValue**(`value`: _any_, `_type?`: _any_): _string_

#### Parameters

| Name     | Type  |
| :------- | :---- |
| `value`  | _any_ |
| `_type?` | _any_ |

**Returns:** _string_

Defined in: [commands/utils/displayHelp.ts:83](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/displayHelp.ts#L83)
