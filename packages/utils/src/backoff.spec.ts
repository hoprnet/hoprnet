import assert from 'assert'
import sinon from 'sinon'
import { wait, backoff } from './backoff'

describe('test wait', function () {
  it('should resolve after 100 ms', async function () {
    await wait(100)
  })
})

// TODO: add more tests
describe('test backoff', function () {
  it('should timeout after 10 ms', async function () {
    const fn = sinon.fake(() => Promise.reject())

    let timedout = false
    try {
      await backoff(fn, {
        minDelay: 1,
        maxDelay: 10,
        delayMultiple: 2
      })
    } catch {
      timedout = true
    }

    assert(timedout)
    assert.strictEqual(fn.callCount, 4)
  })

  it('should resolve after 4th try', async function () {
    const fn = sinon.fake(() => {
      if (fn.callCount === 4) return Promise.resolve()
      return Promise.reject()
    })

    let timedout = false
    try {
      await backoff(fn, {
        minDelay: 1,
        maxDelay: 10,
        delayMultiple: 2
      })
    } catch {
      timedout = true
    }

    assert(!timedout)
    assert.strictEqual(fn.callCount, 4)
  })
})
