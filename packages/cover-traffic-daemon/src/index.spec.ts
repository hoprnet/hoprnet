import LibP2P from 'libp2p'
import PeerId from 'peer-id'
import Hopr from '@hoprnet/hopr-core'
import { HoprOptions } from '@hoprnet/hopr-core'
import HoprCoreEthereum, { Indexer } from '@hoprnet/hopr-core-ethereum'
import { debug, privKeyToPeerId, HoprDB, NativeBalance, AccountEntry, Address } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import BN from 'bn.js'
import ConnectionManager from 'libp2p/src/connection-manager'
import PeerStore from 'libp2p/src/peer-store'
import { Multiaddr } from 'multiaddr'
import AddressManager from 'libp2p/src/address-manager'

const namespace = 'hopr:test:cover-traffic'
const log = debug(namespace)

describe('cover-traffic daemon', async function () {
  const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
  const sampleAddress = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
  const samplePeerId = PeerId.createFromB58String('16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7')
  const sampleMultiaddrs = new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${samplePeerId.toB58String()}`)

  let node: Hopr, libp2p: LibP2P, indexer: Indexer
  let peerId: PeerId, db: HoprDB, chain: HoprCoreEthereum, options: HoprOptions
  beforeEach(function () {
    peerId = privKeyToPeerId(privateKey)
    options = { environment: { id: '1' } } as unknown as HoprOptions
    db = sinon.createStubInstance(HoprDB)

    const dbLogger = debug(namespace + ':db')
    db.close = () => {
      dbLogger('Closing database')
      return Promise.resolve()
    }

    const chainLogger = debug(namespace + ':chain')
    chain = sinon.createStubInstance(HoprCoreEthereum)
    chain.indexer = indexer
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
        getPublicKey: () => {
          chainLogger('getPublicKey method was called')
          return {
            toAddress: () => Promise.resolve(sampleAddress)
          }
        },
        getAccount: () => {
          chainLogger('getAccount method was called')
          return Promise.resolve(new AccountEntry(sampleAddress, sampleMultiaddrs, new BN('1')))
        },
        waitForPublicNodes: () => {
          chainLogger('On-chain request for existing public nodes.')
          return Promise.resolve([])
        },
        announce: () => {
          chainLogger('On-chain announce request sent')
        },
        on: (event: string) => {
          chainLogger(`On-chain signal for event "${event}"`)
        },
        indexer: {
          on: (event: string) => chainLogger(`Indexer on handler top of chain called with event "${event}"`),
          off: (event: string) => chainLogger(`Indexer off handler top of chain called with event "${event}`)
        }
      }
    })

    const libp2pLogger = debug(namespace + ':libp2p')
    libp2p = sinon.createStubInstance(LibP2P)
    libp2p._options = Object.assign({}, libp2p._options, {
      addresses: {
        announceFilter: () => [sampleMultiaddrs]
      }
    })
    libp2p.connectionManager = sinon.createStubInstance(ConnectionManager)
    libp2p.connectionManager.on = sinon.fake((event: string) => {
      libp2pLogger(`Connection manager event handler called with event "${event}"`)
    })
    libp2p.peerStore = new PeerStore({ peerId: samplePeerId })
    libp2p.addressManager = new AddressManager(peerId, { announce: [sampleMultiaddrs.toString()] })
    sinon.stub(LibP2P, 'create').callsFake(() => {
      libp2pLogger('libp2p stub started')
      return Promise.resolve(libp2p)
    })

    node = new Hopr(peerId, db, chain, options)
  })
  afterEach(function () {
    sinon.restore()
  })
  it('should run and stop properly', async function () {
    log('starting stubbed cover-traffic daemon')
    await node.start()
    log('completed stubbed cover-traffic')
    log('Starting node stop process')
    await node.stop()
    log('Stopped node succesfully')
    log('Triggering period check to exit cover-traffic strategy now that node is stopped')
    await node.periodicCheck()
    log('Everything is now stopped')
  })
})
