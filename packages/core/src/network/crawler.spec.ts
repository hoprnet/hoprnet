import assert from 'assert'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import libp2p from 'libp2p'
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

import Hopr from '..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Interactions } from '../interactions'
import { Crawler, CRAWL_TIMEOUT, shouldIncludePeerInCrawlResponse } from './crawler'
import { Crawler as CrawlerInteraction } from '../interactions/network/crawler'
import Multiaddr from 'multiaddr'
import PeerStore, { BLACKLIST_TIMEOUT, BlacklistedEntry } from './peerStore'
import { durations } from '@hoprnet/hopr-utils'

describe('test crawler', function () {
  async function generateNode(
    options?: { timeoutIntentionally: boolean },
    addr = '/ip4/0.0.0.0/tcp/0'
  ): Promise<Hopr<HoprCoreConnector>> {
    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
      },
    })) as Hopr<HoprCoreConnector>

    node.peerInfo.multiaddrs.add(Multiaddr(addr))

    await node.start()

    node.peerRouting.findPeer = (_: PeerId) => Promise.reject('not implemented')

    node.interactions = {
      network: {
        crawler: new CrawlerInteraction(node),
      },
    } as Hopr<HoprCoreConnector>['interactions']

    new Interactions(node)
    node.network = {
      crawler: new Crawler(node, options),
      peerStore: new PeerStore(node),
    } as Hopr<HoprCoreConnector>['network']

    node.on('peer:connect', (peerInfo: PeerInfo) => node.peerStore.put(peerInfo))

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

  it(
    'should crawl the network and timeout while crawling',
    async function () {
      let timeoutCorrectly = false
      let before = Date.now()
      const [Alice, Bob, Chris] = await Promise.all([
        generateNode(),
        generateNode({
          timeoutIntentionally: true,
        }),
        generateNode({
          timeoutIntentionally: true,
        }),
      ])

      await Alice.network.crawler.crawl()

      // await assert.rejects(
      //   () => Alice.network.crawler.crawl(),
      //   Error(`Unable to find enough other nodes in the network.`)
      // )

      Alice.emit('peer:connect', Bob.peerInfo)
      await Alice.network.crawler.crawl()
      Bob.emit('peer:connect', Chris.peerInfo)
      await Alice.network.crawler.crawl()

      await new Promise((resolve) => setTimeout(resolve, 100))
      await Bob.stop()
      await Alice.network.crawler.crawl()
      await new Promise((resolve) => setTimeout(resolve, 200))

      timeoutCorrectly = true

      const after = Date.now() - before
      assert(
        timeoutCorrectly && after < 3 * CRAWL_TIMEOUT && after >= 2 * CRAWL_TIMEOUT,
        `Crawling should timeout correctly`
      )

      await Promise.all([
        /* prettier-ignore */
        Alice.stop(),
        Bob.stop(),
        Chris.stop(),
      ])
    },
    durations.seconds(8)
  )
  it('shouldIncludePeerInCrawlResponse', async () => {
    assert(
      shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/123.4.5.6/tcp/5000'), Multiaddr('/ip4/12.34.56.7/tcp/5000'))
    )
    assert(shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/127.0.0.1/tcp/1000'), Multiaddr('/ip4/127.0.0.1/tcp/5000')))
    assert(
      !shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/127.0.0.1/tcp/5000'), Multiaddr('/ip4/12.34.56.7/tcp/5000'))
    )
  })
})
