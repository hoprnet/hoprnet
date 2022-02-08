import assert from 'assert'
import { oneAtATime } from './concurrency'

describe('concurrency', function () {
  it('one at a time', async function () {
    const limitter = oneAtATime<void>()

    const results: number[] = []

    const promises = []

    const CALLS = 3

    for (let i = 0; i < CALLS; i++) {
      const promise = new Promise<void>((resolve) =>
        setTimeout(
          (result) => {
            results.push(result)
            resolve()
          },
          50,
          i
        )
      )
      promises.push(promise)
      limitter(() => promise)
    }

    await Promise.all(promises)

    assert(results.length == CALLS, `must contain all results instead of just ${results.length}`)
    assert(
      results.every((value: number, index: number) => value == index),
      `must contain the right results`
    )
  })

  it('one at a time - restart', async function () {
    const limitter = oneAtATime<void>()

    const results: number[] = []

    let promise = new Promise<void>((resolve) =>
      setTimeout(
        (result) => {
          results.push(result)
          resolve()
        },
        20,
        0
      )
    )
    limitter(() => promise)

    await promise

    assert(results.length == 1, `must contain one result instead of ${results.length}`)
    assert(results[0] == 0)

    const promises = []

    const CALLS = 3

    for (let i = 0; i < CALLS; i++) {
      promise = new Promise<void>((resolve) =>
        setTimeout(
          (result) => {
            results.push(result)
            resolve()
          },
          20,
          i + 1
        )
      )
      promises.push(promise)
      limitter(() => promise)
    }

    await Promise.all(promises)

    // @ts-ignore
    assert(results.length == 4, `must contain all results instead of just ${results.length}`)
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
      const promise = new Promise<void>((_, reject) =>
        setTimeout(
          (result) => {
            results.push(result)
            reject()
          },
          20,
          i + 1
        )
      )
      promises.push(promise)
      limitter(() => promise)
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
