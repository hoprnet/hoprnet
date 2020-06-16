import assert from 'assert'

import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import { CrawlResponse, CrawlStatus } from '.'

describe('test crawl response generation', function () {
  it('should create a response', async function () {
    const failingResponse = new CrawlResponse(undefined, {
      status: CrawlStatus.FAIL,
    })

    assert(failingResponse.status == CrawlStatus.FAIL, 'Check status')

    assert.throws(
      () =>
        new CrawlResponse(undefined, {
          status: CrawlStatus.OK,
        }),
      `Should not create successful crawl responses without peerInfos.`
    )

    assert(new CrawlResponse(failingResponse).status == CrawlStatus.FAIL, 'Check status after parsing.')

    const peerInfos = [await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' }))]

    const successfulResponse = new CrawlResponse(undefined, {
      status: CrawlStatus.OK,
      peerInfos,
    })

    assert(
      successfulResponse.status == CrawlStatus.OK &&
        (await successfulResponse.peerInfos)[0].id.toB58String() == peerInfos[0].id.toB58String(),
      'Check status & peerInfo'
    )

    assert(
      (await successfulResponse.peerInfos)[0].id.toB58String() == peerInfos[0].id.toB58String(),
      'Check peerInfo after parsing'
    )

    peerInfos.push(await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })))

    const secondSuccessfulResponse = new CrawlResponse(undefined, {
      status: CrawlStatus.OK,
      peerInfos,
    })

    assert(
      (await secondSuccessfulResponse.peerInfos).every(
        (peerInfo: PeerInfo, index: number) => peerInfos[index].id.toB58String() == peerInfo.id.toB58String()
      ),
      'Check multiple peerInfos'
    )
  })
})
