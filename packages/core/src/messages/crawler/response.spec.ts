import { CrawlResponse } from './response'
import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import { CrawlStatus } from '.'
import assert from 'assert'
import { u8aEquals } from '@hoprnet/hopr-utils'

describe(`test crawl response generation`, function () {
  it(`should create a crawl response`, async function () {
    const mAddrs = await Promise.all(
      Array.from({ length: 11 }).map(async (_value, index) => {
        const peerId = await PeerId.create({ keyType: 'secp256k1' })
        return new Multiaddr(`/ip4/1.2.3.4/tcp/${9091 + index}/p2p/${peerId.toB58String()}`)
      })
    )
    const response = new CrawlResponse(undefined, {
      status: CrawlStatus.OK,
      addresses: mAddrs
    })

    const continuousMemory = new Uint8Array(2 * response.length)

    continuousMemory.set(response, 2)

    const recoveredResponse = new CrawlResponse({
      bytes: continuousMemory.buffer,
      offset: 2
    })

    assert(response.status == CrawlStatus.OK, `Status must be 'OK'`)
    assert(recoveredResponse.status == CrawlStatus.OK, `Recovered status must be 'OK'`)

    assert(
      u8aEquals(response, recoveredResponse),
      `content should stay the same after serialisation and deserialisation`
    )

    assert(
      response.addresses.length == mAddrs.length &&
        mAddrs.every((addr, index) => {
          return u8aEquals(addr.bytes, response.addresses[index].bytes)
        })
    )

    assert(
      recoveredResponse.addresses.length == mAddrs.length &&
        mAddrs.every((addr, index) => {
          return u8aEquals(addr.bytes, recoveredResponse.addresses[index].bytes)
        })
    )
  })

  it(`should create a empty crawl response`, async function () {
    const response = new CrawlResponse(undefined, {
      status: CrawlStatus.OK,
      addresses: []
    })

    const continuousMemory = new Uint8Array(2 * response.length)

    continuousMemory.set(response, 2)

    const recoveredResponse = new CrawlResponse({
      bytes: continuousMemory.buffer,
      offset: 2
    })

    assert(response.status == CrawlStatus.OK, `Status must be 'OK'`)
    assert(recoveredResponse.status == CrawlStatus.OK, `Recovered status must be 'OK'`)

    assert(
      u8aEquals(response, recoveredResponse),
      `content should stay the same after serialisation and deserialisation`
    )

    assert(response.addresses.length == 0)

    assert(recoveredResponse.addresses.length == 0)
  })

  it(`should create a failing crawl response`, async function () {
    const response = new CrawlResponse(undefined, {
      status: CrawlStatus.FAIL,
      addresses: []
    })

    const continuousMemory = new Uint8Array(2 * response.length)

    continuousMemory.set(response, 2)

    const recoveredResponse = new CrawlResponse({
      bytes: continuousMemory.buffer,
      offset: 2
    })

    assert(response.status == CrawlStatus.FAIL, `Status must be 'OK'`)
    assert(recoveredResponse.status == CrawlStatus.FAIL, `Recovered status must be 'OK'`)

    assert(
      u8aEquals(response, recoveredResponse),
      `content should stay the same after serialisation and deserialisation`
    )

    assert(response.addresses.length == 0)

    assert(recoveredResponse.addresses.length == 0)
  })
})
