import assert from 'assert'
import { expect } from 'chai'
import { stringToU8a, SIGNATURE_LENGTH } from '@hoprnet/hopr-utils'
import { Address, Ticket, Hash, Balance, PublicKey, Signature, UINT256 } from '.'
import { computeWinningProbability } from '../utils'
import * as fixtures from '../fixtures'
import BN from 'bn.js'
import { randomBytes } from 'crypto'

describe('test ticket construction', function () {
  let userA: Address

  before(async function () {
    userA = PublicKey.fromPrivKey(stringToU8a(fixtures.ACCOUNT_A.privateKey)).toAddress()
  })

  it('should create new ticket', async function () {
    const challenge = new PublicKey(new Uint8Array(PublicKey.SIZE))
    const epoch = UINT256.fromString('1')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN(1))
    const winProb = computeWinningProbability(1)
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
    const ticket = new Ticket(userA, challenge, epoch, index, amount, winProb, channelIteration, signature)

    assert(ticket.counterparty.eq(userA), 'wrong counterparty')
    assert(ticket.challenge.eq(challenge), 'wrong challenge')
    assert(ticket.epoch.toBN().eq(epoch.toBN()), 'wrong epoch')
    assert(ticket.amount.toBN().eq(amount.toBN()), 'wrong amount')
    assert(ticket.winProb.eq(winProb), 'wrong winProb')
    assert(ticket.channelIteration.toBN().eq(channelIteration.toBN()), 'wrong channelIteration')
  })

  it('should generate the hash correctly #1', async function () {
    const expectedHash = new Hash(stringToU8a('0xfeaf3e31d5a4b002f7808945392f3e15fda433824206524aa144ecd7164b6af5'))
    const counterparty = new Address(stringToU8a('0xb3aa2138de698597e2e3f84f60ef415d13731b6f'))
    const challenge = new PublicKey(stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10'))
    const epoch = UINT256.fromString('1')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN('0000000002c68af0bb140000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)

    const ticketA = new Ticket(counterparty, challenge, epoch, index, amount, winProb, channelIteration, signature)

    expect(expectedHash.toHex()).to.eq(ticketA.getHash().toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(
      counterparty,
      challenge,
      UINT256.fromString('2'),
      UINT256.fromString('1'),
      amount,
      winProb,
      channelIteration,
      signature
    )

    assert(!expectedHash.eq(wrongTicket.getHash()), 'ticket hash must be different')
  })

  it('should generate the hash correctly #2', async function () {
    const expectedHash = new Hash(stringToU8a('0x9d0c0bba60c64f656eb34dce0bcb8a932e5b19d55b77476249439ed17b79af04'))
    const counterparty = new Address(stringToU8a('0x32c160a5008e517ce06df4f7d4a39ffc52e049cf'))
    const challenge = new PublicKey(stringToU8a('0x03025fcceb8f338198b866e8bb3621f4cbba8cdcd77b72d95328a296049e9e1230'))
    const epoch = UINT256.fromString('2')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN('000000000de0b6b3a7640000', 16))
    const winProb = new Hash(stringToU8a('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff'))
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)

    const ticketA = new Ticket(counterparty, challenge, epoch, index, amount, winProb, channelIteration, signature)

    expect(expectedHash.toHex()).to.eq(ticketA.getHash().toHex(), 'ticket hash does not match the expected value')

    const wrongTicket = new Ticket(
      counterparty,
      challenge,
      UINT256.fromString('1'),
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
  const userB = PublicKey.fromPrivKey(stringToU8a(fixtures.ACCOUNT_B.privateKey)).toAddress()
  const userAPrivKey = stringToU8a(fixtures.ACCOUNT_A.privateKey)
  const userAPubKey = PublicKey.fromPrivKey(userAPrivKey)

  it('should create new signedTicket using struct', function () {
    const ticket = Ticket.create(
      userB,
      new PublicKey(randomBytes(33)),
      UINT256.fromString('0'),
      UINT256.fromString('1'),
      new Balance(new BN(15)),
      new Hash(new Uint8Array(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))),
      UINT256.fromString('0'),
      userAPrivKey
    )

    assert(ticket.verify(userAPubKey))

    // Mutate ticket and see signature fails
    // @ts-ignore readonly
    ticket.amount = new Balance(new BN(123))
    assert(!ticket.verify(userAPubKey), 'Mutated ticket signatures should not work')
  })
})
