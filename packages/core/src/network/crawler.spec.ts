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
import { CRAWL_TIMEOUT, shouldIncludePeerInCrawlResponse } from './crawler'
import { Crawler as CrawlerInteraction } from '../interactions/network/crawler'
import Multiaddr from 'multiaddr'
import { Network } from './index'
import { BlacklistedEntry } from './network-peers'
import { BLACKLIST_TIMEOUT } from '../constants'
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
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprCoreConnector>

    node.peerInfo.multiaddrs.add(Multiaddr(addr))

    await node.start()

    node._interactions = ({
      network: {
        crawler: new CrawlerInteraction(node)
        /*
          interact: async (peer) => {
            return Promise.resolve(
              node.network.networkPeers.peers.map(
                x => new PeerInfo(PeerId.createFromB58String((x.id)))
              )
            )
          }
        }
          */
      }
    } as any) as Hopr<HoprCoreConnector>['_interactions']


    node._network = new Network(node, node._interactions, {} as any, { crawl: options })
    node.getConnectedPeers = () => node._network.networkPeers.peers.map(x => x.id)
    node.on('peer:connect', (peerInfo: PeerInfo) => node.peerStore.put(peerInfo))
    return (node as unknown) as Hopr<HoprCoreConnector>
  }

  it('should crawl the network and find some nodes', async function () {
    const [Alice, Bob, Chris, Dave, Eve] = await Promise.all([
      generateNode(),
      generateNode(),
      generateNode(),
      generateNode(),
      generateNode()
    ])

    await Alice.crawl()
    Alice.emit('peer:connect', Bob.peerInfo)
    await Alice.crawl()

    assert(Alice.getConnectedPeers().includes(Bob.peerInfo.id))

    Bob.emit('peer:connect', Chris.peerInfo)
    await Alice.crawl()

    assert(Alice.getConnectedPeers().includes(Bob.peerInfo.id))
    assert(Alice.getConnectedPeers().includes(Chris.peerInfo.id))

    Chris.emit('peer:connect', Dave.peerInfo)
    await Alice.crawl()

    assert(Alice.getConnectedPeers().includes(Bob.peerInfo.id))
    assert(Alice.getConnectedPeers().includes(Chris.peerInfo.id))
    assert(Alice.getConnectedPeers().includes(Dave.peerInfo.id))

    Bob.emit('peer:connect', Alice.peerInfo)
    Dave.emit('peer:connect', Eve.peerInfo)

    await Bob.crawl()

    // Simulate node failure
    await Bob.stop()
    assert(Chris.getConnectedPeers().includes(Bob.peerInfo.id), 'Chris should know about Bob')
    // Simulates a heartbeat run that kicks out Bob
    Alice._network.networkPeers.blacklistPeer(Bob.peerInfo.id)
    await Alice.crawl()

    assert(
      !Alice.getConnectedPeers().includes(Bob.peerInfo.id),
      'Alice should not add Bob to her networkPeers after blacklisting him'
    )
    assert(
      Alice._network.networkPeers.deletedPeers.some((entry: BlacklistedEntry) => entry.id.equals(Bob.peerInfo.id))
    )

    // Remove Bob from blacklist
    Alice._network.networkPeers.deletedPeers[0].deletedAt -= BLACKLIST_TIMEOUT + 1

    Alice.emit('peer:connect', Chris.peerInfo)

    await Alice.crawl()

    assert(Alice._network.networkPeers.deletedPeers.length == 0)

    // Alice.network.networkPeers.push({
    //   id: Bob.peerInfo.id.toB58String(),
    //   lastSeen: Date.now()
    // })

    await new Promise((resolve) => setTimeout(resolve, 50))

    assert(Alice.getConnectedPeers().includes(Bob.peerInfo.id))

    await Promise.all([Alice.stop(), Bob.stop(), Chris.stop(), Dave.stop(), Eve.stop()])
  })

  it(
    'should crawl the network and timeout while crawling',
    async function () {
      let timeoutCorrectly = false
      let before = Date.now()
      const [Alice, Bob, Chris] = await Promise.all([
        generateNode(),
        generateNode({
          timeoutIntentionally: true
        }),
        generateNode({
          timeoutIntentionally: true
        })
      ])

      await Alice.crawl()
      Alice.emit('peer:connect', Bob.peerInfo)
      await Alice.crawl()
      Bob.emit('peer:connect', Chris.peerInfo)
      await Alice.crawl()

      await new Promise((resolve) => setTimeout(resolve, 100))
      await Bob.stop()
      await Alice.crawl()
      await new Promise((resolve) => setTimeout(resolve, 200))

      timeoutCorrectly = true

      const after = Date.now() - before

      assert(
        timeoutCorrectly && after < 3 * CRAWL_TIMEOUT && after >= 2 * CRAWL_TIMEOUT,
        `Crawling should timeout correctly`
      )

      await Promise.all([Alice.stop(), Bob.stop(), Chris.stop()])
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
