import assert from 'assert'
import { PromiscuousStrategy, OutgoingChannelStatus } from './channel-strategy.js'
import BN from 'bn.js'

describe('test strategies', async function () {
  it('perform basic promiscuous strategy test', async function () {
    // Perform the same test we perform in the Rust crate to make sure
    // TS wrappers work as intended.

    let strategy = new PromiscuousStrategy()
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
      new OutgoingChannelStatus('Alice', stake),
      new OutgoingChannelStatus('Charlie', stake),
      new OutgoingChannelStatus('Gustave', '1000000000000000')
    ]

    let res = strategy.tick(new BN(stake), peers.keys(), outgoing_channels, (x: string) => peers.get(x))

    assert.equal(res.max_auto_channels, 4)
    assert.equal(res.to_close().length, 2)
    assert.equal(res.to_open().length, 3)

    assert(res.to_close().includes('Alice'))
    assert(res.to_close().includes('Gustave'))

    assert.equal(res.to_open()[0].peer_id, 'Gustave')
    assert.equal(res.to_open()[1].peer_id, 'Eugene')
    assert.equal(res.to_open()[2].peer_id, 'Bob')
  })
})
