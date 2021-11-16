import LibP2P from 'libp2p'
import PeerId from 'peer-id'
import Hopr from '@hoprnet/hopr-core'
import { HoprOptions } from '@hoprnet/hopr-core'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { debug, privKeyToPeerId, HoprDB } from '@hoprnet/hopr-utils'
import sinon from 'sinon'

const log = debug('hopr:test:cover-traffic')

describe('cover-traffic daemon', async function () {
  const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'  

  let node: Hopr, libp2p: LibP2P
  let peerId: PeerId, db: HoprDB, chain: HoprCoreEthereum, options: HoprOptions
  beforeEach(function () {
    peerId = privKeyToPeerId(privateKey)
    options = { environment: { id: '1' } } as unknown as HoprOptions
    db = sinon.createStubInstance(HoprDB)
    chain = sinon.createStubInstance(HoprCoreEthereum)
    node = new Hopr(peerId, db, chain, options)
    libp2p = sinon.createStubInstance(LibP2P)
    sinon.stub(node.indexer, 'start').callsFake(() => Promise.resolve(log('indexer called for cover-traffic daemon')))
    sinon.stub(node.indexer, 'getPublicNodes').callsFake(() => {
      log('indexer called for public nodes')
      return Promise.resolve([])
    })
    sinon.stub(LibP2P, 'create').callsFake(() => {
      log('libp2p stub started')
      return Promise.resolve(libp2p)
    })
  })
  afterEach(function () {
    sinon.restore()
  })
  it('should run properly', async function () {
    log('starting stubbed cover-traffic daemon')
    await node.start()
  })
})
