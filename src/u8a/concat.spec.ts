import assert from 'assert'

import { u8aConcat } from './concat'
import { u8aEquals } from './equals'

import { randomBytes } from 'crypto'

describe('test u8a concat', function() {
  it('should concat two Uint8Arrays', function() {
      const firstArray = randomBytes(43)
      const secondArray = randomBytes(31)
      const thirdArray = new Uint8Array()
      
      assert(u8aConcat().length == 0)
      
      assert(u8aConcat(thirdArray).length == 0)

      assert(u8aConcat(thirdArray, thirdArray).length == 0)

      assert(u8aConcat(undefined).length == 0)

      assert(u8aConcat(undefined, thirdArray).length == 0)

      let tmp: Uint8Array

      tmp = u8aConcat(undefined, firstArray, secondArray)

      assert(u8aEquals(tmp.subarray(0,43), firstArray) && u8aEquals(tmp.subarray(43), secondArray))

      tmp = u8aConcat(firstArray, undefined, secondArray, thirdArray)

      assert(u8aEquals(tmp.subarray(0,43), firstArray) && u8aEquals(tmp.subarray(43), secondArray))
  })
})
