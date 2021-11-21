import crypto from 'crypto'

const BIT_WIDTH = 64n

function nextRandomFullWidth(): bigint {
  return crypto.randomBytes(Number(BIT_WIDTH / 8n)).readBigUInt64LE()
}

/**
 * Maximum random integer that can be generated using randomInteger function.
 */
export const MAX_RANDOM_INTEGER = (1n << BIT_WIDTH) - 1n

/**
 * Internal function generating random integer in half-close interval [0, bound).
 * Uses an optimized Lemire's method (https://arxiv.org/abs/1805.10941)
 * as devised in https://github.com/apple/swift/pull/39143 by Stepen Canon.
 *
 * @param bound Maximum number that can be generated.
 */
function randomBoundedInteger(bound: number): number {
  let bnBound = BigInt(bound) > MAX_RANDOM_INTEGER ? MAX_RANDOM_INTEGER : BigInt(bound)

  let uboundTwosComplement = ~bnBound + 1n + MAX_RANDOM_INTEGER

  let res = bnBound * nextRandomFullWidth()
  let resHi = res >> BIT_WIDTH

  // Fast-out
  if ((res & MAX_RANDOM_INTEGER) <= uboundTwosComplement) return Number(resHi)

  let newRnd = (bnBound * nextRandomFullWidth()) >> BIT_WIDTH
  let carry = ((resHi + newRnd) >> BIT_WIDTH) & 1n

  return Number(resHi + carry)
}

/**
 * Returns a random value between `start` and `end`.
 * @example
 * ```
 * randomInteger(3) // result in { 0, 1, 2}
 * randomInteger(0, 3) // result in { 0, 1, 2 }
 * randomInteger(7, 9) // result in { 7, 8 }
 * ```
 * @param start start of the interval (inclusive)
 * @param end end of the interval (not inclusive)
 * @returns random number between @param start and @param end
 */
export function randomInteger(start: number, end?: number): number {
  if (!end) {
    end = start
    start = 0
  }

  if (end <= start || start < 0) throw Error('invalid range')

  return start + randomBoundedInteger(end - start)
}

export function randomChoice<T>(collection: T[]): T {
  if (collection.length == undefined || collection.length == 0) {
    throw new Error('empty collection, cannot choose random element')
  }
  return collection[randomInteger(0, collection.length)]
}
