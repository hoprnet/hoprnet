import assert from 'assert'
import { StrategyFactory } from './channel-strategy.js'
import BN from 'bn.js'
import { ChannelStatus } from '@hoprnet/hopr-utils'
import { privKeyToPeerId, u8aToHex } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'

function createPeerId(): string {
  return privKeyToPeerId(u8aToHex(randomBytes(32))).toString()
}

describe('test strategies', async function () {
  it('perform basic promiscuous strategy test', async function () {
    // Perform the same test we perform in the Rust crate to make sure
    // TS wrappers work as intended.

    let strategy = StrategyFactory.getStrategy('promiscuous')
    assert.equal(strategy.name, 'promiscuous')

    const stake = '1000000000000000000'

    let alice = createPeerId()
    let bob = createPeerId()
    let charlie = createPeerId()
    let gustave = createPeerId()
    let eugene = createPeerId()

    let peers = new Map<string, number>()
    peers.set(alice, 0.1)
    peers.set(bob, 0.7)
    peers.set(charlie, 0.9)
    peers.set(createPeerId(), 0.1)
    peers.set(eugene, 0.8)
    peers.set(createPeerId(), 0.3)
    peers.set(gustave, 1.0)
    peers.set(createPeerId(), 0.1)
    peers.set(createPeerId(), 0.2)
    peers.set(createPeerId(), 0.3)

    let outgoing_channels = [
      { peer_id: alice, stake_str: stake, status: ChannelStatus.Open },
      { peer_id: charlie, stake_str: stake, status: ChannelStatus.Open },
      { peer_id: gustave, stake_str: '1000000000000000', status: ChannelStatus.Open }
    ]

    // Do some dummy ticks to add some samples
    strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x) as number)
    strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x) as number)

    {
      let res = strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x) as number)

      assert.equal(res.max_auto_channels, 4)
      assert.equal(res.to_close().length, 2)
      assert.equal(res.to_open().length, 3)

      assert(res.to_close().includes(alice))
      assert(res.to_close().includes(gustave))

      assert.equal(res.to_open()[0].peer_id, gustave)
      assert.equal(res.to_open()[1].peer_id, eugene)
      assert.equal(res.to_open()[2].peer_id, bob)
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

      assert(res.to_close().includes(alice))
      assert(res.to_close().includes(gustave))

      assert.equal(res.to_open()[0].peer_id, gustave)
    }
  })
})
