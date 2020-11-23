import assert from 'assert'
//import { CRAWL_FAIL_TIMEOUT } from '../constants'
import { Crawler, shouldIncludePeerInCrawlResponse } from './crawler'
import Multiaddr from 'multiaddr'
import NetworkPeerStore from './network-peers'
import { fakePeerId, fakeAddress } from '../test-utils'
import sinon from 'sinon'

const _MOCKS = {}

function generateMock(i) {
  const interactions = {
    interact: (peerId) => _MOCKS[peerId.toB58String()].all().map(fakeAddress)
  } as any

  const getPeer = sinon.fake()
  const putPeer = sinon.fake()
  let id = fakePeerId(i)
  let address = fakeAddress(id)
  let peers = new NetworkPeerStore([])
  const crawler = new Crawler(id, peers, interactions, getPeer, putPeer,
                              (s) => fakePeerId(s))

  _MOCKS[id.toB58String()] = peers
  return {
    id,
    address,
    peers,
    interactions,
    crawler
  }
}

describe('network/crawler test crawler', function () {
  it('should crawl the network and find some nodes', async function () {
    const Alice = generateMock('alice')
    const Bob = generateMock('bob')
    const Chris = generateMock('chris')
    const Dave = generateMock('dave')
    const Eve = generateMock('eve')

    await Alice.crawler.crawl()
    Alice.peers.register(Bob.id)
    await Alice.crawler.crawl()

    assert(Alice.peers.has(Bob.id), 'Alice should know about Bob, 1')
    Bob.peers.register(Chris.id)
    assert(Bob.peers.has(Chris.id), 'Bob should know about Chris')

    await Alice.crawler.crawl()
    assert(Alice.peers.has(Bob.id), 'Alice should know about Bob, 2')
    assert(Alice.peers.has(Chris.id), 'Alice should know about Chris')

    Chris.peers.register(Dave.id)
    await Alice.crawler.crawl()

    assert(Alice.peers.has(Bob.id), 'Alice should know about Bob, 3')
    assert(Alice.peers.has(Chris.id), 'Alice should know about Chris')
    assert(Alice.peers.has(Dave.id), 'Alice should know about Dave')

    Bob.peers.register(Alice.id)
    Dave.peers.register(Eve.id)

    await Bob.crawler.crawl()
    assert(Bob.peers.has(Chris.id), 'Bob should know about Chris')
  })

  it('should crawl the network and timeout while crawling', async function () {
    /*
    this.timeout(5e3)

    let timeoutCorrectly = false
    let before = Date.now()
    const [Alice, Bob, Chris] = await Promise.all([
      generateMocks(),
      generateMocks(),//timeoutIntentionally: true
      generateMocks()//{timeoutIntentionally: true
    ])

    await Alice.crawler.crawl()
    Alice.node.connectionManager.emit('peer:connect', mockConnection(Bob.id, Bob.address))
    await Alice.crawler.crawl()
    Bob.node.connectionManager.emit('peer:connect', mockConnection(Chris.id, Chris.address))
    await Alice.crawler.crawl()

    await new Promise((resolve) => setTimeout(resolve, 100))
    await Bob.node.stop()
    await Alice.crawler.crawl()
    await new Promise((resolve) => setTimeout(resolve, 200))

    timeoutCorrectly = true

    const after = Date.now() - before

    assert(
      timeoutCorrectly && after < 3 * CRAWL_FAIL_TIMEOUT && after >= 2 * CRAWL_FAIL_TIMEOUT,
      `Crawling should timeout correctly`
    )
    */
  })


   it('crawl shouldIncludePeerInCrawlResponse', async () => {
     assert(
       shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/123.4.5.6/tcp/5000'), Multiaddr('/ip4/12.34.56.7/tcp/5000'))
     )
     assert(shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/127.0.0.1/tcp/1000'), Multiaddr('/ip4/127.0.0.1/tcp/5000')))
     assert(
      !shouldIncludePeerInCrawlResponse(Multiaddr('/ip4/127.0.0.1/tcp/5000'), Multiaddr('/ip4/12.34.56.7/tcp/5000'))
     )
   })
})
