import { Ganache } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import assert from 'assert'
import { u8aToHex, stringToU8a, u8aEquals, durations } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { getPrivKeyData, createAccountAndFund, createNode } from '../utils/testing.spec'
import { randomBytes } from 'crypto'
import Web3 from 'web3'
import { HoprToken } from '../tsc/web3/HoprToken'
import { Await } from '../tsc/utils'
import { AcknowledgedTicket, Public } from '../types'
import CoreConnector from '..'
import * as testconfigs from '../config.spec'
import * as configs from '../config'

describe('test storing and retrieving tickets', function () {
  const ganache = new Ganache()
  let web3: Web3
  let hoprToken: HoprToken
  let coreConnector: CoreConnector
  let funder: Await<ReturnType<typeof getPrivKeyData>>

  before(async function () {
    this.timeout(durations.seconds(60))

    await ganache.start()
    await migrate()

    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
  })

  after(async function () {
    await ganache.stop()
  })

  afterEach(async function () {
    await coreConnector.stop()
  })

  beforeEach(async function () {
    this.timeout(durations.seconds(10))

    funder = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))

    const userA = await createAccountAndFund(web3, hoprToken, funder)

    coreConnector = await createNode(userA.privKey)
    await coreConnector.db.clear()
    await coreConnector.initOnchainValues()
  })

  const createAcknowledgedTicket = (
    _counterPartyPubKey?: Public,
    _ackTicket?: AcknowledgedTicket
  ): [Public, AcknowledgedTicket] => {
    const counterPartyPubKey = _counterPartyPubKey ?? new Public(randomBytes(Public.SIZE))
    const array = _ackTicket ?? randomBytes(AcknowledgedTicket.SIZE(coreConnector))

    return [
      counterPartyPubKey,
      new AcknowledgedTicket(coreConnector, {
        bytes: array.buffer,
        offset: array.byteOffset,
      }),
    ]
  }

  it('should store ticket', async function () {
    const [counterPartyPubKey, ackTicket] = createAcknowledgedTicket()

    await coreConnector.tickets.store(counterPartyPubKey, ackTicket)

    const storedAckTicket = new Uint8Array(
      await coreConnector.db.get(
        Buffer.from(
          coreConnector.dbKeys.AcknowledgedTicket(counterPartyPubKey, (await ackTicket.signedTicket).ticket.challenge)
        )
      )
    )

    assert(u8aEquals(ackTicket, storedAckTicket), `check that AcknowledgedTicket is stored correctly`)
  })

  it('should store tickets, and retrieve tickets for only counterPartyPubKey1', async function () {
    const [counterPartyPubKey1, ackTicket1] = createAcknowledgedTicket()
    const [, ackTicket2] = createAcknowledgedTicket(counterPartyPubKey1)
    const [counterPartyPubKey3, ackTicket3] = createAcknowledgedTicket()

    await Promise.all([
      coreConnector.tickets.store(counterPartyPubKey1, ackTicket1),
      coreConnector.tickets.store(counterPartyPubKey1, ackTicket2),
      coreConnector.tickets.store(counterPartyPubKey3, ackTicket3),
    ])

    const storedAckTickets = await coreConnector.tickets.get(counterPartyPubKey1)

    assert(storedAckTickets.size === 2, `check getting ackTickets`)
    assert(
      u8aEquals(ackTicket1, storedAckTickets.get(u8aToHex((await ackTicket1.signedTicket).ticket.challenge))),
      `check that ackTicket1 is stored correctly`
    )
    assert(
      u8aEquals(ackTicket2, storedAckTickets.get(u8aToHex((await ackTicket2.signedTicket).ticket.challenge))),
      `check that ackTicket2 is stored correctly`
    )
  })

  it('should store tickets, and retrieve them all', async function () {
    const [counterPartyPubKey1, ackTicket1] = createAcknowledgedTicket()
    const [counterPartyPubKey2, ackTicket2] = createAcknowledgedTicket()
    const [counterPartyPubKey3, ackTicket3] = createAcknowledgedTicket()

    await Promise.all([
      coreConnector.tickets.store(counterPartyPubKey1, ackTicket1),
      coreConnector.tickets.store(counterPartyPubKey2, ackTicket2),
      coreConnector.tickets.store(counterPartyPubKey3, ackTicket3),
    ])

    const storedAckTickets = await coreConnector.tickets.getAll()

    assert(storedAckTickets.size === 3, `check getting all ackTickets`)
    assert(
      u8aEquals(
        ackTicket1,
        storedAckTickets.get(
          u8aToHex(
            coreConnector.dbKeys.AcknowledgedTicket(
              counterPartyPubKey1,
              (await ackTicket1.signedTicket).ticket.challenge
            )
          )
        )
      ),
      `check that ackTicket1 is stored correctly`
    )
    assert(
      u8aEquals(
        ackTicket2,
        storedAckTickets.get(
          u8aToHex(
            coreConnector.dbKeys.AcknowledgedTicket(
              counterPartyPubKey2,
              (await ackTicket2.signedTicket).ticket.challenge
            )
          )
        )
      ),
      `check that ackTicket2 is stored correctly`
    )
    assert(
      u8aEquals(
        ackTicket3,
        storedAckTickets.get(
          u8aToHex(
            coreConnector.dbKeys.AcknowledgedTicket(
              counterPartyPubKey3,
              (await ackTicket3.signedTicket).ticket.challenge
            )
          )
        )
      ),
      `check that ackTicket3 is stored correctly`
    )
  })
})
