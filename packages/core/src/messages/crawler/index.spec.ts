import assert from 'assert'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

import { CrawlResponse, CrawlStatus } from '.'

describe('test crawl response generation', function () {
  it('should create a response', async function () {
    const failingResponse = new CrawlResponse(undefined, {
      status: CrawlStatus.FAIL
    })

    assert(failingResponse.status == CrawlStatus.FAIL, 'Check status')

    assert.throws(
      () =>
        new CrawlResponse(undefined, {
          status: CrawlStatus.OK
        }),
      `Should not create successful crawl responses without peerInfos.`
    )

    assert(
      new CrawlResponse({
        bytes: failingResponse.buffer,
        offset: failingResponse.byteOffset
      }).status == CrawlStatus.FAIL,
      'Check status after parsing.'
    )

    const id = await PeerId.create({ keyType: 'secp256k1' })
    const addresses = [new Multiaddr(`/ip4/127.0.0.1/tcp/9091/p2p/${id.toB58String()}`)]

    const successfulResponse = new CrawlResponse(undefined, {
      status: CrawlStatus.OK,
      addresses
    })

    assert(
      successfulResponse.status == CrawlStatus.OK && successfulResponse.addresses[0].getPeerId() == id.toB58String(),
      'Check status & peerInfo'
    )

    const id2 = await PeerId.create({ keyType: 'secp256k1' })
    addresses.push(new Multiaddr(`/ip4/192.168.1.1/tcp/9011/p2p/${id2.toB58String()}`))

    const secondSuccessfulResponse = new CrawlResponse(undefined, {
      status: CrawlStatus.OK,
      addresses
    })

    assert(
      secondSuccessfulResponse.addresses.every((ma: Multiaddr, i: number) => ma.toString() == addresses[i].toString()),
      'Check multiple peerInfos'
    )
  })
})
