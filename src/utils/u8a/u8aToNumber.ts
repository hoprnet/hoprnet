/**
 * Converts a Uint8Array to number.
 * @param arr Uint8Array to convert to number
 */
function u8aToNumber(arr: Uint8Array): number {
  if (arr.length == 1) {
    return arr[0]
  }

  return parseInt(
    arr.reduce((acc, n) => (acc += n.toString(16).padStart(2, '0')), ''),
    16
  )
}

export { u8aToNumber }
