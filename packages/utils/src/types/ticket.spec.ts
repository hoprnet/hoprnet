import assert from 'assert'
import { expect } from 'chai'
import { stringToU8a, SIGNATURE_LENGTH } from '..'
import { ethers } from 'ethers'
import { Address, Ticket, Hash, Balance, PublicKey, Signature, UINT256 } from '.'
import BN from 'bn.js'
import { randomBytes } from 'crypto'
import { Wallet } from 'ethers'

const ACCOUNT_A = new Wallet('0x4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b')
const ACCOUNT_B = new Wallet('0x18a664889e28a432495758f0522b53b2f04a35f810b78c6ea01db305141bcba2')

describe('test ticket construction', function () {
  let userA: Address

  before(async function () {
    userA = PublicKey.fromPrivKey(stringToU8a(ACCOUNT_A.privateKey)).toAddress()
  })

  it('should create new ticket', async function () {
    const challenge = PublicKey.fromPrivKey(randomBytes(32)).toAddress()
    const epoch = UINT256.fromString('1')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN(1))
    const winProb = Ticket.fromProbability(1)
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
    const ticket = new Ticket(userA, challenge, epoch, index, amount, winProb, channelIteration, signature)

    assert(ticket.counterparty.eq(userA), 'wrong counterparty')
    assert(ticket.challenge.eq(challenge), 'wrong challenge')
    assert(ticket.epoch.toBN().eq(epoch.toBN()), 'wrong epoch')
    assert(ticket.amount.toBN().eq(amount.toBN()), 'wrong amount')
    assert(ticket.winProb.toBN().eq(winProb.toBN()), 'wrong winProb')
    assert(ticket.channelIteration.toBN().eq(channelIteration.toBN()), 'wrong channelIteration')
    // assert(ticket.checkResponse(challengeResponse), 'challengeResponse failed')
    // assert(ticket.isWinningTicket(challengeResponse, challengeResponse, winProb), 'ticket should be winning')
  })

  it('should generate the hash correctly #1', async function () {
    const expectedHash = new Hash(stringToU8a('0xd74dbe2b69fbb48babd30ac1dadaad6d24f412ed287191c44adf932ea36415ab'))
    const counterparty = new Address(stringToU8a('0xb3aa2138de698597e2e3f84f60ef415d13731b6f'))
    const challenge = new PublicKey(
      stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10')
    ).toAddress()
    const epoch = UINT256.fromString('1')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN('0000000002c68af0bb140000', 16))
    const winProb = Ticket.fromProbability(1)
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
    const expectedHash = new Hash(stringToU8a('0x3a7f4f551d68fbb2d093bcf1ec951fb27cae179aab20cb2390a453de1e79afde'))
    const counterparty = new Address(stringToU8a('0x32c160a5008e517ce06df4f7d4a39ffc52e049cf'))
    const challenge = new PublicKey(
      stringToU8a('0x03025fcceb8f338198b866e8bb3621f4cbba8cdcd77b72d95328a296049e9e1230')
    ).toAddress()
    const epoch = UINT256.fromString('2')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN('000000000de0b6b3a7640000', 16))
    const winProb = Ticket.fromProbability(1)
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

describe('test ticket methods', function () {
  it('should convert float to uint256', function () {
    assert(Ticket.fromProbability(0).toBN().isZero())
    assert(Ticket.fromProbability(1).toBN().eq(new BN(ethers.constants.MaxUint256.toString())))
  })
})

describe('test signedTicket construction', async function () {
  const userB = PublicKey.fromPrivKey(stringToU8a(ACCOUNT_B.privateKey)).toAddress()
  const userAPrivKey = stringToU8a(ACCOUNT_A.privateKey)
  const userAPubKey = PublicKey.fromPrivKey(userAPrivKey)

  it('should create new signedTicket using struct', function () {
    const ticket = Ticket.create(
      userB,
      new Address(randomBytes(20)),
      UINT256.fromString('0'),
      UINT256.fromString('1'),
      new Balance(new BN(15)),
      Ticket.fromProbability(1),
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
