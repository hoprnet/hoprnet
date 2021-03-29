import assert from 'assert'
import { expect } from 'chai'
import { stringToU8a, randomInteger } from '@hoprnet/hopr-utils'
import { Address, Ticket, Hash, Balance, UINT256 } from '.'
import { privKeyToPubKey, pubKeyToAddress, computeWinningProbability } from '../utils'
import * as testconfigs from '../config.spec'
import BN from 'bn.js'

describe('test ticket construction', function () {
  let userA: Address

  before(async function () {
    userA = await pubKeyToAddress(await privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0])))
  })

  const generateTicketData = async () => {
    const challenge = new Hash(new Uint8Array(Hash.SIZE))
    const epoch = UINT256.fromString('1')
    const amount = new Balance(new BN(1))
    const winProb = new Hash(computeWinningProbability(1))
    const channelIteration = UINT256.fromString('1')

    return {
      counterparty: userA,
      challenge,
      epoch,
      amount,
      winProb,
      channelIteration
    }
  }

  it('should create new ticket using struct', async function () {
    const ticketData = await generateTicketData()
    const ticket = new Ticket(undefined, ticketData)

    assert(ticket.counterparty.eq(userA), 'wrong counterparty')
    assert(ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(ticket.epoch.toBN().eq(ticketData.epoch.toBN()), 'wrong epoch')
    assert(ticket.amount.toBN().eq(ticketData.amount.toBN()), 'wrong amount')
    assert(ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(ticket.channelIteration.toBN().eq(ticketData.channelIteration.toBN()), 'wrong channelIteration')
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
    assert(ticketB.epoch.toBN().eq(ticketData.epoch.toBN()), 'wrong epoch')
    assert(ticketB.amount.toBN().eq(ticketData.amount.toBN()), 'wrong amount')
    assert(ticketB.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(ticketB.channelIteration.toBN().eq(ticketData.channelIteration.toBN()), 'wrong channelIteration')
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
    assert(ticket.epoch.toBN().eq(ticketData.epoch.toBN()), 'wrong epoch')
    assert(ticket.amount.toBN().eq(ticketData.amount.toBN()), 'wrong amount')
    assert(ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(ticket.channelIteration.toBN().eq(ticketData.channelIteration.toBN()), 'wrong channelIteration')
  })

  it('should generate the hash correctly #1', async function () {
    const expectedHash = new Hash(stringToU8a('0x4d5137ffaad9d5eb8d3cd6252fd8e7fc9b04d24e7f3cedf88d21f569d5a57c86'))
    const counterparty = new Address(stringToU8a('0xb3aa2138de698597e2e3f84f60ef415d13731b6f'))
    const challenge = new Hash(stringToU8a('0x12047ebc6ea03568f4c81b75a4cd827785fe97206d9b22fd5364a9db1f50e234'))
    const epoch = UINT256.fromString('1')
    const amount = new Balance(new BN('0000000002c68af0bb140000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')

    const ticketA = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch,
      amount,
      winProb,
      channelIteration
    })

    const ticketB = new Ticket({
      bytes: ticketA.buffer,
      offset: ticketA.byteOffset
    })

    expect(expectedHash.toHex()).to.eq((await ticketA.hash).toHex(), 'ticket hash does not match the expected value')
    expect(expectedHash.toHex()).to.eq((await ticketB.hash).toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch: UINT256.fromString('2'),
      amount,
      winProb,
      channelIteration
    })

    assert(!expectedHash.eq(await wrongTicket.hash), 'ticket hash must be different')
  })

  it('should generate the hash correctly #2', async function () {
    const expectedHash = new Hash(stringToU8a('0x163a9e28a7c44ab41a6488d8041554404bcc3ff694945886c868c1aecb26e719'))
    const counterparty = new Address(stringToU8a('0x32c160a5008e517ce06df4f7d4a39ffc52e049cf'))
    const challenge = new Hash(stringToU8a('0x91e787e6eef8cb5ddd0815e0f7f91dbe34d2a7bb2e99357039649baf61684c96'))
    const epoch = UINT256.fromString('2')
    const amount = new Balance(new BN('000000000de0b6b3a7640000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')

    const ticketA = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch,
      amount,
      winProb,
      channelIteration
    })

    const ticketB = new Ticket({
      bytes: ticketA.buffer,
      offset: ticketA.byteOffset
    })

    expect(expectedHash.toHex()).to.eq((await ticketA.hash).toHex(), 'ticket hash does not match the expected value')
    expect(expectedHash.toHex()).to.eq((await ticketB.hash).toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(undefined, {
      counterparty,
      challenge,
      epoch: UINT256.fromString('1'),
      amount,
      winProb,
      channelIteration
    })

    assert(!expectedHash.eq(await wrongTicket.hash), 'ticket hash must be different')
  })
})
