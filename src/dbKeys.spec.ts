import assert from 'assert'
import { randomBytes } from 'crypto'
import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'
import { Hash, ChannelId } from './types'
import * as dbKeys from './dbKeys'
import { getId } from './utils'
import { getPrivKeyData } from './utils/testing'
import { Await } from './tsc/utils'

const encoder = new TextEncoder()

describe('test dbKeys', function () {
  let userA: Await<ReturnType<typeof getPrivKeyData>>
  let userB: Await<ReturnType<typeof getPrivKeyData>>
  let channelId: ChannelId
  const challenge = new Hash(randomBytes(32))

  before(async () => {
    userA = await getPrivKeyData(randomBytes(32))
    userB = await getPrivKeyData(randomBytes(32))
    channelId = new ChannelId(await getId(userA.address, userB.address))
  })

  it("should create 'Channel' key", function () {
    const result = dbKeys.Channel(userB.pubKey)
    const expected = u8aConcat(encoder.encode(`payments-channel-`), userB.pubKey)

    assert(u8aEquals(result, expected), 'check channel key creation')
  })

  it("should parse 'Channel' key", function () {
    const key = u8aConcat(encoder.encode(`payments-channel-`), userA.pubKey)
    const result = dbKeys.ChannelKeyParse(key)
    const expected = userA.pubKey

    assert(u8aEquals(result, expected), 'check channel key parsing')
  })

  it("should create 'Challenge' key", function () {
    const result = dbKeys.Challenge(channelId, challenge)
    const expected = u8aConcat(encoder.encode('payments-challenge-'), channelId, encoder.encode('-'), challenge)

    assert(u8aEquals(result, expected), 'check challange key creation')
  })

  it("should parse 'Challenge' key", function () {
    const key = u8aConcat(encoder.encode('payments-challenge-'), channelId, encoder.encode('-'), challenge)
    const [result1, result2] = dbKeys.ChallengeKeyParse(key)
    const expected1 = channelId
    const expected2 = challenge

    assert(u8aEquals(result1, expected1), 'check challange key parsing')
    assert(u8aEquals(result2, expected2), 'check challange key parsing')
  })

  it("should create 'ChannelId' key", function () {
    const sigHash = new Hash(randomBytes(32))
    const result = dbKeys.ChannelId(sigHash)
    const expected = u8aConcat(encoder.encode('payments-channelId-'), sigHash)

    assert(u8aEquals(result, expected), 'check channelId key creation')
  })

  it("should create 'Nonce' key", function () {
    const nonce = new Hash(randomBytes(32))
    const result = dbKeys.Nonce(channelId, nonce)
    const expected = u8aConcat(encoder.encode('payments-nonce-'), channelId, encoder.encode('-'), nonce)

    assert(u8aEquals(result, expected), 'check nonce key creation')
  })

  it("should create 'OnChainSecret' key", function () {
    const result = dbKeys.OnChainSecret()
    const expected = 'payments-onChainSecretIntermediary'

    assert(new TextDecoder().decode(result).startsWith(expected), 'check onChainSecret key creation')
  })

  it("should create 'Ticket' key", function () {
    const result = dbKeys.Ticket(channelId, challenge)
    const expected = u8aConcat(encoder.encode('payments-ticket-'), channelId, encoder.encode('-'), challenge)

    assert(u8aEquals(result, expected), 'check ticket key creation')
  })

  it("should create 'ConfirmedBlockNumber' key", function () {
    const result = dbKeys.ConfirmedBlockNumber()
    const expected = encoder.encode('payments-confirmedBlockNumber')

    assert(u8aEquals(result, expected), 'check confirmedBlockNumber key creation')
  })

  it("should create 'ChannelEntry' key", function () {
    const result = dbKeys.ChannelEntry(userA.address, userB.address)
    const expected = u8aConcat(
      encoder.encode('payments-channelEntry-'),
      userA.address,
      encoder.encode('-'),
      userB.address
    )

    assert(u8aEquals(result, expected), 'check channelEntry key creation')
  })

  it("should parse 'ChannelEntry' key", function () {
    const key = u8aConcat(encoder.encode('payments-channelEntry-'), userA.address, encoder.encode('-'), userB.address)
    const [result1, result2] = dbKeys.ChannelEntryParse(key)
    const expected1 = userA.address
    const expected2 = userB.address

    assert(u8aEquals(result1, expected1), 'check channelEntry key parsing')
    assert(u8aEquals(result2, expected2), 'check channelEntry key parsing')
  })
})
