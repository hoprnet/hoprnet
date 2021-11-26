import assert from 'assert'
import { bumpCommitment, ChannelCommitmentInfo, findCommitmentPreImage, initializeCommitment } from './commitment'
import sinon from 'sinon'
import {
  Hash,
  HoprDB, privKeyToPeerId,
  stringToU8a,
  UINT256
} from '@hoprnet/hopr-utils'
import PeerId from "peer-id";

describe('commitment', function () {
  let fakeSet, fakeGet, fakeDB
  let fakeKey: PeerId
  let fakeCommInfo: ChannelCommitmentInfo
  beforeEach(async function () {
    fakeSet = sinon.fake.resolves(undefined)
    fakeGet = sinon.fake.resolves(undefined)
    fakeDB = HoprDB.createMock()
    fakeKey = privKeyToPeerId(stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775'))
    fakeCommInfo = new ChannelCommitmentInfo(
        1,
        'fakeaddress',
        new Hash(new Uint8Array({ length: Hash.SIZE }).fill(1)),
        UINT256.fromString('1')
    )
  })

  it('should publish a hashed secret', async function () {
    this.timeout(3000)

    await initializeCommitment(fakeDB, fakeKey, fakeCommInfo, fakeGet, fakeSet)
    let c1 = await findCommitmentPreImage(fakeDB, fakeCommInfo.channelId)
    assert(c1 != null, 'gives current commitment')
    assert.strictEqual(fakeGet.callCount, 1, 'should look on chain')
    assert(fakeSet.callCount == 1, 'should set a new commitment on chain')

    await bumpCommitment(fakeDB, fakeCommInfo.channelId)
    let c2 = await findCommitmentPreImage(fakeDB, fakeCommInfo.channelId)
    assert(c2, 'gives current commitment')
    assert(c2.hash().eq(c1), 'c2 is commitment of c1')

    fakeGet = () => Promise.resolve(c2)
    await initializeCommitment(fakeDB, fakeKey, fakeCommInfo, fakeGet, fakeSet)
    let c3 = await findCommitmentPreImage(fakeDB, fakeCommInfo.channelId)
    assert(c2.eq(c3), 'Repeated initializations should return the same')
  })
})
