import assert from 'assert'
import { wait, retryWithBackoffThenThrow as backoff, getBackoffRetries, getBackoffRetryTimeout } from './backoff.js'

// Additional time that Node.js takes to compute functions
// and process timeouts
const PROPAGATION_DELAY = 50

describe('test wait', function () {
  it('should resolve after 100 ms', async function () {
    const startTime = Date.now()
    const expectedWait = 100
    await wait(expectedWait)
    assert(Date.now() - startTime - expectedWait <= PROPAGATION_DELAY)
  })
})

describe('test backoff', function () {
  it('should validate options', async function () {
    const fn = () => Promise.resolve()

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
    let callCount = 0
    const fn = () => {
      callCount++
      ticks.push(Date.now())
      return Promise.reject()
    }

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
    assert(callCount == 5)
    assert(ticks[0] - ticks[1] - 100 <= PROPAGATION_DELAY)
    assert(ticks[1] - ticks[2] - 200 <= PROPAGATION_DELAY)
    assert(ticks[2] - ticks[3] - 400 <= PROPAGATION_DELAY)
    assert(ticks[3] - ticks[4] - 800 <= PROPAGATION_DELAY)
  })

  it('should timeout after computed retries', async function () {
    this.timeout(3000)
    const start = Date.now()

    let callCount = 0

    const fn = () => {
      callCount++
      return Promise.reject()
    }

    const minDelay = 100
    const maxDelay = 500
    const delayMultiple = 2
    let timedout = false
    try {
      await backoff(fn, {
        minDelay,
        maxDelay,
        delayMultiple
      })
    } catch {
      timedout = true
    }

    assert(
      getBackoffRetries(minDelay, maxDelay, delayMultiple) == callCount - 1,
      'Expected call count must be equal to computed call count'
    )

    const PROPAGATION_DELAY = 50
    assert(Date.now() - start - getBackoffRetryTimeout(minDelay, maxDelay, delayMultiple) <= PROPAGATION_DELAY)
    assert(timedout)
  })

  it('should resolve after 4th try', async function () {
    const ticks: number[] = []
    let callCount = 0

    const fn = () => {
      ticks.push(Date.now())
      callCount++
      if (callCount == 4) {
        return Promise.resolve()
      }

      return Promise.reject()
    }

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
    assert(callCount == 4)
    assert(ticks[0] - ticks[1] - 100 <= PROPAGATION_DELAY)
    assert(ticks[1] - ticks[2] - 200 <= PROPAGATION_DELAY)
    assert(ticks[2] - ticks[3] - 400 <= PROPAGATION_DELAY)
  })
})
