import { rm } from 'fs/promises'
import assert from 'assert'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { dbMock, debug, privKeyToPeerId } from '@hoprnet/hopr-utils'
import Hopr, { type HoprOptions } from './index.js'
import { sampleOptions } from './index.mock.js'
import { setTimeout } from 'timers/promises'

const log = debug('hopr-core:test:index')

const peerId = privKeyToPeerId('0x1c28c7f301658b4807a136e9fcf5798bc37e24b70f257fd3e6ee5adcf83a8c1f')

describe('hopr core (instance)', async function () {
  it('start and stop Hopr node', async function () {
    this.timeout(15000)
    log('Clean up data folder from previous attempts')
    await rm(sampleOptions.dataPath, { recursive: true, force: true })

    log('Creating hopr node...')
    HoprCoreEthereum.createMockInstance(peerId)
    const node = new Hopr(peerId, dbMock, sampleOptions as HoprOptions)

    log('Node created with Id', node.getId().toString())
    assert(node instanceof Hopr)

    log('Starting node')
    await node.start()

    // Give libp2p some time to initialize
    await setTimeout(8000)

    await node.stop()
    await HoprCoreEthereum.instance.stop()

    await setTimeout(100)

    log('Clean up data folder')
    await rm(sampleOptions.dataPath, { recursive: true, force: true })
  })
})
