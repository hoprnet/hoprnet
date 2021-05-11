import { HoprDB } from './db'
import { PublicKey, UnacknowledgedTicket, Ticket } from '.'
import { randomBytes } from 'crypto'

import assert from 'assert'
import { AcknowledgedTicket, Address, Balance, Hash, UINT256, HalfKey, Response, Opening } from './types'
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
    const keyChallenge = new HalfKey(randomBytes(HalfKey.SIZE)).toChallenge()
    await db.storeUnacknowledgedTickets(
      keyChallenge,
      new UnacknowledgedTicket(createMockedTicket(), new HalfKey(randomBytes(HalfKey.SIZE)))
    )

    assert((await db.getTickets()).length == 1, `DB should find one ticket`)

    const ticket = await db.getUnacknowledgedTicketsByKey(keyChallenge)
    assert(ticket != null)

    await db.replaceTicketWithAcknowledgement(
      keyChallenge,
      new AcknowledgedTicket(ticket.ticket, new Response(randomBytes(Hash.SIZE)), new Opening(randomBytes(Hash.SIZE)))
    )

    assert((await db.getTickets()).length == 1, `DB should find one ticket`)

    assert(
      (await db.getUnacknowledgedTicketsByKey(keyChallenge)) == undefined,
      `DB should not contain any unacknowledgedTicket`
    )

    assert((await db.getAcknowledgedTickets()).length == 1, `DB should contain exactly one acknowledged ticket`)
  })

  it('block number workflow', async function () {
    const initialBlockNumber = await db.getLatestBlockNumber()

    assert(initialBlockNumber == 0, `initial block number must be set to 0`)

    const blockNumber = new BN(23)
    await db.updateLatestBlockNumber(blockNumber)

    const latestBlockNumber = await db.getLatestBlockNumber()

    assert(blockNumber.eqn(latestBlockNumber), `block number must be updated`)
  })
})
