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
  if (list.length == 1) {
    return list[0] ? list[0].slice() : new Uint8Array()
  }
  let totalLength = 0

  const listLength = list.length
  for (let i = 0; i < listLength; i++) {
    const item = list[i]
    if (item == undefined || item.length == 0) {
      continue
    }

    totalLength += item.length
  }

  const result = new Uint8Array(totalLength)
  let offset = 0

  for (let i = 0; i < listLength; i++) {
    const item = list[i]
    if (item == undefined || item.length == 0) {
      continue
    }

    result.set(item, offset)
    offset += item.length
  }

  return result
}
