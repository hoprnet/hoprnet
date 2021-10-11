import assert from 'assert'
import { initializeCommitment, bumpCommitment, findCommitmentPreImage } from './commitment'
import sinon from 'sinon'
import { Hash, HoprDB } from '@hoprnet/hopr-utils'

describe('commitment', function () {
  let fakeSet, fakeGet, fakeDB, fakeId: Hash 
  beforeEach(async function () {
    fakeSet = sinon.fake.resolves(undefined)
    fakeGet = sinon.fake.resolves(undefined)
    fakeDB = HoprDB.createMock()
    fakeId = new Hash(new Uint8Array({ length: Hash.SIZE }).fill(1))
  })

  it('should publish a hashed secret', async function () {
    this.timeout(3000)

    initializeCommitment(fakeDB, fakeId, fakeSet, fakeGet)
    let c1 = await findCommitmentPreImage(fakeDB, fakeId)
    assert(c1 != null, 'gives current commitment')
    assert.strictEqual(fakeGet.callCount, 1, 'should look on chain')
    assert(fakeSet.callCount == 1, 'should set a new commitment on chain')

    await bumpCommitment(fakeDB, fakeId)
    let c2 = await findCommitmentPreImage(fakeDB, fakeId) 
    assert(c2, 'gives current commitment')
    assert(c2.hash().eq(c1), 'c2 is commitment of c1')

    fakeGet = () => Promise.resolve(c2)
    initializeCommitment(fakeSet, fakeGet, fakeDB, fakeId)
    let c3 = await findCommitmentPreImage(fakeDB, fakeId)
    assert(c2.eq(c3), 'Repeated initializations should return the same')
  })
})
