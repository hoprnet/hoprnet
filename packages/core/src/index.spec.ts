import { debug, HoprDB } from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { expect } from 'chai'
import PeerId from 'peer-id'
import sinon from 'sinon'
import Hopr, { HoprOptions } from '.'

const log = debug('hopr-core:test:index')

describe('hopr core (instance)', async function () {
  let peerId: PeerId, db: HoprDB, chain: HoprCoreEthereum, options: HoprOptions
  beforeEach(async function () {
    peerId = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    options = { environment: { id: '1' } } as unknown as HoprOptions
    db = sinon.createStubInstance(HoprDB)
    chain = sinon.createStubInstance(HoprCoreEthereum)
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should be able to create a hopr node instance without crashing', async function () {
    expect(() => {
      log('Creating hopr node...')
      const node = new Hopr(peerId, db, chain, options)
      log('Node created with Id', node.getId().toB58String())
      expect(node instanceof Hopr)
    }).to.not.throw()
  })
})
