import { toU8a, u8aAdd } from '../u8a'
import { iterateHash, recoverIteratedHash } from './hashIterator'
import assert from 'assert'

const HASH_LENGTH = 4
const ONE = toU8a(1, HASH_LENGTH)
const mockHash = (arr: Uint8Array) => Promise.resolve(u8aAdd(false, arr, ONE))

describe('test hash iterator', function () {
  const STEPSIZE = 5
  const MAX_ITERATIONS = 4 * STEPSIZE + 2
  const LENGTH = 4
  const hint = (i: number) => Promise.resolve(toU8a(i, LENGTH))

  it('should iterate', async function () {
    const hashes = await iterateHash(new Uint8Array(HASH_LENGTH).fill(0x00), mockHash, MAX_ITERATIONS)
    assert.equal(hashes.length, MAX_ITERATIONS + 1) // Including source
    assert.deepStrictEqual(hashes[hashes.length - 1], toU8a(MAX_ITERATIONS, HASH_LENGTH))
  })

  it('should be able to recover hashes in blocks', async function () {
    for (let i = 1; i <= MAX_ITERATIONS; i++) {
      const expected = toU8a(i - 1, LENGTH)
      assert.deepStrictEqual(
        expected,
        await recoverIteratedHash(toU8a(i, LENGTH), mockHash, hint, MAX_ITERATIONS, STEPSIZE)
      )
    }
  })
})
