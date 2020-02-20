// Copyright 2017-2019 @polkadot/util authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

/**
 * @name u8aConcat
 * @summary Creates a concatenated Uint8Array from the inputs.
 * @description
 * Concatenates the input arrays into a single `UInt8Array`.
 *
 * @example
 * ```javascript
 *
 * u8aConcat(
 *   new Uint8Array([1, 2, 3]),
 *   new Uint8Array([4, 5, 6])
 * ); // [1, 2, 3, 4, 5, 6]
 * ```
 */
export function u8aConcat(...list: Uint8Array[]): Uint8Array {
  let totalLength = 0

  const listLength = list.length
  for (let i = 0; i < listLength; i++) {
    totalLength += list[i].length
  }

  const result = new Uint8Array(totalLength)
  let offset = 0

  for (let i = 0; i < listLength; i++) {
    result.set(list[i], offset)
    offset += list[i].length
  }

  return result
}
