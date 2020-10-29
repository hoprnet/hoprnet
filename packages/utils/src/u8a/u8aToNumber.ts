/**
 * Converts a Uint8Array to number.
 * @param arr Uint8Array to convert to number
 */
function u8aToNumber(arr: Uint8Array): number {
  const arrLength = arr.length

  if (arrLength == 0) {
    return 0
  }

  let result = 0
  for (let i = arrLength; i > 0; i--) {
    result |= arr[i - 1] << ((arrLength - i) * 8)
  }

  return result
}

export {u8aToNumber}
