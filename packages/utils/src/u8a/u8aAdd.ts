/**
 * Adds the contents of two arrays together while ignoring the final overflow.
 * Computes `a + b % ( 2 ** (8 * a.length) - 1)`
 *
 * @example
 * u8aAdd(false, new Uint8Array([1], new Uint8Array([2])) // Uint8Array([3])
 * u8aAdd(false, new Uint8Array([1], new Uint8Array([255])) // Uint8Array([0])
 * u8aAdd(false, new Uint8Array([0, 1], new Uint8Array([0, 255])) // Uint8Array([1, 0])
 * @param inplace result is stored in a if set to true
 * @param a first array
 * @param b second array
 */
export function u8aAdd(inplace: boolean, a: Uint8Array, b: Uint8Array): Uint8Array {
  let result = inplace ? a : new Uint8Array(a.length)

  let overflow = 0
  let tmp: number

  for (let offset = a.length; offset > 0; offset--) {
    tmp = a[offset - 1] + b[offset - 1] + overflow
    overflow = tmp >> 8
    result[offset - 1] = tmp
  }

  return result
}
