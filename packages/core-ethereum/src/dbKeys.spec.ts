import assert from 'assert'
import { randomBytes } from 'crypto'
import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'
import { Hash } from './types'
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
      userA.pubKey.serialize(),
      encoder.encode('-'),
      challenge.serialize()
    )

    assert(u8aEquals(result, expected), 'check AcknowledgedTicket key creation')
  })

  it("should parse 'AcknowledgedTicket' key", function () {
    const key = u8aConcat(
      encoder.encode('tickets-acknowledged-'),
      userA.pubKey.serialize(),
      encoder.encode('-'),
      challenge.serialize()
    )
    const [result1, result2] = dbKeys.AcknowledgedTicketParse(key)
    const expected1 = userA.pubKey.serialize()
    const expected2 = challenge

    assert(u8aEquals(result1.serialize(), expected1), 'check AcknowledgedTicket key parsing')
    assert(result2.eq(expected2), 'check AcknowledgedTicket key parsing')
  })
})
