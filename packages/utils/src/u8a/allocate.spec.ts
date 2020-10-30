import {u8aAllocate} from './allocate'
import {randomInteger} from '../randomInteger'

describe('test u8aAllocate spec', function () {
  const BUFFER_LENGTH = 10
  let page: ArrayBuffer
  this.beforeEach(() => {
    page = new ArrayBuffer(BUFFER_LENGTH)
  })

  it('should not throw an error if the added page is smaller than the given offset', () => {
    const offset = BUFFER_LENGTH / 2
    const arrayHalfPage = new Uint8Array(randomInteger(0, offset))

    u8aAllocate({page, offset}, arrayHalfPage)
  })
})
