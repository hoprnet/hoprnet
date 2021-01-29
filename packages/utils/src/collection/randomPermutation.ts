import { randomInteger } from '../crypto/randomInteger'

/**
 * Return a random permutation of the given `array`
 * by using the (optimized) Fisher-Yates shuffling algorithm.
 *
 * @param array the array to permutate
 *
 * @example
 *
 * ```javascript
 * randomPermutation([1,2,3,4]);
 * // first run: [2,4,1,2]
 * // second run: [3,1,2,4]
 * // ...
 * ```
 */
export function randomPermutation<T>(array: T[]): T[] {
  if (array.length <= 1) {
    return array
  }

  let j: number
  let tmp: T

  for (let i = array.length - 1; i > 0; i--) {
    j = randomInteger(0, i + 1)
    tmp = array[i]
    array[i] = array[j]
    array[j] = tmp
  }

  return array
}
