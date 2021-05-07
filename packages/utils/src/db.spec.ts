import { HoprDB } from './db'
import { PublicKey, UnacknowledgedTicket, Ticket } from '.'
import { randomBytes } from 'crypto'

import assert from 'assert'
import { AcknowledgedTicket, Address, Balance, Hash, UINT256 } from './types'
import BN from 'bn.js'

function createMockedTicket() {
  return Ticket.create(
    new Address(randomBytes(Address.SIZE)),
    PublicKey.fromPrivKey(randomBytes(32)),
    UINT256.fromString('0'),
    UINT256.fromString('0'),
    new Balance(new BN(0)),
    UINT256.fromProbability(1),
    UINT256.fromString('1'),
    randomBytes(32)
  )
}
describe(`database tests`, function () {
  let db: HoprDB

  beforeEach(function () {
    db = HoprDB.createMock()
  })
  afterEach(async function () {
    await db.close()
  })

  it('hasPacket', async function () {
    const packetTag = randomBytes(5)

    const present = await db.checkAndSetPacketTag(packetTag)

    assert(present == false, `Packet tag must not be present`)

    const secondTry = await db.checkAndSetPacketTag(packetTag)

    assert(secondTry == true, `Packet tag must be set`)
  })

  it('ticket workflow', async function () {
    const key = PublicKey.fromPrivKey(randomBytes(32))
    await db.storeUnacknowledgedTickets(
      key,
      new UnacknowledgedTicket(createMockedTicket(), new Hash(new Uint8Array(Hash.SIZE)))
    )

    assert((await db.getTickets()).length == 1, `DB should find one ticket`)

    const ticket = await db.getUnacknowledgedTicketsByKey(key)
    assert(ticket != null)

    await db.replaceTicketWithAcknowledgement(
      key,
      new AcknowledgedTicket(ticket.ticket, new Hash(randomBytes(Hash.SIZE)), new Hash(randomBytes(Hash.SIZE)))
    )

    assert((await db.getTickets()).length == 1, `DB should find one ticket`)

    const empty = await db.getUnacknowledgedTicketsByKey(key)

    console.log(empty)
    assert(empty == undefined)

    // const ackedTicket = await db.getA
  })
})
