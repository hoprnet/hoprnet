import assert from 'assert'
import { randomBytes } from 'crypto'
import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'
import { Hash } from './types'
import * as dbKeys from './dbKeys'
import { getPrivKeyData } from './utils/testing.spec'
import { Await } from './tsc/utils'

const encoder = new TextEncoder()

describe('test dbKeys', function () {
  let userA: Await<ReturnType<typeof getPrivKeyData>>
  const challenge = new Hash(randomBytes(32))

  before(async () => {
    userA = await getPrivKeyData(randomBytes(32))
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
