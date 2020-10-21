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

import { LibP2P } from '..'
import { CRAWL_TIMEOUT, shouldIncludePeerInCrawlResponse } from './crawler'
import { Crawler as CrawlerInteraction } from '../interactions/network/crawler'
import Multiaddr from 'multiaddr'
import { Network } from './index'
import { Interactions } from '../interactions'
import { BlacklistedEntry } from './network-peers'
import { BLACKLIST_TIMEOUT } from '../constants'
import { durations } from '@hoprnet/hopr-utils'

type Mocks = {
  node: LibP2P
  network: Network
  interactions: Interactions<any>
}

describe('test crawler', function () {
  async function generateMocks(
    options?: { timeoutIntentionally: boolean },
    addr = '/ip4/0.0.0.0/tcp/0'
  ): Promise<Mocks> {
    const node = await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })

    node.peerInfo.multiaddrs.add(Multiaddr(addr))

    await node.start()

    const interactions = {
      network: {
        crawler: new CrawlerInteraction(node, (conn) => {
          return network.crawler.handleCrawlRequest(conn)
        })
      }
    } as Interactions<any>

    const network = new Network(node, interactions, {} as any, { crawl: options })
    node.on('peer:connect', (peerInfo: PeerInfo) => node.peerStore.put(peerInfo))

    return {
      node,
      interactions,
      network
    }
  }

  it('should crawl the network and find some nodes', async function () {
    const [Alice, Bob, Chris, Dave, Eve] = await Promise.all([
      generateMocks(),
      generateMocks(),
      generateMocks(),
      generateMocks(),
      generateMocks()
    ])

    await Alice.network.crawler.crawl()
    Alice.node.emit('peer:connect', Bob.node.peerInfo)
    await Alice.network.crawler.crawl()

    assert(Alice.network.networkPeers.has(Bob.node.peerInfo.id))

    Bob.node.emit('peer:connect', Chris.node.peerInfo)
    assert(Bob.network.networkPeers.has(Chris.node.peerInfo.id))

    await Alice.network.crawler.crawl()
    assert(Alice.network.networkPeers.has(Bob.node.peerInfo.id))
    assert(Alice.network.networkPeers.has(Chris.node.peerInfo.id))

    Chris.node.emit('peer:connect', Dave.node.peerInfo)
    await Alice.network.crawler.crawl()

    assert(Alice.network.networkPeers.has(Bob.node.peerInfo.id))
    assert(Alice.network.networkPeers.has(Chris.node.peerInfo.id))
    assert(Alice.network.networkPeers.has(Dave.node.peerInfo.id))

    Bob.node.emit('peer:connect', Alice.node.peerInfo)
    Dave.node.emit('peer:connect', Eve.node.peerInfo)

    await Bob.network.crawler.crawl()

    // Simulate node failure
    await Bob.node.stop()
    assert(Chris.network.networkPeers.has(Bob.node.peerInfo.id), 'Chris should know about Bob')
    // Simulates a heartbeat run that kicks out Bob
    Alice.network.networkPeers.blacklistPeer(Bob.node.peerInfo.id)
    await Alice.network.crawler.crawl()

    assert(
      !Alice.network.networkPeers.has(Bob.node.peerInfo.id),
      'Alice should not add Bob to her networkPeers after blacklisting him'
    )
    assert(
      Alice.network.networkPeers.deletedPeers.some((entry: BlacklistedEntry) => entry.id.equals(Bob.node.peerInfo.id))
    )

    // Remove Bob from blacklist
    Alice.network.networkPeers.deletedPeers[0].deletedAt -= BLACKLIST_TIMEOUT + 1

    Alice.node.emit('peer:connect', Chris.node.peerInfo)

    await Alice.network.crawler.crawl()

    assert(Alice.network.networkPeers.deletedPeers.length == 0)

    // Alice.network.networkPeers.push({
    //   id: Bob.peerInfo.id.toB58String(),
    //   lastSeen: Date.now()
    // })

    await new Promise((resolve) => setTimeout(resolve, 50))

    assert(Alice.network.networkPeers.has(Bob.node.peerInfo.id))

    await Promise.all([Alice.node.stop(), Bob.node.stop(), Chris.node.stop(), Dave.node.stop(), Eve.node.stop()])
  })
  it(
    'should crawl the network and timeout while crawling',
    async function () {
      let timeoutCorrectly = false
      let before = Date.now()
      const [Alice, Bob, Chris] = await Promise.all([
        generateMocks(),
        generateMocks({
          timeoutIntentionally: true
        }),
        generateMocks({
          timeoutIntentionally: true
        })
      ])

      await Alice.network.crawler.crawl()
      Alice.node.emit('peer:connect', Bob.node.peerInfo)
      await Alice.network.crawler.crawl()
      Bob.node.emit('peer:connect', Chris.node.peerInfo)
      await Alice.network.crawler.crawl()

      await new Promise((resolve) => setTimeout(resolve, 100))
      await Bob.node.stop()
      await Alice.network.crawler.crawl()
      await new Promise((resolve) => setTimeout(resolve, 200))

      timeoutCorrectly = true

      const after = Date.now() - before

      assert(
        timeoutCorrectly && after < 3 * CRAWL_TIMEOUT && after >= 2 * CRAWL_TIMEOUT,
        `Crawling should timeout correctly`
      )

      await Promise.all([Alice.node.stop(), Bob.node.stop(), Chris.node.stop()])
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
