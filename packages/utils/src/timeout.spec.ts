import { timeoutAfter } from './timeout'
import assert from 'assert'

describe('testing timeoutAfter', () => {
  it('if promise resolves first, it is no-op', async() => {
    let p = timeoutAfter(() => Promise.resolve('ok'), 100)
    assert(await p == 'ok')
  })

  it('rejects if promise does not resolve', async() => {
    let p = timeoutAfter(() => new Promise(() => {}), 2)
    let threw = false
    try {
      await p
    } catch (e){
      threw = true
    }
    assert(threw)
  })

})
