import { Ganache, getNewPort } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import { durations } from '@hoprnet/hopr-utils'
import HoprCore from '.'
import assert from 'assert'

import { privKeyToPeerId } from './utils'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import PeerInfo from 'peer-info'

describe('test hopr-core', function () {
  const ganache = new Ganache()

  beforeAll(async function () {
    await ganache.start()
    await migrate()
  }, durations.seconds(30))

  it(
    'should start a node',
    async function () {
      const node = await HoprCore.create({
        debug: true,
        bootstrapNode: true,
        dbPath: process.cwd() + '/testdb',
        network: 'ethereum',
        provider: 'ws://127.0.0.1:9545',
        hosts: {
          ip4: {
            ip: '0.0.0.0',
            port: getNewPort(),
          },
        },
      })

      assert(node != null, `Node creation must not lead to 'undefined'`)

      await node.stop()
    },
    durations.seconds(3)
  )

  it(
    `should not call ourself`,
    async function () {
      const peerId = await privKeyToPeerId(NODE_SEEDS[0])

      const node = await HoprCore.create({
        debug: true,
        peerId,
        bootstrapNode: true,
        network: 'ethereum',
        provider: 'ws://127.0.0.1:9545',
        hosts: {
          ip4: {
            ip: '0.0.0.0',
            port: getNewPort(),
          },
        },
        bootstrapServers: [new PeerInfo(peerId)],
      })

      await node.stop()
    },
    durations.seconds(3)
  )

  afterAll(async function () {
    await ganache.stop()
  })
})
