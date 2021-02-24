/**
 * Concatenates the input arrays into a single `UInt8Array`.
 *
 * @example
 * u8aConcat(
 *   new Uint8Array([1, 1, 1]),
 *   new Uint8Array([2, 2, 2])
 * ); // Uint8Arrau([1, 1, 1, 2, 2, 2])
 *  * u8aConcat(
 *   new Uint8Array([1, 1, 1]),
 *   undefined
 *   new Uint8Array([2, 2, 2])
 * ); // Uint8Arrau([1, 1, 1, 2, 2, 2])
 */
export function u8aConcat(...list: (Uint8Array | undefined)[]): Uint8Array {
  if (list == undefined || list.length == 0) {
    return new Uint8Array()
  }

  let totalLength = 0

  const listLength = list.length
  for (let i = 0; i < listLength; i++) {
    if (list[i] == undefined) {
      continue
    }

    totalLength += list[i].length
  }

  const result = new Uint8Array(totalLength)
  let offset = 0

  for (let i = 0; i < listLength; i++) {
    if (list[i] == undefined) {
      continue
    }

    if (list[i] !== undefined) {
      result.set(list[i], offset)
      offset += list[i].length
    }
  }

  return result
}
