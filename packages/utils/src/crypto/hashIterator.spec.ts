import { toU8a, u8aAdd} from '../u8a'
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
    const hashes = await iterateHash(new Uint8Array(HASH_LENGTH).fill(0x00), hashFunc, MAX_ITERATIONS)
    assert.equal(hashes.length, MAX_ITERATIONS)
    assert.deepStrictEqual(
      hashes[hashes.length - 1],
      toU8a(MAX_ITERATIONS, HASH_LENGTH),
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
  })
})
