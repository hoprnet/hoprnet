import assert from 'assert'
import LevelUp from 'levelup'
import MemDown from 'memdown'
import { Commitment } from './commitment'
import sinon from 'sinon'

describe('test commitment', function () {
  let fakeSet, fakeGet, fakeDB, fakeId
  beforeEach(async function () {
    fakeSet = sinon.fake.resolves(true)
    fakeGet = sinon.fake.resolves(undefined)
    fakeDB = new LevelUp(MemDown())
    fakeId = 'test'
  })

  it('should publish a hashed secret', async function () {
    let cm = new Commitment(fakeSet, fakeGet, fakeDB, fakeId)
    let c1 = await cm.getCurrentCommitment()
    assert(c1, 'gives current commitment')
    assert(fakeGet.callcount == 1)
    assert(fakeSet.calledWith(c1))
    await cm.bumpCommitment()
    let c2 = await cm.getCurrentCommitment()
    assert(fakeSet.calledWith(c2))
    assert(fakeSet.callCount == 2)
    assert(c2, 'gives current commitment')
    assert(c2.hash().eq(c1))

    //
    let cm2 = new Commitment(fakeSet, fakeGet, fakeDB, fakeId)
    let c3 = await cm2.getCurrentCommitment()
    assert(c2.eq(c3), 'Repeated initializations should return the same')
  })
})
