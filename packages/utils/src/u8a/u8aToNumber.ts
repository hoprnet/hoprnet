/**
 * Converts a Uint8Array to number.
 * @param arr Uint8Array to convert to number
 */
function u8aToNumber(arr: Uint8Array): number | bigint {
  const arrLength = arr.length

  if (arrLength == 0) {
    return 0
  }

  if (arrLength == 1) {
    return arr[0]
  }

  if (arrLength < 4) {
    let _arr = new Uint8Array(4)
    _arr.set(arr, 4 - arr.length)

    return new DataView(_arr.buffer, _arr.byteOffset).getUint32(0)
  }

  if (arrLength == 4) {
    return new DataView(arr.buffer, arr.byteOffset).getUint32(0)
  }

  if (arrLength < 8) {
    let _arr = new Uint8Array(8)
    _arr.set(arr, 4 - arr.length)

    return new DataView(_arr.buffer, _arr.byteOffset).getBigUint64(0)
  }

  if (arrLength == 8) {
    return new DataView(arr.buffer, arr.byteOffset).getBigUint64(0)
  }

  throw Error(`Array has too many elements. Can only extract up to 8 elements, got ${arrLength}`)
}

export { u8aToNumber }
