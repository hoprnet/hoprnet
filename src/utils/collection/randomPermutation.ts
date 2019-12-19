import { randomBytes } from 'crypto'
import BN from 'bn.js'

import randomInteger from '../base/randomInteger'

/**
 * Return a random permutation of the given @param array
 * by using the (optimized) Fisher-Yates shuffling algorithm.
 *
 * @param  {Array} array the array to permutate
 */
export default function randomPermutation<T>(array: T[]): T[] {
    if (array.length <= 1) {
        return array
    }

    let j: number
    let tmp: T

    const byteAmount: number = Math.max(Math.ceil(Math.log2(array.length)) / 8, 1)

    for (let i = array.length - 1; i > 0; i--) {
        j = randomInteger(0, i + 1)
        tmp = array[i]
        array[i] = array[j]
        array[j] = tmp
    }

    return array
}
