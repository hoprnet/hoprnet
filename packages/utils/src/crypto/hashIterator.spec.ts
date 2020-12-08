import { toU8a, u8aAdd } from '../u8a'
import { iterateHash, recoverIteratedHash } from './hashIterator'
import assert from 'assert'

describe('test hash iterator', function () {
  const HASH_LENGTH = 4
  const ONE = new Uint8Array(HASH_LENGTH)
  ONE[HASH_LENGTH - 1] = 1
  const hashFunc = (arr: Uint8Array) => u8aAdd(false, arr, ONE)

  it('should iterate', async function () {
    const STEPSIZE = 5
    const MAX_ITERATIONS = 4 * STEPSIZE + 2
    const expected = {
      hash: toU8a(MAX_ITERATIONS, 4),
      intermediates: [
        { iteration: 0, preImage: toU8a(0, 4) },
        { iteration: 1 * STEPSIZE, preImage: toU8a(1 * STEPSIZE, 4) },
        { iteration: 2 * STEPSIZE, preImage: toU8a(2 * STEPSIZE, 4) },
        { iteration: 3 * STEPSIZE, preImage: toU8a(3 * STEPSIZE, 4) },
        { iteration: 4 * STEPSIZE, preImage: toU8a(4 * STEPSIZE, 4) }
      ]
    }

    assert.deepStrictEqual(expected, await iterateHash(new Uint8Array(HASH_LENGTH).fill(0x00), hashFunc, MAX_ITERATIONS, STEPSIZE))

    for (let i = 1; i <= MAX_ITERATIONS; i++) {
      assert.deepStrictEqual({
        preImage: toU8a(i - 1, 4),
        iteration: i - 1
      }, await recoverIteratedHash(toU8a(i, 4), hashFunc, (i: number) => {
        if (i % STEPSIZE == 0) {
          return toU8a(i, 4)
        }
      }, MAX_ITERATIONS, STEPSIZE))
      
    }
    // console.log()

  })
})
