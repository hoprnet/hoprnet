import assert from 'assert'
import sinon from 'sinon'
import { wait, retryWithBackoff as backoff } from './backoff'

const getRoundedDiff = (startTime: number, endTime: number): number => {
  return Math.round((endTime - startTime) / 100) * 100
}

describe('test wait', function () {
  it('should resolve after 100 ms', async function () {
    const startTime = Date.now()
    await wait(100)
    const endTime = Date.now()
    const roundedDiff = getRoundedDiff(startTime, endTime)
    assert.strictEqual(roundedDiff, 100)
  })
})

describe('test backoff', function () {
  it('should validate options', async function () {
    const fn = sinon.fake(() => Promise.resolve())

    assert.rejects(
      backoff(fn, {
        minDelay: 10,
        maxDelay: 10
      }),
      'minDelay should be smaller than maxDelay'
    )

    assert.rejects(
      backoff(fn, {
        delayMultiple: 1
      }),
      'delayMultiple should be larger than 1'
    )
  })

  it('should timeout exponentially', async function () {
    this.timeout(3000)

    const ticks: number[] = []
    const fn = sinon.fake(() => {
      ticks.push(Date.now())
      return Promise.reject()
    })

    let timedout = false
    try {
      await backoff(fn, {
        minDelay: 100,
        maxDelay: 1000,
        delayMultiple: 2
      })
    } catch {
      timedout = true
    }

    assert(timedout)
    assert.strictEqual(fn.callCount, 6)
    assert.strictEqual(getRoundedDiff(ticks[0], ticks[1]), 100)
    assert.strictEqual(getRoundedDiff(ticks[1], ticks[2]), 200)
    assert.strictEqual(getRoundedDiff(ticks[2], ticks[3]), 400)
    assert.strictEqual(getRoundedDiff(ticks[3], ticks[4]), 800)
    assert.strictEqual(getRoundedDiff(ticks[4], ticks[5]), 1000)
  })

  it('should resolve after 4th try', async function () {
    const ticks: number[] = []
    const fn = sinon.fake(() => {
      ticks.push(Date.now())
      if (fn.callCount === 4) return Promise.resolve()
      return Promise.reject()
    })

    let timedout = false
    try {
      await backoff(fn, {
        minDelay: 100,
        maxDelay: 1000,
        delayMultiple: 2
      })
    } catch {
      timedout = true
    }

    assert(!timedout)
    assert.strictEqual(fn.callCount, 4)
    assert.strictEqual(getRoundedDiff(ticks[0], ticks[1]), 100)
    assert.strictEqual(getRoundedDiff(ticks[1], ticks[2]), 200)
    assert.strictEqual(getRoundedDiff(ticks[2], ticks[3]), 400)
  })
})
