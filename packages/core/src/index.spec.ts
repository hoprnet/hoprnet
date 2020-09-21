import { Ganache } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import { durations } from '@hoprnet/hopr-utils'
import HoprCore from '.'
import assert from 'assert'

describe('test hopr-core', function () {
  const ganache = new Ganache()

  beforeAll(async function () {
    await ganache.start()
    await migrate()
  }, durations.seconds(30))

  it('should start a node', async function () {
    jest.setTimeout(durations.seconds(3))

    const node = await HoprCore.create({
      debug: true,
      bootstrapNode: true,
      dbPath: process.cwd() + '/testdb',
      network: 'ethereum',
      provider: 'ws://127.0.0.1:9545',
      hosts: {
        ip4: {
          ip: '0.0.0.0',
          port: 9091,
        },
      },
    })

    assert(node != null)

    await node.stop()
  })

  afterAll(async function () {
    await ganache.stop()
  })
})
