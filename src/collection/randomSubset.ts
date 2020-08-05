import { randomPermutation } from './randomPermutation'
import { randomInteger } from '../randomInteger'

/**
 * Picks @param subsetSize elements at random from @param array .
 * The order of the picked elements does not coincide with their
 * order in @param array
 *
 * @param array the array to pick the elements from
 * @param subsetSize the requested size of the subset
 * @param filter called with `(peerInfo)` and should return `true`
 * for every node that should be in the subset
 *
 * @returns array with at most @param subsetSize elements
 * that pass the test.
 *
 * @notice If less than @param subsetSize elements pass the test,
 * the result will contain less than @param subsetSize elements.
 */
export function randomSubset<T>(array: T[], subsetSize: number, filter?: (candidate: T) => boolean): T[] {
  if (subsetSize < 0) {
    throw Error(`Invalid input arguments. Please provide a positive subset size. Got '${subsetSize}' instead.`)
  }

  if (subsetSize > array.length) {
    throw Error(`Invalid subset size. Subset size must not be greater than the array size.`)
  }

  if (subsetSize == 0) {
    return []
  }

  if (subsetSize == array.length) {
    // Returns a random permutation of all elements that pass
    // the test.
    return randomPermutation(filter != null ? array.filter(filter) : array)
  }

  if (subsetSize == 1) {
    let i = 0
    let index = randomInteger(0, array.length)

    while (filter != null && !filter(array[index])) {
      if (i == array.length) {
        // There seems to be no element in the array
        // that passes the test.
        return []
      }
      i++
      index = (index + 1) % array.length
    }
    return [array[index]]
  }

  let notChosen = new Set<number>()
  let chosen = new Set<number>()
  let found: boolean
  let breakUp = false
  let arrLength = array.length

  let index: number
  for (let i = 0; i < subsetSize && !breakUp; i++) {
    index = randomInteger(0, arrLength)

    found = false

    do {
      while (chosen.has(index) || notChosen.has(index)) {
        index = (index + 1) % arrLength
      }

      if (filter != null && !filter(array[index])) {
        notChosen.add(index)
        index = (index + 1) % arrLength
        found = false
      } else {
        chosen.add(index)
        found = true
      }

      if (notChosen.size + chosen.size == arrLength) {
        breakUp = true
        break
      }
    } while (!found)
  }

  const result: T[] = []
  for (let index of chosen) {
    result.push(array[index])
  }

  // Reshuffle items from (ordered) Set
  return result.length > 0 ? randomPermutation(result) : result
}
