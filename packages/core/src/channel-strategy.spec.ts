import assert from 'assert'
import { StrategyFactory } from './channel-strategy.js'
import BN from 'bn.js'
import { ChannelStatus } from '@hoprnet/hopr-utils'

describe('test strategies', async function () {
  it('perform basic promiscuous strategy test', async function () {
    // Perform the same test we perform in the Rust crate to make sure
    // TS wrappers work as intended.

    let strategy = StrategyFactory.getStrategy('promiscuous')
    assert.equal(strategy.name, 'promiscuous')

    const stake = '1000000000000000000'

    let peers = new Map<string, number>()
    peers.set('Alice', 0.1)
    peers.set('Bob', 0.7)
    peers.set('Charlie', 0.9)
    peers.set('Dahlia', 0.1)
    peers.set('Eugene', 0.8)
    peers.set('Felicia', 0.3)
    peers.set('Gustave', 1.0)
    peers.set('Heather', 0.1)
    peers.set('Ian', 0.2)
    peers.set('Joe', 0.3)

    let outgoing_channels = [
      { peer_id: 'Alice', stake_str: stake, status: ChannelStatus.Open },
      { peer_id: 'Charlie', stake_str: stake, status: ChannelStatus.Open },
      { peer_id: 'Gustave', stake_str: '1000000000000000', status: ChannelStatus.Open }
    ]

    // Do some dummy ticks to add some samples
    strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x) as number)
    strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x) as number)

    {
      let res = strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x) as number)

      assert.equal(res.max_auto_channels, 4)
      assert.equal(res.to_close().length, 2)
      assert.equal(res.to_open().length, 3)

      assert(res.to_close().includes('Alice'))
      assert(res.to_close().includes('Gustave'))

      assert.equal(res.to_open()[0].peer_id, 'Gustave')
      assert.equal(res.to_open()[1].peer_id, 'Eugene')
      assert.equal(res.to_open()[2].peer_id, 'Bob')
    }

    // Now reconfigure the strategy and tick again with same inputs
    strategy.configure({
      max_channels: 2,
      auto_redeem_tickets: false
    })

    {
      let res = strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x) as number)

      assert.equal(res.max_auto_channels, 2)
      assert.equal(res.to_close().length, 2)
      assert.equal(res.to_open().length, 1)

      assert(res.to_close().includes('Alice'))
      assert(res.to_close().includes('Gustave'))

      assert.equal(res.to_open()[0].peer_id, 'Gustave')
    }
  })
})
