[@hoprnet/hopr-utils](../README.md) / [Exports](../modules.md) / SimpleTimer

# Class: SimpleTimer

Currently the SimpleTimer is NOT a wrapper for HistogramTimer,
but rather implements the timer logic using js_sys::Date to achieve a similar functionality.
This is because WASM does not support system time functionality from the Rust stdlib.

## Table of contents

### Constructors

- [constructor](SimpleTimer.md#constructor)

### Methods

- [free](SimpleTimer.md#free)

## Constructors

### constructor

• **new SimpleTimer**()

## Methods

### free

▸ **free**(): `void`

#### Returns

`void`
