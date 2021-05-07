[@hoprnet/hoprd](../README.md) / [Exports](../modules.md) / commands/utils/message

# Module: commands/utils/message

## Table of contents

### Functions

- [decodeMessage](commands_utils_message.md#decodemessage)
- [encodeMessage](commands_utils_message.md#encodemessage)

## Functions

### decodeMessage

▸ **decodeMessage**(`encoded`: Uint8Array): *object*

Tries to decode the message and returns the message as well as
the measured latency.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `encoded` | Uint8Array | an encoded message |

**Returns:** *object*

| Name | Type |
| :------ | :------ |
| `latency` | *number* |
| `msg` | *string* |

Defined in: [commands/utils/message.ts:18](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/message.ts#L18)

___

### encodeMessage

▸ **encodeMessage**(`msg`: *string*): Uint8Array

Adds the current timestamp to the message in order to measure the latency.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `msg` | *string* | the message |

**Returns:** Uint8Array

Defined in: [commands/utils/message.ts:9](https://github.com/hoprnet/hoprnet/blob/448a47a/packages/hoprd/src/commands/utils/message.ts#L9)
