import assert from 'assert'
import { expect } from 'chai'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { Address, Ticket, Hash, Balance, UINT256 } from '.'
import { privKeyToPubKey, pubKeyToAddress, computeWinningProbability } from '../utils'
import * as testconfigs from '../config.spec'
import BN from 'bn.js'

describe('test ticket construction', function () {
  let userA: Address

  before(async function () {
    userA = await pubKeyToAddress(await privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0])))
  })

  it('should create new ticket', async function () {
    const challenge = new Hash(new Uint8Array(Hash.SIZE))
    const epoch = UINT256.fromString('1')
    const amount = new Balance(new BN(1))
    const winProb = computeWinningProbability(1)
    const channelIteration = UINT256.fromString('1')
    const ticket = new Ticket(userA, challenge, epoch, amount, winProb, channelIteration)

    assert(ticket.counterparty.eq(userA), 'wrong counterparty')
    assert(ticket.challenge.eq(challenge), 'wrong challenge')
    assert(ticket.epoch.toBN().eq(epoch.toBN()), 'wrong epoch')
    assert(ticket.amount.toBN().eq(amount.toBN()), 'wrong amount')
    assert(ticket.winProb.eq(winProb), 'wrong winProb')
    assert(ticket.channelIteration.toBN().eq(channelIteration.toBN()), 'wrong channelIteration')
  })

  it('should generate the hash correctly #1', async function () {
    const expectedHash = new Hash(stringToU8a('0x4d5137ffaad9d5eb8d3cd6252fd8e7fc9b04d24e7f3cedf88d21f569d5a57c86'))
    const counterparty = new Address(stringToU8a('0xb3aa2138de698597e2e3f84f60ef415d13731b6f'))
    const challenge = new Hash(stringToU8a('0x12047ebc6ea03568f4c81b75a4cd827785fe97206d9b22fd5364a9db1f50e234'))
    const epoch = UINT256.fromString('1')
    const amount = new Balance(new BN('0000000002c68af0bb140000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')

    const ticketA = new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration)

    expect(expectedHash.toHex()).to.eq(ticketA.getHash().toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(counterparty, challenge, UINT256.fromString('2'), amount, winProb, channelIteration)

    assert(!expectedHash.eq(wrongTicket.getHash()), 'ticket hash must be different')
  })

  it('should generate the hash correctly #2', async function () {
    const expectedHash = new Hash(stringToU8a('0x163a9e28a7c44ab41a6488d8041554404bcc3ff694945886c868c1aecb26e719'))
    const counterparty = new Address(stringToU8a('0x32c160a5008e517ce06df4f7d4a39ffc52e049cf'))
    const challenge = new Hash(stringToU8a('0x91e787e6eef8cb5ddd0815e0f7f91dbe34d2a7bb2e99357039649baf61684c96'))
    const epoch = UINT256.fromString('2')
    const amount = new Balance(new BN('000000000de0b6b3a7640000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')

    const ticketA = new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration)

    expect(expectedHash.toHex()).to.eq(ticketA.getHash().toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(counterparty, challenge, UINT256.fromString('1'), amount, winProb, channelIteration)

    assert(!expectedHash.eq(wrongTicket.getHash()), 'ticket hash must be different')
  })
})
