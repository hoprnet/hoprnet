import { timeout } from './timeout'
import assert from 'assert'

describe('testing timeoutAfter', () => {
  it('resolve promise', async () => {
    let result = await timeout(100, () => Promise.resolve('ok'))
    assert(result === 'ok')
  })

  it('reject with timeout', async () => {
    await assert.rejects(async () => await timeout(100, () => new Promise<void>(() => {})))
  })

  it('reject asynchronously', async () => {
    await assert.rejects(
      async () =>
        await timeout(
          100,
          () =>
            new Promise((_, reject) => {
              setTimeout(reject, 50)
            })
        )
    )
  })

  it('reject synchronously', async () => {
    await assert.rejects(
      async () =>
        await timeout(100, () => {
          throw Error()
        })
    )
  })
})
