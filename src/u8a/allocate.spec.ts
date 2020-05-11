import { u8aAllocate } from './allocate'
import { range } from 'ramda'
import { expect } from 'chai'

describe('test u8aAllocate spec', function () {
  const BUFFER_LENGTH = 10
  let page: ArrayBuffer
  this.beforeEach(() => {
    page = new ArrayBuffer(BUFFER_LENGTH)
  })

  it('should not throw an error if the added page is smaller than the given offset', () => {
    const offset = BUFFER_LENGTH / 2
    const arrayHalfPage = new Uint8Array(range(0, offset))
    expect(() => u8aAllocate({ page, offset }, arrayHalfPage)).to.not.throw()
  })
})
