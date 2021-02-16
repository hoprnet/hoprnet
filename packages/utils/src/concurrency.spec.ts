import assert from 'assert'
import { oneAtATime } from './concurrency'

describe('concurrency', function () {
  it('one at a time', async function () {
    let resolveP1, p1Started = false, resolveP2, p2Started = false, out = 0
    const p1 = () => new Promise<void>(resolve => {
      p1Started = true
      out = 1
      resolveP1 = resolve
    })
    const p2 = () => new Promise<void>(resolve => {
      p2Started = true
      out = 2
      resolveP2 = resolve
    })
    const o = oneAtATime()
    let p1prom = o(p1)
    let p2prom = o(p2)
    await setImmediate(() => {})
    assert(p1Started)
    assert(!p2Started)
    assert(p1Started)
    assert(!p2Started)
    resolveP1()
    await p1prom
    assert(p1Started)
    assert(p2Started)
    assert(out == 2)
    resolveP2()
    await p2prom
    assert(out == 2)
  })
})
