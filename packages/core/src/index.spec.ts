import { debug, HoprDB, stringToU8a } from '@hoprnet/hopr-utils'
import { sampleChainOptions, useFixtures } from '@hoprnet/hopr-core-ethereum'
import { expect } from 'chai'
import PeerId from 'peer-id'
import sinon from 'sinon'
import Hopr, { HoprOptions } from '.'
import HoprEthereum from '@hoprnet/hopr-core-ethereum'
import { ACCOUNT_A, PARTY_A } from '@hoprnet/hopr-core-ethereum/src/fixtures'
import { ChainWrapperSingleton } from '@hoprnet/hopr-core-ethereum'

const log = debug('hopr-core:test:index')

describe('hopr core (instance)', async function () {
  let peerId: PeerId, db: HoprDB, connector: HoprEthereum, options: HoprOptions
  beforeEach(async function () {
    log('Before each hook starting by setting up chain fixture')
    const { chain } = await useFixtures()
    log('ChainWrapper obtained from fixtures')
    sinon.stub(ChainWrapperSingleton, 'create').callsFake(() => {
      log('chainwrapper singleton stub started')
      return Promise.resolve(chain)
    })
    log('ChainWrapperSingleton create stubbed', ChainWrapperSingleton.create);
    peerId = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    options = { environment: { id: '1' } } as unknown as HoprOptions
    db = sinon.createStubInstance(HoprDB)
    connector = new HoprEthereum(db, PARTY_A, stringToU8a(ACCOUNT_A.privateKey), sampleChainOptions)
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should be able to start a hopr node instance without crashing', async function () {
    expect(async () => {
      log('Creating hopr node...')
      const node = new Hopr(peerId, db, connector, options)
      log('Node created with Id', node.getId().toB58String())
      expect(node instanceof Hopr, 'and have the right instance')
      log('Starting node')
      await node.start()
    }).to.not.throw()
  })
})
