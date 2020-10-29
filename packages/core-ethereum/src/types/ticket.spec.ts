import assert from 'assert'
import {randomBytes} from 'crypto'
import BN from 'bn.js'
import {stringToU8a, randomInteger, u8aToHex} from '@hoprnet/hopr-utils'
import {AccountId, Ticket, Hash, TicketEpoch, Balance} from '.'
import {privKeyToPubKey, pubKeyToAccountId} from '../utils'
import * as testconfigs from '../config.spec'

const WIN_PROB = new BN(1)

describe('test ticket construction', async function () {
  const userA = await pubKeyToAccountId(await privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0])))

  const generateTicketData = async () => {
    const challenge = new Hash(randomBytes(32))
    const epoch = new TicketEpoch(123)
    const amount = new Balance(34567)
    const winProb = new Hash(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))

    return {
      counterparty: userA,
      challenge,
      epoch,
      amount,
      winProb
    }
  }
  it('should create new ticket using struct', async function () {
    const ticketData = await generateTicketData()

    const ticket = new Ticket(undefined, ticketData)

    assert(ticket.counterparty.eq(userA), 'wrong counterparty')
    assert(ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
  })

  it('should create new ticket using array', async function () {
    const ticketData = await generateTicketData()

    const ticketA = new Ticket(undefined, ticketData)
    const ticketB = new Ticket({
      bytes: ticketA.buffer,
      offset: ticketA.byteOffset
    })

    assert(ticketB.counterparty.eq(userA), 'wrong counterparty')
    assert(ticketB.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(ticketB.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(ticketB.amount.eq(ticketData.amount), 'wrong amount')
    assert(ticketB.winProb.eq(ticketData.winProb), 'wrong winProb')
  })

  it('should create new ticket out of continous memory', async function () {
    const ticketData = await generateTicketData()

    const offset = randomInteger(1, 31)
    const array = new Uint8Array(Ticket.SIZE + offset)

    const ticket = new Ticket(
      {
        bytes: array.buffer,
        offset: array.byteOffset + offset
      },
      ticketData
    )

    assert(ticket.counterparty.eq(ticketData.counterparty), 'wrong counterparty')
    assert(ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
  })

  it('should generate the hash correctly #1', async function () {
    const expectedHash = new Hash(stringToU8a('0x4c663e1dca1587cdab37a0f3be79596af6d5ed8d7cea6ec681ffc242642b1623'))

    const counterparty = new AccountId(stringToU8a('0xb3aa2138de698597e2e3f84f60ef415d13731b6f'))
    const challenge = new Hash(stringToU8a('0x12047ebc6ea03568f4c81b75a4cd827785fe97206d9b22fd5364a9db1f50e234'))
    const epoch = new TicketEpoch(1)
    const amount = new Balance('0000000002c68af0bb140000', 16)

    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))

    const ticketA = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch,
      amount,
      winProb
    })

    const ticketB = new Ticket({
      bytes: ticketA.buffer,
      offset: ticketA.byteOffset
    })

    assert(expectedHash.eq(await ticketA.hash), 'ticket hash does not match the expected value')
    assert(expectedHash.eq(await ticketB.hash), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch: new TicketEpoch(2),
      amount,
      winProb
    })

    assert(!expectedHash.eq(await wrongTicket.hash), 'ticket hash must be different')
  })

  it('should generate the hash correctly #2', async function () {
    const expectedHash = new Hash(stringToU8a('0xb1d503c212b450bbd1d30e7465097df6acadd12ae7241ab415b8db5b8ece70de'))

    const counterparty = new AccountId(stringToU8a('0x32c160a5008e517ce06df4f7d4a39ffc52e049cf'))
    const challenge = new Hash(stringToU8a('0x91e787e6eef8cb5ddd0815e0f7f91dbe34d2a7bb2e99357039649baf61684c96'))
    const epoch = new TicketEpoch(2)
    const amount = new Balance('000000000de0b6b3a7640000', 16)

    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))

    const ticketA = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch,
      amount,
      winProb
    })

    const ticketB = new Ticket({
      bytes: ticketA.buffer,
      offset: ticketA.byteOffset
    })

    assert(expectedHash.eq(await ticketA.hash), 'ticket hash does not match the expected value')
    assert(expectedHash.eq(await ticketB.hash), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch: new TicketEpoch(1),
      amount,
      winProb
    })

    assert(!expectedHash.eq(await wrongTicket.hash), 'ticket hash must be different')
  })
})
