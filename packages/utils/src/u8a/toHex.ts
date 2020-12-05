const ALPHABET = '0123456789abcdef'
/**
 * Converts a Uint8Array to a hex string.
 * @notice Mainly used for debugging.
 * @param arr Uint8Array
 * @param prefixed if `true` add a `0x` in the beginning
 */
export function u8aToHex(arr?: Uint8Array, prefixed: boolean = true): string {
  let result = prefixed ? '0x' : ''

  if (arr == undefined || arr.length == 0) {
    return result
  }
  const arrLength = arr.length

  for (let i = 0; i < arrLength; i++) {
    result += ALPHABET[arr[i] >> 4]
    result += ALPHABET[arr[i] & 15]
  }

  return result
}
