import assert from 'assert'
import { expect } from 'chai'
import { Address, Ticket, Hash, Balance, PublicKey, Signature, UINT256 } from '.'
import { computeWinningProbability } from '../utils'
import * as testconfigs from '../config.spec'
import BN from 'bn.js'
import { randomBytes } from 'crypto'
import { stringToU8a } from '@hoprnet/hopr-utils'

describe('test ticket construction', function () {
  let userA: Address

  before(async function () {
    userA = PublicKey.fromPrivKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0])).toAddress()
  })

  it('should create new ticket', async function () {
    const challenge = new PublicKey(new Uint8Array(Hash.SIZE))
    const epoch = UINT256.fromString('1')
    const amount = new Balance(new BN(1))
    const winProb = computeWinningProbability(1)
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(null, 0)
    const ticket = new Ticket(userA, challenge, epoch, amount, winProb, channelIteration, signature)

    assert(ticket.counterparty.eq(userA), 'wrong counterparty')
    assert(ticket.challenge.eq(challenge), 'wrong challenge')
    assert(ticket.epoch.toBN().eq(epoch.toBN()), 'wrong epoch')
    assert(ticket.amount.toBN().eq(amount.toBN()), 'wrong amount')
    assert(ticket.winProb.eq(winProb), 'wrong winProb')
    assert(ticket.channelIteration.toBN().eq(channelIteration.toBN()), 'wrong channelIteration')
  })

  it('should generate the hash correctly #1', async function () {
    const expectedHash = new Hash(stringToU8a('0xb3739c3614045c81352c6a42e11eb7287c657fb4f9a0099f401f5a04ee383f1a'))
    const counterparty = new Address(stringToU8a('0xb3aa2138de698597e2e3f84f60ef415d13731b6f'))
    const challenge = new PublicKey(stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10'))
    const epoch = UINT256.fromString('1')
    const amount = new Balance(new BN('0000000002c68af0bb140000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(null, 0)

    const ticketA = new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration, signature)

    expect(expectedHash.toHex()).to.eq(ticketA.getHash().toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(
      counterparty,
      challenge,
      UINT256.fromString('2'),
      amount,
      winProb,
      channelIteration,
      signature
    )

    assert(!expectedHash.eq(wrongTicket.getHash()), 'ticket hash must be different')
  })

  it('should generate the hash correctly #2', async function () {
    const expectedHash = new Hash(stringToU8a('0x2876159da8c14d8a6551767643f4f6a39814aed87bfe5f41700e57139cc302c9'))
    const counterparty = new Address(stringToU8a('0x32c160a5008e517ce06df4f7d4a39ffc52e049cf'))
    const challenge = new PublicKey(stringToU8a('0x03025fcceb8f338198b866e8bb3621f4cbba8cdcd77b72d95328a296049e9e1230'))
    const epoch = UINT256.fromString('2')
    const amount = new Balance(new BN('000000000de0b6b3a7640000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(null, 0)

    const ticketA = new Ticket(counterparty, challenge, epoch, amount, winProb, channelIteration, signature)

    expect(expectedHash.toHex()).to.eq(ticketA.getHash().toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(
      counterparty,
      challenge,
      UINT256.fromString('1'),
      amount,
      winProb,
      channelIteration,
      signature
    )

    assert(!expectedHash.eq(wrongTicket.getHash()), 'ticket hash must be different')
  })
})

const WIN_PROB = new BN(1)

describe('test signedTicket construction', async function () {
  const userB = await PublicKey.fromPrivKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[1])).toAddress()
  const userAPrivKey = stringToU8a(testconfigs.DEMO_ACCOUNTS[0])
  const userAPubKey = PublicKey.fromPrivKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0]))

  it('should create new signedTicket using struct', async function () {
    const ticket = Ticket.create(
      userB,
      new PublicKey(randomBytes(33)),
      UINT256.fromString('0'),
      new Balance(new BN(15)),
      new Hash(new Uint8Array(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))),
      UINT256.fromString('0'),
      userAPrivKey
    )

    assert(ticket.verify(userAPubKey))
    assert(ticket.getSigner().toHex() == userAPubKey.toHex(), 'signer incorrect')

    // Mutate ticket and see signature fails
    // @ts-ignore readonly
    ticket.amount = new Balance(new BN(123))
    assert(!(await ticket.verify(userAPubKey)), 'Mutated ticket signatures should not work')
  })
})
