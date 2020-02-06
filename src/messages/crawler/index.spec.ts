import assert from 'assert'

import PeerInfo from 'peer-info'

import { CrawlResponse, Status } from '.'

describe('test crawl response generation', function() {
  it('should create a response', async function() {
    const failingResponse = new CrawlResponse(undefined, {
      status: Status.FAIL
    })

    assert(failingResponse.status == Status.FAIL, 'Check status')

    assert(new CrawlResponse(failingResponse).status == Status.FAIL, 'Check status after parsing.')

    const peerInfos = [await PeerInfo.create()]

    const successfulResponse = new CrawlResponse(undefined, {
      status: Status.OK,
      peerInfos
    })

    assert(
      successfulResponse.status == Status.OK && (await successfulResponse.peerInfos)[0].id.toB58String() == peerInfos[0].id.toB58String(),
      'Check status & peerInfo'
    )

    assert((await successfulResponse.peerInfos)[0].id.toB58String() == peerInfos[0].id.toB58String(), 'Check peerInfo after parsing')

    peerInfos.push(await PeerInfo.create())

    const secondSuccessfulResponse = new CrawlResponse(undefined, {
      status: Status.OK,
      peerInfos
    })

    assert(
      (await secondSuccessfulResponse.peerInfos).every((peerInfo: PeerInfo, index: number) => peerInfos[index].id.toB58String() == peerInfo.id.toB58String()),
      'Check multiple peerInfos'
    )
  })
})
