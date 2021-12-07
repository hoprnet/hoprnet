import { debug, HoprDB } from '@hoprnet/hopr-utils'
import { sampleChainOptions, useFixtures } from '@hoprnet/hopr-core-ethereum'
import { expect } from 'chai'
import PeerId from 'peer-id'
import sinon from 'sinon'
import Hopr, { HoprOptions } from '.'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { ChainWrapperSingleton } from '@hoprnet/hopr-core-ethereum/src/ethereum'
import { ACCOUNT_A, PARTY_A } from '@hoprnet/hopr-core-ethereum/src/fixtures'

const log = debug('hopr-core:test:index')

describe('hopr core (instance)', async function () {
  let peerId: PeerId, db: HoprDB, chain: HoprCoreEthereum, options: HoprOptions
  beforeEach(async function () {
    sinon.stub(ChainWrapperSingleton, 'create').callsFake(async () => {
      log('chainwrapper singleton stub started')
      const { chain } = (await useFixtures())
      return Promise.resolve(chain)
    })
    peerId = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    options = { environment: { id: '1' } } as unknown as HoprOptions
    db = sinon.createStubInstance(HoprDB)
    chain = new HoprCoreEthereum(db, PARTY_A, Buffer.from(ACCOUNT_A.privateKey, 'hex'), sampleChainOptions)
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should be able to start a hopr node instance without crashing', async function () {
    expect(async () => {
      log('Creating hopr node...')
      const node = new Hopr(peerId, db, chain, options)
      log('Node created with Id', node.getId().toB58String())
      expect(node instanceof Hopr, 'and have the right instance')
      log('Starting node')
      await node.start()
    }).to.not.throw()
  })
})
