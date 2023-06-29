import { rm } from 'fs/promises'
import { randomBytes } from 'crypto'
import assert from 'assert'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { debug, LevelDb, privKeyToPeerId, u8aToHex } from '@hoprnet/hopr-utils'
import Hopr, { type HoprOptions } from './index.js'
import { sampleOptions } from './index.mock.js'
import { setTimeout } from 'timers/promises'
import { Multiaddr } from '@multiformats/multiaddr'
import { startStunServer } from '@hoprnet/hopr-connect'
import type { PeerId } from '@libp2p/interface-peer-id'
import { Database, PublicKey } from '../lib/core_hopr.js'

/**
 * Synchronous function to sample PeerIds
 * @returns a PeerId
 */
export function createPeerId(): PeerId {
  return privKeyToPeerId(u8aToHex(randomBytes(32)))
}

const log = debug('hopr-core:test:index')

const peerId = privKeyToPeerId('0x1c28c7f301658b4807a136e9fcf5798bc37e24b70f257fd3e6ee5adcf83a8c1f')

interface MinimalStunServer {
  tcpPort: number
  close: () => Promise<void>
}

function stunServerToAddress(stunServers: MinimalStunServer[]) {
  return stunServers.map((serverPort: MinimalStunServer) => {
    const peerId = createPeerId()
    return {
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${serverPort.tcpPort}/p2p/${peerId.toString()}`)],
      id: peerId
    }
  })
}

describe('hopr core (instance)', function () {
  it('start and stop Hopr node', async function () {
    const stunServers = await Promise.all(Array.from({ length: 2 }, (_) => startStunServer()))

    this.timeout(15000)
    log('Clean up data folder from previous attempts')
    await rm(sampleOptions.dataPath, { recursive: true, force: true })

    const opts: HoprOptions = {
      ...sampleOptions,

      testing: {
        preferLocalAddresses: true,
        localModeStun: true
      }
    } as HoprOptions

    log('Creating hopr node...')
    HoprCoreEthereum.createMockInstance(peerId)
    const db = new LevelDb()
    await db.backend.open()

    const node = new Hopr(peerId, new Database(db, PublicKey.from_peerid_str(peerId.toString())), opts as HoprOptions)

    log('Node created with Id', node.getId().toString())
    assert(node instanceof Hopr)

    log('Starting node')
    await node.start(stunServerToAddress(stunServers))

    // Give libp2p some time to initialize
    await setTimeout(8000)

    await node.stop()
    await HoprCoreEthereum.getInstance().stop()

    await setTimeout(100)

    log('Clean up data folder')
    await rm(sampleOptions.dataPath, { recursive: true, force: true })

    await Promise.all(stunServers.map((s) => s.close()))
  })
})
