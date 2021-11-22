import crypto from 'crypto'

/**
 * Maximum bit length of a random number.
 */
const BIT_WIDTH = 64n

/**
 * Internal function generating fix length random integer.
 */
function nextRandomFullWidth(): bigint {
  return crypto.randomBytes(Number(BIT_WIDTH / 8n)).readBigUInt64LE()
}

/**
 * Maximum random integer that can be generated using randomInteger function.
 */
export const MAX_RANDOM_INTEGER = (1n << BIT_WIDTH) - 1n

/**
 * Internal function generating random integer in { 0,1,2,... bound-1 }.
 * Uses an optimized Lemire's method (https://arxiv.org/abs/1805.10941)
 * as devised in https://github.com/apple/swift/pull/39143 by Stepen Canon.
 *
 * NOTE: The maximum bit length of a number that can be generated using this function
 * is fixed to 64 bits.
 * @param bound Maximum number that can be generated.
 */
function randomBoundedInteger(bound: number): number {

  /* -- Here's the original explanation of the optimized method by Stephen Canon: --
   *
   * Everyone knows that generating an unbiased random integer in a range
   * 0 ..< upperBound, where upperBound is not a power of two, requires
   * rejection sampling. What if I told you that Big Random Number has
   * lied to us for decades, and we have been played for absolute fools?
   *
   * Previously Swift used Lemire's "nearly divisionless" method
   * (https://arxiv.org/abs/1805.10941) for this operation. We instead
   * now use a novel method that:
   *
   * - never divides
   * - avoids rejection sampling entirely
   * - achieves a theoretically optimal bound on the amount of randomness
   *   consumed to generate a sample
   * - delivers actual performance improvements for most real cases
   *
   * Lemire interprets each word from the random source as a fixed-point
   * number in [0, 1), multiplies by upperBound, and takes the floor. Up
   * to this point, this is the algorithm suggested by Knuth in TAoCP vol 2,
   * and as observed by Knuth, it is slightly biased. Lemire cleverly
   * corrects this bias via rejection sampling, which requires one division
   * in the general case (hence, "nearly divisionless").
   *
   * Our new algorithm takes a different approach. Rather than using
   * rejection sampling, we observe that the bias decreases exponentially
   * in the number of bits used for the computation. In the limit we are
   * interpreting the bitstream from the random source as a uniform real
   * number r in [0, 1) and ⌊r * upperBound⌋ provides an unbiased sample
   * in 0 ..< upperBound. The only challenge, then, is to know when we
   * have computed enough bits of the product to know what the result is.
   *
   * Observe that we can split the random stream at any bit position i,
   * yielding r = r₀ + r₁ with r₀ a fixed-point number in [0,1) and
   * 0 ≤ r₁ < 2⁻ⁱ. Further observe that:
   *
   *    result = ⌊r * upperBound⌋
   *           = ⌊r₀ * upperBound⌋ + ⌊frac(r₀*upperBound) + r₁*upperBound⌋
   *
   * The first term of this expression is Knuth's biased sample, which is
   * computed with just a full-width multiply.
   *
   * If i > log₂(upperBound), both summands in the second term are smaller
   * than 1, so the second term is either 0 or 1. Applying the bound on r₁,
   * we see that if frac(r₀ * upperBound) <= 1 - upperBound * 2⁻ⁱ, the
   * second term is necessarily zero, and the first term is the unbiased
   * result. Happily, this is _also_ a trivial computation on the low-order
   * part of the full-width multiply.
   *
   * If the test fails, we do not reject the sample, throwing away the bits
   * we have already consumed from the random source; instead we increase i
   * by a convenient amount, computing more bits of the product. This is the
   * criticial improvement; while Lemire has a probability of 1/2 to reject
   * for each word consumed in the worst case, we have a probability of
   * terminating of 1/2 for each _bit_ consumed. This reduces the worst-case
   * expected number of random bits required from O(log₂(upperBound)) to
   * log₂(upperBound) + O(1), which is optimal[1].
   *
   * Of more practical interest, this new algorithm opens an intriguing
   * possibility: we can compute just 64 extra bits, and have a probability
   * of 1 - 2⁻⁶⁴ of terminating. This is so close to certainty that we can
   * simply stop without introducing any measurable bias (detecting any
   * difference would require about 2¹²⁸ samples, which is prohibitive).
   * This is a significant performance improvement for slow random
   * generators, since it asymptotically reduces the number of bits
   * required by a factor of two for bignums, while matching or reducing
   * the expected number of bits required for smaller numbers. This is the
   * algorithm implemented below (the formally-uniform method is not
   * much more complex to implement and is only a little bit slower, but
   * there's no reason to do so).
   *
   * More intriguing still, this algorithm can be made unconditional by
   * removing the early out, so that every value computed requires word
   * size + 64 bits from the stream, which breaks the loop-carried
   * dependency for fast generators, unlocking vectorization and
   * parallelization where it was previously impossible. This is an
   * especially powerful advantage when paired with bitstream generators
   * that allow skip-ahead such as newer counter-based generators used
   * in simulations and ML.
   *
   * Note that it is _possible_ to employ Lemire's tighter early-out
   * check that involves a division with this algorithm as well; this
   * is beneficial in some cases when upperBound is a constant and the
   * generator is slow, but we do not think it necessary with the new
   * algorithm and other planned improvements.
   *
   * [1] We can actually achieve log₂(upperBound) + ε for any ε > 0 by
   * generating multiple random samples at once, but that is only of
   * theoretical interest--it is still interesting, however, since I
   * don't think anyone has described how to attain it previously.
  */

  const bnBound = BigInt(bound) > MAX_RANDOM_INTEGER ? MAX_RANDOM_INTEGER : BigInt(bound)

  // 2's complement
  const uboundNeg = MAX_RANDOM_INTEGER - bnBound + 1n

  const res = bnBound * nextRandomFullWidth()
  const resHi = res >> BIT_WIDTH

  // Early-out
  if ((res & MAX_RANDOM_INTEGER) <= uboundNeg) return Number(resHi)

  const newRnd = (bnBound * nextRandomFullWidth()) >> BIT_WIDTH
  const carry = ((resHi + newRnd) >> BIT_WIDTH) & 1n

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
 * The maximum number generated by this function is MAX_RANDOM_INTEGER.
 * @param start start of the interval (inclusive). Must be non-negative.
 * @param end end of the interval (not inclusive). Must not exceed MAX_RANDOM_INTEGER.
 * @returns random number between @param start and @param end
 */
export function randomInteger(start: number, end?: number): number {
  if (!end) {
    end = start
    start = 0
  }

  if (end <= start || start < 0 || end > MAX_RANDOM_INTEGER) throw Error('invalid range')

  return start + randomBoundedInteger(end - start)
}

export function randomChoice<T>(collection: T[]): T {
  if (collection.length == undefined || collection.length == 0) {
    throw new Error('empty collection, cannot choose random element')
  }
  return collection[randomInteger(0, collection.length)]
}
