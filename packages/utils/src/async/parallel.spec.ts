import { nAtATime } from './parallel'
import assert from 'assert'

function testFunction(timeout: number, result: number, throwException: boolean): Promise<number> {
  return new Promise<number>((resolve, reject) =>
    setTimeout(
      // Make sure the function call is indeed asynchronous
      setImmediate,
      timeout,
      throwException ? (val: number) => reject(Error(val.toString())) : resolve,
      result
    )
  )
}

describe('test nAtATime', function () {
  it('test no concurrency', async function () {
    const CALLS = 3
    const result = await nAtATime(
      testFunction,
      Array.from({ length: CALLS }, (_, index) => [20, index, false]),
      1
    )

    assert(result.length == CALLS, `must contain all results`)
    assert(
      result.every((value, index) => Number.isInteger(value) && value == index),
      `must contain the right results`
    )
  })

  it('test concurrency', async function () {
    const CALLS = 3
    const result = await nAtATime(
      testFunction,
      Array.from({ length: CALLS }, (_, index) => [20 * index, index, false]),
      2
    )

    assert(result.length == CALLS, `must contain all results`)
    assert(
      result.every((value, index) => Number.isInteger(value) && value == index),
      `must contain the right results`
    )
  })

  it('test concurrency - inverse order', async function () {
    const CALLS = 3
    const result = await nAtATime(
      testFunction,
      Array.from({ length: CALLS }, (_, index) => [20 * (CALLS - index - 1), index, false]),
      2
    )

    assert(result.length == CALLS, `must contain all results`)
    assert(
      result.sort().every((value, index) => Number.isInteger(value) && value == index),
      `must contain the right results`
    )
  })

  it('test concurrency with exceptions', async function () {
    const CALLS = 1
    const result = await nAtATime(
      testFunction,
      Array.from({ length: CALLS }, (_, index) => [20 * index, index, true]),
      1
    )

    assert(result.length == 1, `must contain all results`)
    assert(typeof result[0] == 'object' && (result[0] as Error).message === '0', `must contain the right results`)
  })

  it('test concurrency - edge cases', async function () {
    // Must return immediately
    assert((await nAtATime(testFunction, [], Infinity)).length == 0)

    // Must return immediately
    assert((await nAtATime(testFunction, [[10e3, 0, false]], 0)).length == 0)
    assert((await nAtATime(testFunction, [[10e3, 0, false]], -1)).length == 0)
  })
})
