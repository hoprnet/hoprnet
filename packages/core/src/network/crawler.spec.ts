import assert from 'assert'
import PeerId from 'peer-id'
import type { Connection } from 'libp2p'

import { CRAWL_TIMEOUT /*, shouldIncludePeerInCrawlResponse */ } from './crawler'
import { Crawler as CrawlerInteraction } from '../interactions/network/crawler'
import Multiaddr from 'multiaddr'
import { Network } from './index'
import { Interactions } from '../interactions'
import { generateLibP2PMock } from '../test-utils'

let mockConnection = (p: PeerId, addr: Multiaddr): Connection => {
  return { remotePeer: p, remoteAddr: addr } as Connection
}

async function generateMocks(options?: { timeoutIntentionally: boolean }, addr = '/ip4/0.0.0.0/tcp/0') {
  const { node, address } = await generateLibP2PMock(addr)

  await node.start()

  const interactions = {
    network: {
      crawler: new CrawlerInteraction(node, (conn) => {
        return network.crawler.handleCrawlRequest(conn)
      })
    }
  } as Interactions<any>

  const network = new Network(node, interactions, {} as any, { crawl: options })
  node.connectionManager.on('peer:connect', (conn: Connection) => {
    network.networkPeers.register(conn.remotePeer)
    node.peerStore.addressBook.add(conn.remotePeer, [conn.remoteAddr])
  })

  return {
    node,
    address,
    interactions,
    network
  }
}

describe('network/crawler test crawler', function () {
  it('should crawl the network and find some nodes', async function () {
    this.timeout(5e3)

    const [Alice, Bob, Chris, Dave, Eve] = await Promise.all([
      generateMocks(),
      generateMocks(),
      generateMocks(),
      generateMocks(),
      generateMocks()
    ])

    await Alice.network.crawler.crawl()
    Alice.node.connectionManager.emit('peer:connect', mockConnection(Bob.node.peerId, Bob.address))
    await Alice.network.crawler.crawl()

    assert(Alice.network.networkPeers.has(Bob.node.peerId), 'Alice should know about Bob, 1')

    Bob.node.connectionManager.emit('peer:connect', mockConnection(Chris.node.peerId, Chris.address))
    assert(Bob.network.networkPeers.has(Chris.node.peerId), 'Bob should know about Chris')

    await Alice.network.crawler.crawl()
    assert(Alice.network.networkPeers.has(Bob.node.peerId), 'Alice should know about Bob, 2')
    assert(Alice.network.networkPeers.has(Chris.node.peerId), 'Alice should know about Chris')

    Chris.node.connectionManager.emit('peer:connect', mockConnection(Dave.node.peerId, Dave.address))
    await Alice.network.crawler.crawl()

    assert(Alice.network.networkPeers.has(Bob.node.peerId), 'Alice should know about Bob, 3')
    assert(Alice.network.networkPeers.has(Chris.node.peerId), 'Alice should know about Chris')
    assert(Alice.network.networkPeers.has(Dave.node.peerId), 'Alice should know about Dave')

    Bob.node.connectionManager.emit('peer:connect', mockConnection(Alice.node.peerId, Alice.address))
    Dave.node.connectionManager.emit('peer:connect', mockConnection(Eve.node.peerId, Eve.address))

    await Bob.network.crawler.crawl()

    // Simulate node failure
    await Bob.node.stop()
    assert(Chris.network.networkPeers.has(Bob.node.peerId), 'Chris should know about Bob')
    await Promise.all([Alice.node.stop(), Bob.node.stop(), Chris.node.stop(), Dave.node.stop(), Eve.node.stop()])
  })

  it('should crawl the network and timeout while crawling', async function () {
    this.timeout(5e3)

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
    Alice.node.connectionManager.emit('peer:connect', mockConnection(Bob.node.peerId, Bob.address))
    await Alice.network.crawler.crawl()
    Bob.node.connectionManager.emit('peer:connect', mockConnection(Chris.node.peerId, Chris.address))
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
  })
  // @TODO redo crawl filtering
  // it('shouldIncludePeerInCrawlResponse', async () => {
  //   assert(
  //     shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/123.4.5.6/tcp/5000'), Multiaddr('/ip4/12.34.56.7/tcp/5000'))
  //   )
  //   assert(shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/127.0.0.1/tcp/1000'), Multiaddr('/ip4/127.0.0.1/tcp/5000')))
  //   assert(
  //     !shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/127.0.0.1/tcp/5000'), Multiaddr('/ip4/12.34.56.7/tcp/5000'))
  //   )
  // })
})
