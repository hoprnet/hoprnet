import assert from 'assert'
import { u8aConcat } from './concat'
import { u8aEquals } from './equals'
import { randomBytes } from 'crypto'

describe('test u8a concat', function () {
  const firstArray = randomBytes(43)
  const secondArray = randomBytes(31)
  const empty = new Uint8Array()

  it('should always return 0 when undefined', () => {
    assert(u8aConcat().length === 0)
    assert(u8aConcat(empty).length == 0)
    assert(u8aConcat(empty, empty).length == 0)
    assert(u8aConcat(undefined).length == 0)
    assert(u8aConcat(undefined, empty).length == 0)
  })

  it('should concat two Uint8Arrays', function () {
    let tmp: Uint8Array
    tmp = u8aConcat(undefined, firstArray, secondArray)
    assert(u8aEquals(tmp.subarray(0, 43), firstArray) && u8aEquals(tmp.subarray(43), secondArray))
    tmp = u8aConcat(firstArray, undefined, secondArray, empty)
    assert(u8aEquals(tmp.subarray(0, 43), firstArray) && u8aEquals(tmp.subarray(43), secondArray))
  })
})
