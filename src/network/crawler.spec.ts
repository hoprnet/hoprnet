import assert from 'assert'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

// @ts-ignore
import libp2p = require('libp2p')
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

import Debug from 'debug'
import chalk from 'chalk'

import Hopr from '..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Interactions } from '../interactions'
import { Crawler } from './crawler'
import { Crawler as CrawlerInteraction } from '../interactions/network/crawler'
import Multiaddr from 'multiaddr'
import PeerStore, { BLACKLIST_TIMEOUT, BlacklistedEntry } from './peerStore'

describe('test crawler', function () {
  async function generateNode(): Promise<Hopr<HoprCoreConnector>> {
    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
      },
    })) as Hopr<HoprCoreConnector>

    node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))

    await node.start()

    node.peerRouting.findPeer = (_: PeerId) => Promise.reject('not implemented')

    node.interactions = {
      network: {
        crawler: new CrawlerInteraction(node),
      },
    } as Hopr<HoprCoreConnector>['interactions']

    new Interactions(node)
    node.network = {
      crawler: new Crawler(node),
      peerStore: new PeerStore(node),
    } as Hopr<HoprCoreConnector>['network']

    node.on('peer:connect', (peerInfo: PeerInfo) => node.peerStore.put(peerInfo))

    node.log = Debug(`${chalk.blue(node.peerInfo.id.toB58String())}: `)

    return (node as unknown) as Hopr<HoprCoreConnector>
  }

  it('should crawl the network and find some nodes', async function () {
    const [Alice, Bob, Chris, Dave, Eve] = await Promise.all([
      generateNode(),
      generateNode(),
      generateNode(),
      generateNode(),
      generateNode(),
    ])

    await Alice.network.crawler.crawl()

    // await assert.rejects(
    //   () => Alice.network.crawler.crawl(),
    //   Error(`Unable to find enough other nodes in the network.`)
    // )

    Alice.emit('peer:connect', Bob.peerInfo)

    await Alice.network.crawler.crawl()

    assert(Alice.network.peerStore.has(Bob.peerInfo.id.toB58String()))
    // await assert.rejects(
    //   () => Alice.network.crawler.crawl(),
    //   Error(`Unable to find enough other nodes in the network.`)
    // )

    Bob.emit('peer:connect', Chris.peerInfo)

    await Alice.network.crawler.crawl()

    assert(Alice.network.peerStore.has(Bob.peerInfo.id.toB58String()))
    assert(Alice.network.peerStore.has(Chris.peerInfo.id.toB58String()))

    // await assert.rejects(
    //   () => Alice.network.crawler.crawl(),
    //   Error(`Unable to find enough other nodes in the network.`)
    // )

    Chris.emit('peer:connect', Dave.peerInfo)

    await Alice.network.crawler.crawl()

    assert(Alice.network.peerStore.has(Bob.peerInfo.id.toB58String()))
    assert(Alice.network.peerStore.has(Chris.peerInfo.id.toB58String()))
    assert(Alice.network.peerStore.has(Dave.peerInfo.id.toB58String()))

    Bob.emit('peer:connect', Alice.peerInfo)
    Dave.emit('peer:connect', Eve.peerInfo)

    await Bob.network.crawler.crawl()

    // Simulate node failure
    await Bob.stop()

    assert(Chris.network.peerStore.has(Bob.peerInfo.id.toB58String()), 'Chris should know about Bob')

    // Simulates a heartbeat run that kicks out Bob
    Alice.network.peerStore.blacklistPeer(Bob.peerInfo.id.toB58String())

    await Alice.network.crawler.crawl()

    assert(
      !Alice.network.peerStore.has(Bob.peerInfo.id.toB58String()),
      'Alice should not add Bob to her peerStore after blacklisting him'
    )

    assert(
      Alice.network.peerStore.deletedPeers.some((entry: BlacklistedEntry) => entry.id === Bob.peerInfo.id.toB58String())
    )

    // Remove Bob from blacklist
    Alice.network.peerStore.deletedPeers[0].deletedAt -= BLACKLIST_TIMEOUT + 1

    Alice.emit('peer:connect', Chris.peerInfo)

    await Alice.network.crawler.crawl()

    assert(Alice.network.peerStore.deletedPeers.length == 0)

    // Alice.network.peerStore.push({
    //   id: Bob.peerInfo.id.toB58String(),
    //   lastSeen: Date.now()
    // })

    await new Promise((resolve) => setTimeout(resolve, 50))

    assert(Alice.network.peerStore.has(Bob.peerInfo.id.toB58String()))

    await Promise.all([
      /* prettier-ignore */
      Alice.stop(),
      Bob.stop(),
      Chris.stop(),
      Dave.stop(),
      Eve.stop(),
    ])
  })
})
