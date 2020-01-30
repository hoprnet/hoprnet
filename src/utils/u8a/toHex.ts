const ALPHABET = '0123456789abcdef'
/**
 * Converts a Uint8Array to a hex string.
 * @notice Mainly used for debugging.
 * @param arr Uint8Array
 * @param prefixed if `true` add a `0x` in the beginning
 */
export function u8aToHex(arr: Uint8Array, prefixed: boolean = true) {
  return arr.reduce((acc: string, value: number) => (acc += ALPHABET[value >> 4] + ALPHABET[value & 15]), prefixed ? '0x' : '')
}
