import assert from 'assert'
import {bumpCommitment, ChannelCommitmentInfo, findCommitmentPreImage, initializeCommitment} from './commitment'
import sinon from 'sinon'
import {
  Balance,
  ChannelEntry,
  ChannelStatus,
  Hash,
  HoprDB,
  privKeyToPeerId,
  PublicKey,
  SECRET_LENGTH,
  stringToU8a,
  UINT256
} from '@hoprnet/hopr-utils'
import BN from 'bn.js'

describe('commitment', function () {
  let fakeSet, fakeGet, fakeDB
  let fakeKey: Uint8Array
  let fakeCommInfo: ChannelCommitmentInfo
  beforeEach(async function () {
    fakeSet = sinon.fake.resolves(undefined)
    fakeGet = sinon.fake.resolves(undefined)
    fakeDB = HoprDB.createMock()
    fakeKey = new Uint8Array(SECRET_LENGTH).fill(0)
    fakeCommInfo = new ChannelCommitmentInfo(
      new ChannelEntry(
      PublicKey.fromPeerId(
        privKeyToPeerId(stringToU8a('0x5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca'))
      ),
      PublicKey.fromPeerId(
        privKeyToPeerId(stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa'))
      ),
      new Balance(new BN(123)),
      new Hash(new Uint8Array({ length: Hash.SIZE }).fill(1)),
      UINT256.fromString('1'),
      UINT256.fromString('1'),
      ChannelStatus.Open,
      UINT256.fromString('1'),
      UINT256.fromString('0')
    ),
    1,
    "fakeaddress")
  })

  it('should publish a hashed secret', async function () {
    this.timeout(3000)

    const fakeId = fakeCommInfo.channelEntry.getId()

    await initializeCommitment(fakeDB, fakeKey, fakeCommInfo, fakeGet, fakeSet)
    let c1 = await findCommitmentPreImage(fakeDB, fakeId)
    assert(c1 != null, 'gives current commitment')
    assert.strictEqual(fakeGet.callCount, 1, 'should look on chain')
    assert(fakeSet.callCount == 1, 'should set a new commitment on chain')

    await bumpCommitment(fakeDB, fakeId)
    let c2 = await findCommitmentPreImage(fakeDB, fakeId)
    assert(c2, 'gives current commitment')
    assert(c2.hash().eq(c1), 'c2 is commitment of c1')

    fakeGet = () => Promise.resolve(c2)
    await initializeCommitment(fakeDB, fakeKey, fakeCommInfo, fakeGet, fakeSet)
    let c3 = await findCommitmentPreImage(fakeDB, fakeId)
    assert(c2.eq(c3), 'Repeated initializations should return the same')
  })
})
