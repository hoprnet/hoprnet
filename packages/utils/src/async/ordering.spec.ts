import { ordered } from './ordering'

import { setTimeout as setTimeoutPromise } from 'timers/promises'
import assert from 'assert'

describe(`check ordered`, function () {
  it('no element', async function () {
    const order = ordered<number>()

    setTimeout(order.end.bind(order), 50)
    for await (const _msg of order.iterator()) {
    }

    // Produces a timeout if not successful
  })
  it('single element', async function () {
    const order = ordered<number>()

    order.push({
      index: 1,
      value: 1
    })

    for await (const msg of order.iterator()) {
      assert(msg.value == 1)
      assert(msg.index == 1)
      order.end()
    }
  })

  it('correct order', async function () {
    const order = ordered<number>()

    // dispatch from main thread
    ;(async () => {
      for (let i = 0; i < 5; i++) {
        order.push({
          index: i,
          value: i
        })

        await setTimeoutPromise(50)
      }
      order.end()
    })()

    let i = 0
    for await (const msg of order.iterator()) {
      assert(msg.index == i)
      assert(msg.value == i)
      i++
    }

    assert(i == 5)
  })

  it('skip elements', async function () {
    const order = ordered<number>()

    // dispatch from main thread
    ;(async () => {
      for (let i = 0; i < 5; i++) {
        order.push({
          index: i * 2,
          value: i * 2
        })

        await setTimeoutPromise(50)
      }

      for (let i = 0; i < 5; i++) {
        try {
          order.push({
            index: i * 2 + 1,
            value: i * 2 + 1
          })
        } catch (err) {
          console.log(err)
        }

        await setTimeoutPromise(50)
      }

      order.end()
    })()

    let i = 0
    for await (const msg of order.iterator()) {
      assert(msg.index == i)
      assert(msg.value == i)
      i++
    }
    assert(i == 10)
  })
})
