/**
 * Adds the contents of two arrays together while ignoring the final overflow.
 * Computes a + b mod ( 2^(8 * a.length) - 1)
 *
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
    result.set([tmp], offset - 1)
  }

  return result
}
