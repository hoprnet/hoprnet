import { stringToU8a, toU8a, u8aAdd, u8aEquals } from '../u8a/index.js'
import { iterateHash, recoverIteratedHash } from './hashIterator.js'
import assert from 'assert'
import { Hash } from '../types.js'

import { iterate_hash, recover_iterated_hash } from '../../../core/lib/core_types.js'

describe('test hash iterator', function () {
  const HASH_LENGTH = 4
  const ONE = new Uint8Array(HASH_LENGTH)
  ONE[HASH_LENGTH - 1] = 1
  const hashFunc = (arr: Uint8Array): Uint8Array => u8aAdd(false, arr, ONE)

  const STEPSIZE = 5
  const MAX_ITERATIONS = 4 * STEPSIZE + 2
  const LENGTH = 4

  const hint = (i: number): Promise<Uint8Array> => {
    if (i % STEPSIZE == 0) {
      return Promise.resolve(toU8a(i, LENGTH))
    }
  }
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
  it('correspondence of iterated hash & recovery', async function () {
    let seed = new Uint8Array(16)
    let hashFn = (msg: Uint8Array) => Hash.create([msg]).serialize().slice(0, Hash.size())
    let TS_iterated = await iterateHash(seed, hashFn, 1000, 10)
    let RS_iterated = iterate_hash(seed, 1000, 10)

    assert(u8aEquals(TS_iterated.hash, RS_iterated.hash()))
    assert.equal(TS_iterated.intermediates.length, RS_iterated.count_intermediates())

    for (let i = 0; i < RS_iterated.count_intermediates(); i++) {
      assert.equal(TS_iterated.intermediates[i].iteration, RS_iterated.intermediate(i).iteration)
      assert(u8aEquals(TS_iterated.intermediates[i].preImage, RS_iterated.intermediate(i).intermediate))
    }

    let RS_hint = RS_iterated.intermediate(98)
    assert.equal(RS_hint.iteration, 980)
    assert(
      u8aEquals(RS_hint.intermediate, stringToU8a('a380d145d8612d33912494f1b36571c0b59b9bd459e6bb7d5ea05946be4c256b'))
    )

    let target_idx = 988
    let target_hash = stringToU8a('614eeebc22e8a79cbcac8bb6ba140768dd4bee4017460ad941de72f0fd5610e3')

    let TS_recovered = await recoverIteratedHash(
      target_hash,
      hashFn,
      async (i) => (i == RS_hint.iteration ? RS_hint.intermediate : undefined),
      1000,
      10,
      undefined
    )
    assert(TS_recovered != undefined)

    let TS_hint = TS_iterated.intermediates[98]
    assert.equal(TS_hint.iteration, 980)
    let RS_recovered = recover_iterated_hash(
      target_hash,
      (i: number) => (i == TS_hint.iteration ? TS_hint.preImage : undefined),
      1000,
      10,
      undefined
    )
    assert(RS_recovered != undefined)

    assert.equal(TS_recovered.iteration, RS_recovered.iteration)
    assert(u8aEquals(TS_recovered.preImage, RS_recovered.intermediate))
    assert.equal(target_idx, RS_recovered.iteration)
  })
})
