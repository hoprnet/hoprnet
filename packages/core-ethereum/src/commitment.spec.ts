import assert from 'assert'
import { bumpCommitment, ChannelCommitmentInfo, findCommitmentPreImage, initializeCommitment } from './commitment.js'
import sinon from 'sinon'
import { Hash, LevelDb, privKeyToPeerId, stringToU8a, U256 } from '@hoprnet/hopr-utils'
import type { PeerId } from '@libp2p/interface-peer-id'
import { Database as Ethereum_Database, PublicKey as Ethereum_PublicKey } from '../lib/core_ethereum_db.js'

describe('commitment', function () {
  let fakeSet: any, fakeGet: any, fakeDB: any
  let fakeKey: PeerId
  let fakeCommInfo: ChannelCommitmentInfo
  beforeEach(async function () {
    const privateKey = stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775')

    fakeSet = sinon.fake.resolves(undefined)
    fakeGet = sinon.fake.resolves(undefined)
    fakeDB = new Ethereum_Database(new LevelDb(), Ethereum_PublicKey.from_privkey(privateKey))
    fakeKey = privKeyToPeerId(privateKey)
    fakeCommInfo = new ChannelCommitmentInfo(
      1,
      'fakeaddress',
      new Hash(new Uint8Array({ length: Hash.size() }).fill(1)),
      U256.one()
    )
  })

  it('should publish a hashed secret', async function () {
    this.timeout(5e3)

    await initializeCommitment(fakeDB, fakeKey, fakeCommInfo, fakeGet, fakeSet)
    let c1 = await findCommitmentPreImage(fakeDB, fakeCommInfo.channelId)
    assert(c1 != null, 'gives current commitment')
    assert.strictEqual(fakeGet.callCount, 1, 'should look on chain')
    assert(fakeSet.callCount == 1, 'should set a new commitment on chain')

    await bumpCommitment(fakeDB, fakeCommInfo.channelId, c1)
    let c2 = await findCommitmentPreImage(fakeDB, fakeCommInfo.channelId)
    assert(c2, 'gives current commitment')
    assert(c2.hash().eq(c1), 'c2 is commitment of c1')

    fakeGet = () => Promise.resolve(c2)
    await initializeCommitment(fakeDB, fakeKey, fakeCommInfo, fakeGet, fakeSet)
    let c3 = await findCommitmentPreImage(fakeDB, fakeCommInfo.channelId)
    assert(c2.eq(c3), 'Repeated initializations should return the same')
  })
})
