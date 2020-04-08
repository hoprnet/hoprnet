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
export declare function randomPermutation<T>(array: T[]): T[];
