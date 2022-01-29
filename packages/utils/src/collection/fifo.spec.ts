import assert from 'assert'

import { FIFO } from './fifo'

describe('test fifo', function () {
  it('entry insertion and extraction', function () {
    const queue = FIFO<number>()

    const items = [1, 2, 3, 4]

    let length = 0
    for (const item of items) {
      queue.push(item)
      assert(queue.size() == ++length)
    }

    assert(length == items.length)
    assert(queue.size() == items.length)

    for (const item of items) {
      assert(queue.shift() == item)
      assert(queue.size() == --length)
    }

    assert(length == 0)
    assert(queue.size() == 0)
  })

  it('entry insertion and extraction', function () {
    const queue = FIFO<number>()

    assert(queue.shift() == undefined)

    assert(queue.size() == 0)
  })
})
