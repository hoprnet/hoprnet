import assert from 'assert'
import { expect } from 'chai'
import { stringToU8a, SIGNATURE_LENGTH } from '..'
import { Address, Ticket, Hash, Balance, PublicKey, Signature, UINT256, Response, Challenge } from '.'
import BN from 'bn.js'
import { randomBytes } from 'crypto'
import { Wallet } from 'ethers'
import { INVERSE_TICKET_WIN_PROB, PRICE_PER_PACKET } from '../constants'

const ACCOUNT_A = new Wallet('0x4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b')
const ACCOUNT_B = new Wallet('0x18a664889e28a432495758f0522b53b2f04a35f810b78c6ea01db305141bcba2')

describe('test ticket construction', function () {
  let userA: Address

  before(async function () {
    userA = PublicKey.fromPrivKey(stringToU8a(ACCOUNT_A.privateKey)).toAddress()
  })

  it('should create new ticket', async function () {
    const challenge = new Response(Uint8Array.from(randomBytes(32))).toChallenge().toEthereumChallenge()
    const epoch = UINT256.fromString('1')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN(1))
    const winProb = UINT256.fromInverseProbability(new BN(1))
    const channelIteration = UINT256.fromString('1')
    const signature = new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
    const ticket = new Ticket(userA, challenge, epoch, index, amount, winProb, channelIteration, signature)

    assert(ticket.counterparty.eq(userA), 'wrong counterparty')
    assert(ticket.challenge.eq(challenge), 'wrong challenge')
    assert(ticket.epoch.toBN().eq(epoch.toBN()), 'wrong epoch')
    assert(ticket.amount.toBN().eq(amount.toBN()), 'wrong amount')
    assert(ticket.winProb.toBN().eq(winProb.toBN()), 'wrong winProb')
    assert(ticket.channelIteration.toBN().eq(channelIteration.toBN()), 'wrong channelIteration')
  })

  it('should generate the hash correctly #1', async function () {
    const expectedHash = new Hash(stringToU8a('0x44b673afb0846f969a106fcbfcc178ce7c4f40e55134a2e6717f6738062756f7'))
    const counterparty = new Address(stringToU8a('0xb3aa2138de698597e2e3f84f60ef415d13731b6f'))
    const challenge = new Challenge(
      stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10')
    ).toEthereumChallenge()
    const epoch = UINT256.fromString('1')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN('0000000002c68af0bb140000', 16))
    const winProb = UINT256.fromInverseProbability(new BN(1))
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
    const expectedHash = new Hash(stringToU8a('0x1ee41a36038629f6a098adff754b044d00716d58e60af6b7d0f92f4a8fea90d3'))
    const counterparty = new Address(stringToU8a('0x32c160a5008e517ce06df4f7d4a39ffc52e049cf'))
    const challenge = new Challenge(
      stringToU8a('0x03025fcceb8f338198b866e8bb3621f4cbba8cdcd77b72d95328a296049e9e1230')
    ).toEthereumChallenge()
    const epoch = UINT256.fromString('2')
    const index = UINT256.fromString('1')
    const amount = new Balance(new BN('000000000de0b6b3a7640000', 16))
    const winProb = UINT256.fromInverseProbability(new BN(1))
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
  it('probability generation - edge case', function () {
    assert.throws(() => UINT256.fromInverseProbability(new BN(-1)))

    const maxUint256 = new BN(new Uint8Array(UINT256.SIZE).fill(0xff))

    assert(UINT256.fromInverseProbability(maxUint256).toBN().eqn(1))

    assert(UINT256.fromInverseProbability(new BN(1)).toBN().eq(maxUint256))
  })
})

describe('test signedTicket construction', function () {
  const userB = PublicKey.fromPrivKey(stringToU8a(ACCOUNT_B.privateKey)).toAddress()
  const userAPrivKey = stringToU8a(ACCOUNT_A.privateKey)
  const userAPubKey = PublicKey.fromPrivKey(userAPrivKey)

  it('should create new signedTicket using struct', function () {
    const ticket = Ticket.create(
      userB,
      new Response(Uint8Array.from(randomBytes(32))).toChallenge(),
      UINT256.fromString('0'),
      UINT256.fromString('1'),
      new Balance(new BN(15)),
      UINT256.fromInverseProbability(new BN(1)),
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

describe('test getPathPosition', function () {
  const userB = PublicKey.fromPrivKey(stringToU8a(ACCOUNT_B.privateKey)).toAddress()
  const userAPrivKey = stringToU8a(ACCOUNT_A.privateKey)

  it('check that path position detection works on multiple positions', function () {
    for (let pathLength = 0; pathLength < 4; pathLength++) {
      const ticket = Ticket.create(
        userB,
        new Response(Uint8Array.from(randomBytes(32))).toChallenge(),
        UINT256.fromString('0'),
        UINT256.fromString('1'),
        new Balance(INVERSE_TICKET_WIN_PROB.mul(PRICE_PER_PACKET).muln(pathLength)),
        UINT256.fromInverseProbability(INVERSE_TICKET_WIN_PROB),
        UINT256.fromString('0'),
        userAPrivKey
      )

      assert(ticket.getPathPosition() == pathLength)
    }
  })
})
