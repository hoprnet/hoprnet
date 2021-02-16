import { toU8a, u8aAdd, u8aEquals } from '../u8a'
import { iterateHash, recoverIteratedHash } from './hashIterator'
import assert from 'assert'

describe('test hash iterator', function () {
  const HASH_LENGTH = 4
  const ONE = new Uint8Array(HASH_LENGTH)
  ONE[HASH_LENGTH - 1] = 1
  const hashFunc = (arr: Uint8Array) => Promise.resolve(u8aAdd(false, arr, ONE))

  const STEPSIZE = 5
  const MAX_ITERATIONS = 4 * STEPSIZE + 2
  const LENGTH = 4

  const hint = (i: number) => Promise.resolve(toU8a(i, LENGTH))

  it('should iterate', async function () {
    const expected = {
      hash: toU8a(MAX_ITERATIONS, LENGTH),
      intermediates: [
        { iteration: 0, preImage: toU8a(0, LENGTH) },
        { iteration: 1 * STEPSIZE, preImage: toU8a(1 * STEPSIZE, LENGTH) },
        { iteration: 2 * STEPSIZE, preImage: toU8a(2 * STEPSIZE, LENGTH) },
        { iteration: 3 * STEPSIZE, preImage: toU8a(3 * STEPSIZE, LENGTH) },
        { iteration: 4 * STEPSIZE, preImage: toU8a(4 * STEPSIZE, LENGTH) }
      ]
    }

    assert.deepStrictEqual(
      expected,
      await iterateHash(new Uint8Array(HASH_LENGTH).fill(0x00), hashFunc, MAX_ITERATIONS, STEPSIZE)
    )

    for (let i = 1; i <= MAX_ITERATIONS; i++) {
      assert.deepStrictEqual(
        {
          preImage: toU8a(i - 1, LENGTH),
          iteration: i - 1
        },
        await recoverIteratedHash(toU8a(i, LENGTH), hashFunc, hint, MAX_ITERATIONS, STEPSIZE)
      )
    }

    assert(
      u8aEquals(
        (await iterateHash(undefined, hashFunc, MAX_ITERATIONS, STEPSIZE, hint)).hash,
        toU8a(MAX_ITERATIONS, LENGTH)
      )
    )
  })
})
