import { limitConcurrency } from './promise-pool'
import assert from 'assert'


describe('testing promise pool', () => {
  it('limitConcurrency should be similar to promise.all below a threshold', async () => {
    let resolveA
    let resolveB
    let a: Promise<number> = new Promise((r) => {resolveA = r})
    let b: Promise<number> = new Promise((r) => {resolveB = r})
    let promises = [a, b, Promise.resolve(3)]
    let retProm = limitConcurrency<number>(5, () => promises.length == 0, () => promises.shift())
    resolveA(1)
    resolveB(2)
    let ret = await retProm
    assert(ret.length === 3)
    assert(ret[0] == 1)
    assert(ret[1] == 2)
    assert(ret[2] == 3)
  })

  it('limitConcurrency should run a maximum of maxConcurrency promises at once', async() => {
    let totalPromises = 5
    let maxConcurrency = 3
    let resolvers = []
    let promises = []
    for (let i = 0; i < totalPromises; i++){
      promises.push(new Promise((r) => {
        resolvers.push(r)
      }))
    }
    let retProm = limitConcurrency(maxConcurrency, () => promises.length == 0, () => promises.shift())
    assert(promises.length === totalPromises - maxConcurrency)
    for (let resolve of resolvers){
      resolve()
    }
    assert((await retProm).length === totalPromises)
  })

  it('should act like Promise.all - rejections cause exit', async() => {
    let resolveA
    let rejectB
    let a: Promise<number> = new Promise((r) => {resolveA = r})
    let b: Promise<number> = new Promise((_, r) => {rejectB = r})
    let promises = [a, b, Promise.resolve(3)]
    let retProm = limitConcurrency<number>(1, () => promises.length == 0, () => promises.shift())
    resolveA(1)
    rejectB()
    let thrown = false
    try { 
      await retProm
    } catch (e) {
      thrown = true
    }
    assert(thrown)
  })

})
