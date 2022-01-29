import assert from 'assert'
import { oneAtATime } from './concurrency'

describe('concurrency', function () {
  it('one at a time', async function () {
    const limitter = oneAtATime<void>()

    const results: number[] = []

    const promises = []

    const CALLS = 3

    for (let i = 0; i < CALLS; i++) {
      limitter(async () => {
        const promise = new Promise<number>((resolve) => setTimeout(resolve, 50, i))
        promises.push(promise)
        results.push(await promise)
      })
    }

    await Promise.all(promises)

    assert(results.length == CALLS, `must contain all results`)
    assert(
      results.every((value: number, index: number) => value == index),
      `must contain the right results`
    )
  })

  it('one at a time - restart', async function () {
    const limitter = oneAtATime<void>()

    const results: number[] = []

    let promise: Promise<number>
    limitter(async () => {
      promise = new Promise<number>((resolve) => setTimeout(setImmediate, 20, resolve, 0))
      results.push(await promise)
    })

    await promise

    assert(results.length == 1, `must contain one result`)
    assert(results[0] == 0)

    const promises: Promise<number>[] = []

    const CALLS = 3

    for (let i = 0; i < CALLS; i++) {
      limitter(async () => {
        const promise = new Promise<number>((resolve) => setTimeout(setImmediate, 20, resolve, i + 1))
        promises.push(promise)
        results.push(await promise)
      })
    }

    await Promise.all(promises)

    // @ts-ignore
    assert(results.length == 4, `must contain all results`)
    assert(
      results.every((value: number, index: number) => value == index),
      `must contain the right results`
    )
  })

  it('one at a time - asynchronous errors', async function () {
    const limitter = oneAtATime<void>()

    const results: number[] = []

    const promises = []

    const CALLS = 3

    for (let i = 0; i < CALLS; i++) {
      limitter(async () => {
        const promise = new Promise<number>((_, reject) => setTimeout(setImmediate, 20, reject, i))
        promises.push(promise)
        results.push(await promise)
      })
    }

    assert.rejects(async () => await Promise.all(promises))

    assert(results.length == 0, `must contain no results`)
  })

  it('one at a time - synchronous errors', async function () {
    const limitter = oneAtATime<void>()

    const results: number[] = []

    const promises = []

    const CALLS = 3

    for (let i = 0; i < CALLS; i++) {
      limitter(async () => {
        throw Error(`i`)
      })
    }

    assert.rejects(async () => await Promise.all(promises))

    assert(results.length == 0, `must contain no results`)
  })
})
