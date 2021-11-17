import LibP2P from 'libp2p'
import PeerId from 'peer-id'
import Hopr from '@hoprnet/hopr-core'
import { HoprOptions } from '@hoprnet/hopr-core'
import HoprCoreEthereum, { Indexer } from '@hoprnet/hopr-core-ethereum'
import { debug, privKeyToPeerId, HoprDB, NativeBalance } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import BN from 'bn.js'

const namespace = 'hopr:test:cover-traffic'
const log = debug(namespace)

describe('cover-traffic daemon', async function () {
  const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'

  let node: Hopr, libp2p: LibP2P, indexer: Indexer
  let peerId: PeerId, db: HoprDB, chain: HoprCoreEthereum, options: HoprOptions
  beforeEach(function () {
    peerId = privKeyToPeerId(privateKey)
    options = { environment: { id: '1' } } as unknown as HoprOptions
    db = sinon.createStubInstance(HoprDB)

    const indexerLogger = debug(namespace + ':indexer')
    indexer = {
      getPublicNodes: () => {
        indexerLogger('getPublicNodes method was called')
      },
    } as unknown as Indexer

    const chainLogger = debug(namespace + ':chain')
    chain = sinon.createStubInstance(HoprCoreEthereum)
    chain.indexer = indexer;
    // @TODO: Use better (ie typed) way to overload stub
    chain.start = sinon.fake(() => {
      chainLogger('On-chain instance start method was called.')
      return {
        getNativeBalance: () => {
          chainLogger('getNativeBalance method was called')
          // Adding a value of >0 to register node has been funded.
          // @TODO: Pick a more relevant value.
          return Promise.resolve(new NativeBalance(new BN('10000000000000000000')))
        },
        waitForPublicNodes: () => {
          chainLogger('On-chain request for existing public nodes.')
          return Promise.resolve([])
        },
        on: () => {
          chainLogger('On-chain signal for event.')
        },
        indexer: {
          on: () => indexerLogger('Indexer on handler top of chain called'),
          off: () => indexerLogger('Indexer off handler top of chain called')
        }
      };
    });

    const libp2pLogger = debug(namespace + ':libp2p')
    libp2p = sinon.createStubInstance(LibP2P)
    sinon.stub(LibP2P, 'create').callsFake(() => {
      libp2pLogger('libp2p stub started')
      return Promise.resolve(libp2p)
    })

    node = new Hopr(peerId, db, chain, options)
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
