import { timeout } from './timeout.js'
import assert from 'assert'

describe('testing timeoutAfter', () => {
  it('resolve promise', async () => {
    let result = await timeout(100, () => Promise.resolve('ok'))
    assert(result === 'ok')
  })

  it('reject with timeout', async () => {
    await assert.rejects(async () => await timeout(100, () => new Promise<void>(() => {})), Error('Timeout'))
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
        ),
      undefined
    )
  })

  it('reject synchronously', async () => {
    await assert.rejects(
      async () =>
        await timeout(100, () => {
          throw Error('internal error')
        }),
      Error(`internal error`)
    )
  })
})
