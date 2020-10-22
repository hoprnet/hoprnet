import { Ganache, getNewPort } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import { durations } from '@hoprnet/hopr-utils'
import Hopr from '.'
import assert from 'assert'

import { privKeyToPeerId } from './utils'
import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import Multiaddr from 'multiaddr'

describe('test hopr-core', function () {
  let ganache
  let node

  beforeAll(async function () {
    ganache = new Ganache()
    await ganache.start()
    await migrate()
  }, durations.seconds(30))

  afterAll(async function () {
    await ganache.stop()
  })

  afterEach(async function () {
    await node.stop()
  })

  it(
    'should start a node',
    async function () {
      node = await Hopr.create({
        debug: true,
        bootstrapNode: true,
        password: '',
        dbPath: process.cwd() + '/testdb',
        network: 'ethereum',
        provider: 'ws://127.0.0.1:9545',
        hosts: {
          ip4: {
            ip: '0.0.0.0',
            port: getNewPort()
          }
        }
      })

      assert(node != null, `Node creation must not lead to 'undefined'`)
    },
    durations.seconds(100)
  )

  it(
    `should not call ourself`,
    async function () {
      const peerId = await privKeyToPeerId(NODE_SEEDS[0])

      node = await Hopr.create({
        debug: true,
        peerId,
        bootstrapNode: true,
        network: 'ethereum',
        provider: 'ws://127.0.0.1:9545',
        hosts: {
          ip4: {
            ip: '0.0.0.0',
            port: getNewPort()
          }
        },
        bootstrapServers: [new Multiaddr('/p2p/' + peerId.toB58String())]
      })
    },
    durations.seconds(5)
  )
})
