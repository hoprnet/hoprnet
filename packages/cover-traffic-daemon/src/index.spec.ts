import LibP2P from 'libp2p'
import PeerId from 'peer-id'
import Hopr from '@hoprnet/hopr-core'
import { HoprOptions } from '@hoprnet/hopr-core'
import HoprCoreEthereum, { Indexer } from '@hoprnet/hopr-core-ethereum'
import { debug, privKeyToPeerId, HoprDB, NativeBalance } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import BN from 'bn.js'


const log = debug('hopr:test:cover-traffic')

describe('cover-traffic daemon', async function () {
  const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'

  let node: Hopr, libp2p: LibP2P, indexer: Indexer
  let peerId: PeerId, db: HoprDB, chain: HoprCoreEthereum, options: HoprOptions
  beforeEach(function () {
    peerId = privKeyToPeerId(privateKey)
    options = { environment: { id: '1' } } as unknown as HoprOptions
    db = sinon.createStubInstance(HoprDB)

    indexer = {
      getPublicNodes: () => {
        log('getPublicNodes method was called')
      },
    } as unknown as Indexer

    chain = sinon.createStubInstance(HoprCoreEthereum)
    chain.indexer = indexer;
    chain.start = sinon.fake(() => {
      log('On-chain instance start method was called.')
      return {
        getNativeBalance: () => {
          log('getNativeBalance method was called')
          // Adding a value of >0 to register node has been funded.
          return Promise.resolve(new NativeBalance(new BN('10000000000000000000')))
        }
      };
    });

    node = new Hopr(peerId, db, chain, options)
    libp2p = sinon.createStubInstance(LibP2P)
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
    log('starting stubbed cover-traffic daemonasdad')

  })
})
