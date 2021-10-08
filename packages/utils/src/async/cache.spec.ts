import assert from 'assert'
import sinon from 'sinon'
import { cacheNoArgAsyncFunction } from '.'

describe('cache', function () {
  let clock: sinon.SinonFakeTimers

  before(function () {
    clock = sinon.useFakeTimers()
  })

  after(function () {
    clock.restore()
  })

  it('should get when nothing is cached', async function () {
    const funcMock = sinon.fake.resolves(10)
    const cachedFunction = cacheNoArgAsyncFunction<number>(funcMock, 1000)
    const result = cachedFunction()
    assert.equal(await result, 10)
    assert.equal(funcMock.callCount, 1)
  })

  it('should get when something is cached', async function () {
    const funcMock = sinon.fake.resolves(10)
    const cachedFunction = cacheNoArgAsyncFunction<number>(funcMock, 1000)
    const result = cachedFunction()
    assert.equal(await result, 10)
    const result2 = cachedFunction()
    assert.equal(await result2, 10)
    assert.equal(funcMock.callCount, 1)
  })

  it('should get when cache expires', async function () {
    const funcMock = sinon.fake.resolves(10)
    const cachedFunction = cacheNoArgAsyncFunction<number>(funcMock, 1000)
    const result = cachedFunction()
    assert.equal(await result, 10)
    clock.tick(1001)
    const result2 = cachedFunction()
    assert.equal(await result2, 10)
    assert.equal(funcMock.callCount, 2)
  })
})
