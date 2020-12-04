/**
 * Concatenates the input arrays into a single `UInt8Array`.
 *
 * @example
 * u8aConcat(
 *   new Uint8Array([1, 2, 3]),
 *   new Uint8Array([4, 5, 6])
 * ); // [1, 2, 3, 4, 5, 6]
 */
export function u8aConcat(...list: (Uint8Array | undefined)[]): Uint8Array {
  if (list == undefined || list.length == 0) {
    return new Uint8Array()
  }

  let totalLength = 0

  const listLength = list.length
  for (let i = 0; i < listLength; i++) {
    if (list[i] !== undefined) {
      // @ts-ignore
      totalLength += list[i].length
    }
  }

  const result = new Uint8Array(totalLength)
  let offset = 0

  for (let i = 0; i < listLength; i++) {
    if (list[i] !== undefined) {
      // @ts-ignore
      result.set(list[i], offset)
      // @ts-ignore
      offset += list[i].length
    }
  }

  return result
}
