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

  it('entry insertion and extraction - empty array', function () {
    const queue = FIFO<number>()

    assert(queue.shift() == undefined)

    assert(queue.size() == 0)
  })

  it('correct head', function () {
    const queue = FIFO<number>()

    assert(queue.peek() == undefined)
    queue.push(1)

    assert(queue.peek() == 1)

    queue.push(2)
    queue.shift()

    assert(queue.peek() == 2)
  })

  it('replace items', function () {
    const queue = FIFO<number>()

    assert(
      queue.replace(
        () => false,
        () => assert.fail()
      ) == false
    )

    queue.push(1)

    queue.replace(
      (item: number) => item == 1,
      () => 2
    )

    assert(queue.shift() == 2)

    for (let i = 0; i < 5; i++) {
      queue.push(i)
    }

    assert(
      queue.replace(
        (item: number) => item == 3,
        () => 5
      ) == true
    )

    const arr: number[] = []
    while (queue.size() > 0) {
      arr.push(queue.shift())
    }

    const requiredArr = [0, 1, 2, 5, 4]

    for (let i = 0; i < 5; i++) {
      assert(arr[i] == requiredArr[i])
    }
  })

  it('fifo to array', function () {
    const queue = FIFO<number>()

    assert(queue.toArray().length == 0)

    queue.push(0)

    let queueArray = queue.toArray()
    assert(queueArray.length == 1)
    assert(queueArray.every((value: number, index: number) => value == index))

    for (let i = 0; i < 5; i++) {
      queue.push(i + 1)
    }

    queueArray = queue.toArray()
    assert(queueArray.length == 6)
    assert(queueArray.every((value: number, index: number) => value == index))
  })
})
