import { getNewPort } from '@hoprnet/hopr-testing'
import Hopr from '.'
import assert from 'assert'

import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import Multiaddr from 'multiaddr'
import { ACCOUNT_1 } from './fixtures'

describe('test hopr-core', function () {
  let node: Hopr

  afterEach(async function () {
    await node.stop()
  })

  it('should start a node', async function () {
    this.timeout(5000)

    node = await Hopr.create({
      bootstrapNode: true,
      password: '',
      createDbIfNotExist: true,
      dbPath: process.cwd() + '/testdb',
      network: 'ethereum',
      provider: 'ws://127.0.0.1:8545',
      hosts: {
        ip4: {
          ip: '0.0.0.0',
          port: getNewPort()
        }
      }
    })

    assert(node != null, `Node creation must not lead to 'undefined'`)
  })

  it(`should not call ourself`, async function () {
    this.timeout(5000)

    const peerId = await privKeyToPeerId(ACCOUNT_1)
    node = await Hopr.create({
      bootstrapNode: true,
      peerId,
      createDbIfNotExist: true,
      network: 'ethereum',
      provider: 'ws://127.0.0.1:8545',
      hosts: {
        ip4: {
          ip: '0.0.0.0',
          port: getNewPort()
        }
      },
      bootstrapServers: [new Multiaddr('/p2p/' + peerId.toB58String())]
    })
  })
})
