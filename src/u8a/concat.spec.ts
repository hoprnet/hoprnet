import assert from 'assert'

import { u8aConcat } from './concat'
import { u8aEquals } from './equals'
import { timer, MAX_EXECUTION_TIME_FOR_CONCAT_IN_MS } from '../utils'

import { randomBytes } from 'crypto'

describe('test u8a concat', function () {
  const firstArray = randomBytes(43)
  const secondArray = randomBytes(31)
  const thirdArray = new Uint8Array()

  it('should always return 0 when undefined or same array is given', () => {
    assert(u8aConcat().length === 0)
    assert(u8aConcat(thirdArray).length == 0)
    assert(u8aConcat(thirdArray, thirdArray).length == 0)
    assert(u8aConcat(undefined).length == 0)
    assert(u8aConcat(undefined, thirdArray).length == 0)
  })

  it(`should run under ${MAX_EXECUTION_TIME_FOR_CONCAT_IN_MS}ms execution time`, () => {
    assert(timer(() => u8aConcat()) < MAX_EXECUTION_TIME_FOR_CONCAT_IN_MS)
  })

  it('should concat two Uint8Arrays', function () {
    let tmp: Uint8Array
    tmp = u8aConcat(undefined, firstArray, secondArray)
    assert(u8aEquals(tmp.subarray(0, 43), firstArray) && u8aEquals(tmp.subarray(43), secondArray))
    tmp = u8aConcat(firstArray, undefined, secondArray, thirdArray)
    assert(u8aEquals(tmp.subarray(0, 43), firstArray) && u8aEquals(tmp.subarray(43), secondArray))
  })

  it(`should concat two Uint8Arrays in under ${MAX_EXECUTION_TIME_FOR_CONCAT_IN_MS}ms`, function () {
    assert(timer(() => u8aConcat(undefined, firstArray, secondArray)) < MAX_EXECUTION_TIME_FOR_CONCAT_IN_MS)
  })
})
