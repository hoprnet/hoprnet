import { Ganache } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import { durations } from '@hoprnet/hopr-utils'
import HoprCore from '.'

describe('test hopr-core', function () {
  const ganache = new Ganache()

  beforeAll(async function () {
    jest.setTimeout(durations.seconds(30))

    await ganache.start()
    await migrate()
  })

  it('should start a node', function (done) {
    expect(
      HoprCore.create({
        debug: true,
        bootstrapNode: true,
        network: 'ethereum',
        provider: 'ws://127.0.0.1:9545',
        hosts: {
          ip4: {
            ip: '0.0.0.0',
            port: 9091,
          },
        },
      })
    )
      .resolves.not.toBeUndefined()
      .then(done)
  })
})
