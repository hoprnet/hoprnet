import assert from 'assert'
import { CRAWL_FAIL_TIMEOUT } from '../constants'
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
  const crawler = new Crawler(id, peers, interactions, getPeer, putPeer, (s) => fakePeerId(s))

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
    let clock = sinon.useFakeTimers(Date.now())
    const Alice = generateMock('alice')
    const Bob = generateMock('bob')
    const Chris = generateMock('chris')
    const oldInteract = Alice.interactions.interact
    Alice.interactions.interact = (id) => {
      if (id.equals(Bob.id)) {
        return new Promise(() => {}) // Never resolve
      }
      return oldInteract(id)
    }

    await Alice.crawler.crawl() // No-op
    Alice.peers.register(Bob.id)
    let prom = Alice.crawler.crawl() // Crawl bob, and timeout
    clock.tick(CRAWL_FAIL_TIMEOUT * 2)
    await prom
    assert(!Alice.peers.has(Chris.id), 'Alice does not know about Chris')
    Bob.peers.register(Chris.id)
    let prom2 = Alice.crawler.crawl() // Crawl bob and timeout again.
    clock.tick(CRAWL_FAIL_TIMEOUT * 2)
    await prom2
    assert(!Alice.peers.has(Chris.id), 'Alice does not know about Chris')
    clock.restore()
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
