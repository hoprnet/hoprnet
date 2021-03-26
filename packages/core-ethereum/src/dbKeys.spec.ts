import assert from 'assert'
import { randomBytes } from 'crypto'
import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'
import { Hash, Address } from './types'
import * as dbKeys from './dbKeys'
import { getId } from './utils'
import { getPrivKeyData } from './utils/testing.spec'
import { Await } from './tsc/utils'

const encoder = new TextEncoder()

describe('test dbKeys', function () {
  let userA: Await<ReturnType<typeof getPrivKeyData>>
  let userB: Await<ReturnType<typeof getPrivKeyData>>
  let channelId: Hash
  const challenge = new Hash(randomBytes(32))

  before(async () => {
    userA = await getPrivKeyData(randomBytes(32))
    userB = await getPrivKeyData(randomBytes(32))
    channelId = await getId(userA.address, userB.address)
  })

  it("should create 'Channel' key", function () {
    const result = dbKeys.Channel(new Address(userB.pubKey))
    const expected = u8aConcat(encoder.encode(`payments-channel-`), userB.pubKey)

    assert(u8aEquals(result, expected), 'check channel key creation')
  })

  it("should parse 'Channel' key", function () {
    const key = u8aConcat(encoder.encode(`payments-channel-`), userA.pubKey)
    const result = dbKeys.ChannelKeyParse(key)
    const expected = userA.pubKey

    assert(u8aEquals(result.serialize(), expected), 'check channel key parsing')
  })

  it("should create 'Challenge' key", function () {
    const result = dbKeys.Challenge(channelId, challenge)
    const expected = u8aConcat(
      encoder.encode('payments-challenge-'),
      channelId.serialize(),
      encoder.encode('-'),
      challenge.serialize()
    )

    assert(u8aEquals(result, expected), 'check challenge key creation')
  })

  it("should parse 'Challenge' key", function () {
    const key = u8aConcat(
      encoder.encode('payments-challenge-'),
      channelId.serialize(),
      encoder.encode('-'),
      challenge.serialize()
    )
    const [result1, result2] = dbKeys.ChallengeKeyParse(key)
    const expected1 = channelId
    const expected2 = challenge

    assert(result1.eq(expected1), 'check challenge key parsing')
    assert(result2.eq(expected2), 'check challenge key parsing')
  })

  it("should create 'ChannelId' key", function () {
    const sigHash = new Hash(randomBytes(32))
    const result = dbKeys.ChannelId(sigHash)
    const expected = u8aConcat(encoder.encode('payments-channelId-'), sigHash.serialize())

    assert(u8aEquals(result, expected), 'check channelId key creation')
  })

  it("should create 'Nonce' key", function () {
    const nonce = new Hash(randomBytes(32))
    const result = dbKeys.Nonce(channelId, nonce)
    const expected = u8aConcat(
      encoder.encode('payments-nonce-'),
      channelId.serialize(),
      encoder.encode('-'),
      nonce.serialize()
    )

    assert(u8aEquals(result, expected), 'check nonce key creation')
  })

  it("should create 'OnChainSecret' key", function () {
    const result = dbKeys.OnChainSecret()
    const expected = 'payments-onChainSecretIntermediary'

    assert(new TextDecoder().decode(result).startsWith(expected), 'check onChainSecret key creation')
  })

  it("should create 'AcknowledgedTicket' key", function () {
    const result = dbKeys.AcknowledgedTicket(userA.pubKey, challenge)
    const expected = u8aConcat(
      encoder.encode('tickets-acknowledged-'),
      userA.pubKey,
      encoder.encode('-'),
      challenge.serialize()
    )

    assert(u8aEquals(result, expected), 'check AcknowledgedTicket key creation')
  })

  it("should parse 'AcknowledgedTicket' key", function () {
    const key = u8aConcat(
      encoder.encode('tickets-acknowledged-'),
      userA.pubKey,
      encoder.encode('-'),
      challenge.serialize()
    )
    const [result1, result2] = dbKeys.AcknowledgedTicketParse(key)
    const expected1 = userA.pubKey
    const expected2 = challenge

    assert(u8aEquals(result1, expected1), 'check AcknowledgedTicket key parsing')
    assert(result2.eq(expected2), 'check AcknowledgedTicket key parsing')
  })

  it("should create 'LatestBlockNumber' key", function () {
    const result = dbKeys.LatestBlockNumber()
    const expected = encoder.encode('payments-latestBlockNumber')

    assert(u8aEquals(result, expected), 'check latestBlockNumber key creation')
  })

  it("should create 'LatestConfirmedSnapshot' key", function () {
    const result = dbKeys.LatestConfirmedSnapshot()
    const expected = encoder.encode('payments-latestConfirmedSnapshot')

    assert(u8aEquals(result, expected), 'check latestConfirmedSnapshot key creation')
  })

  it("should create 'ChannelEntry' key", function () {
    const result = dbKeys.ChannelEntry(userA.pubKey, userB.pubKey)
    const expected = u8aConcat(
      encoder.encode('payments-channelEntry-'),
      userA.pubKey,
      encoder.encode('-'),
      userB.pubKey
    )

    assert(u8aEquals(result, expected), 'check channelEntry key creation')
  })

  it("should parse 'ChannelEntry' key", function () {
    const key = u8aConcat(encoder.encode('payments-channelEntry-'), userA.pubKey, encoder.encode('-'), userB.pubKey)
    const [result1, result2] = dbKeys.ChannelEntryParse(key)
    const expected1 = userA.pubKey
    const expected2 = userB.pubKey

    assert(u8aEquals(result1, expected1), 'check channelEntry key parsing')
    assert(u8aEquals(result2, expected2), 'check channelEntry key parsing')
  })
})
