import assert from 'assert'
import LevelUp from 'levelup'
import MemDown from 'memdown'
import { Commitment } from './commitment'
import sinon from 'sinon'
import { Hash } from './types'

describe('test commitment', function () {
  let fakeSet, fakeGet, fakeDB, fakeId
  beforeEach(async function () {
    fakeSet = sinon.fake.resolves(true)
    fakeGet = sinon.fake.resolves(undefined)
    fakeDB = new LevelUp(MemDown())
    fakeId = new Hash(new Uint8Array([1]))
  })

  it('should publish a hashed secret', async function () {
    let cm = new Commitment(fakeSet, fakeGet, fakeDB, fakeId)
    let c1 = await cm.getCurrentCommitment()
    assert(c1 != null, 'gives current commitment')
    assert.equal(fakeGet.callCount, 1, 'should look on chain')
    assert(fakeSet.calledWith(c1), 'should set a new commitment on chain')
    await cm.bumpCommitment()
    let c2 = await cm.getCurrentCommitment()
    assert(c2, 'gives current commitment')
    assert(c2.hash().eq(c1), 'c2 is commitment of c1')
    //
    fakeGet = () => Promise.resolve(c2)
    let cm2 = new Commitment(fakeSet, fakeGet, fakeDB, fakeId)
    let c3 = await cm2.getCurrentCommitment()
    assert(c2.eq(c3), 'Repeated initializations should return the same')
  })
})
