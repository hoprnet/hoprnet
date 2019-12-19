import { randomBytes } from 'crypto'
import BN from 'bn.js'

import randomInteger from '../base/randomInteger'
import randomPermutation from './randomPermutation'

/**
 * Picks @param subsetSize elements at random from @param array .
 * The order of the picked elements does not coincide with their
 * order in @param array
 *
 * @param  {Array} array the array to pick the elements from
 * @param  {Number} subsetSize the requested size of the subset
 * @param  {Function} filter called with `(peerInfo)` and should return `true`
 * for every node that should be in the subset
 *
 * @returns {Array} array with at most @param subsetSize elements
 * that pass the test.
 *
 * @notice If less than @param subsetSize elements pass the test,
 * the result will contain less than @param subsetSize elements.
 */
export default function randomSubset<T>(array: T[], subsetSize: number, filter: (candidate: T) => boolean): T[] {
    if (subsetSize < 0) {
        throw Error(`Invalid input arguments. Please provide a positive subset size. Got '${subsetSize}' instead.`)
    }

    if (subsetSize > array.length) {
        throw Error(`Invalid subset size. Subset size must not be greater than set size.`)
    } 

    if (subsetSize <= 0) {
        return []
    }

    if (subsetSize === array.length) {
        // Returns a random permutation of all elements that pass
        // the test.
        return randomPermutation(array.filter(filter))
    }

    const byteAmount: number = Math.max(Math.ceil(Math.log2(array.length)) / 8, 1)

    if (subsetSize == 1) {
        let i = 0
        let index = randomInteger(0, array.length)
        
        while (!filter(array[index])) {
            if (i === array.length) {
                // There seems to be no element in the array
                // that passes the test.
                return []
            }
            i++
            index = (index + 1) % array.length
        }
        return [array[index]]
    }

    let notChosen = new Set<T>()
    let chosen = new Set<T>()
    let found: boolean
    let breakUp = false

    let index: number
    const arrayLength = array.length
    for (let i = 0; i < subsetSize && !breakUp; i++) {
        index = randomInteger(0, )
        
        
        new BN(randomBytes(byteAmount)).umod(new BN(arrayLength)).addn(index).toNumber()

        found = false

        do {
            while (chosen.has(index) || notChosen.has(index)) {
                index = (index + 1) % array.length
            }

            if (!filter(array[index])) {
                notChosen.add(index)
                index = (index + 1) % array.length
                found = false
            } else {
                chosen.add(index)
                found = true
            }

            if (notChosen.size + chosen.size == array.length && chosen.size < subsetSize) {
                breakUp = true
                break
            }
        } while (!found)
    }

    const result = []
    for (let index of chosen) {
        result.push(array[index])
    }

    return result
}