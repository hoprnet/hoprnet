import assert from 'assert'
import { randomBytes } from 'crypto'
import { stringToU8a, randomInteger, u8aToHex } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { Address, Ticket, Hash, Balance, SignedTicket, UINT256 } from '.'
import { pubKeyToAddress, privKeyToPubKey } from '../utils'
import * as testconfigs from '../config.spec'

const WIN_PROB = new BN(1)

const generateTicketData = async (receiver: Address) => {
  const challenge = new Hash(randomBytes(32))
  const epoch = new UINT256(0)
  const amount = new Balance(new BN(15))
  const winProb = new Hash(new Uint8Array(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE)))
  const onChainSecret = new Hash(randomBytes(27))
  const channelIteration = new UINT256(0)

  return {
    counterparty: receiver,
    challenge,
    epoch,
    amount,
    winProb,
    onChainSecret,
    channelIteration
  }
}

describe('test signedTicket construction', async function () {
  const [, userB] = await Promise.all(
    testconfigs.DEMO_ACCOUNTS.slice(0, 2).map(
      async (str: string) => await pubKeyToAddress(await privKeyToPubKey(stringToU8a(str)))
    )
  )

  const [userAPrivKey] = testconfigs.DEMO_ACCOUNTS.slice(0, 2).map((str: string) => stringToU8a(str))

  const userAPubKey = await privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0]))

  it('should create new signedTicket using struct', async function () {
    const ticketData = await generateTicketData(userB as Address)

    const ticket = new Ticket(undefined, ticketData)

    const signature = await ticket.sign(userAPrivKey)

    const signedTicket = new SignedTicket(undefined, {
      signature,
      ticket
    })

    assert(await signedTicket.verify(userAPubKey))

    assert(new Hash(await signedTicket.signer).eq(new Hash(userAPubKey)), 'signer incorrect')

    assert(signedTicket.ticket.counterparty.eq(userB as Address), 'wrong counterparty')
    assert(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.toBN().eq(ticketData.amount.toBN()), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')

    let exponent = randomInteger(0, 7)
    let index = randomInteger(0, signedTicket.length - 1)

    signedTicket[index] = signedTicket[index] ^ (1 << exponent)

    if (await signedTicket.verify(userAPubKey)) {
      // @TODO change to assert.fail
      console.log(`found invalid signature, <${u8aToHex(signedTicket)}>, byte #${index}, bit #${exponent}`)
    }
  })

  it('should create new signedTicket using array', async function () {
    const ticketData = await generateTicketData(userB as Address)

    const ticket = new Ticket(undefined, ticketData)

    const signedTicketA = new SignedTicket(undefined, {
      ticket
    })

    const signature = await ticket.sign(userAPrivKey)
    signedTicketA.set(signature, signedTicketA.signatureOffset - signedTicketA.byteOffset)

    assert(await signedTicketA.verify(userAPubKey))

    const signedTicketB = new SignedTicket({
      bytes: signedTicketA.buffer,
      offset: signedTicketA.byteOffset
    })

    assert(await signedTicketB.verify(userAPubKey))

    assert(new Hash(await signedTicketA.signer).eq(new Hash(userAPubKey)), 'signer incorrect')
    assert(new Hash(await signedTicketB.signer).eq(new Hash(userAPubKey)), 'signer incorrect')

    assert(signedTicketB.ticket.counterparty.eq(userB as Address), 'wrong counterparty')
    assert(signedTicketB.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicketB.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicketB.ticket.amount.toBN().eq(ticketData.amount.toBN()), 'wrong amount')
    assert(signedTicketB.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')

    let exponentA = randomInteger(0, 7)
    let indexA = randomInteger(0, signedTicketA.length - 1)

    signedTicketA[indexA] = signedTicketA[indexA] ^ (1 << exponentA)

    if (await signedTicketA.verify(userAPubKey)) {
      // @TODO change to assert.fail
      console.log(`found invalid signature, <${u8aToHex(signedTicketA)}>, byte #${indexA}, bit #${exponentA}`)
    }

    let exponentB = randomInteger(0, 7)
    let indexB = randomInteger(0, signedTicketB.length - 1)

    signedTicketB[indexB] = signedTicketB[indexB] ^ (1 << exponentB)

    if (await signedTicketB.verify(userAPubKey)) {
      // @TODO change to assert.fail
      console.log(`found invalid signature, <${u8aToHex(signedTicketB)}>, byte #${indexB}, bit #${exponentB}`)
    }
  })

  it('should create new signedTicket out of continous memory', async function () {
    const ticketData = await generateTicketData(userB as Address)

    const ticket = new Ticket(undefined, ticketData)

    const signature = await ticket.sign(userAPrivKey)

    const offset = randomInteger(1, 31)

    const array = new Uint8Array(SignedTicket.SIZE + offset).fill(0x00)

    const signedTicket = new SignedTicket(
      {
        bytes: array.buffer,
        offset: array.byteOffset + offset
      },
      {
        ticket,
        signature
      }
    )

    assert(await signedTicket.verify(userAPubKey))

    assert(new Hash(await signedTicket.signer).eq(new Hash(userAPubKey)), 'signer incorrect')

    assert(signedTicket.ticket.counterparty.eq(userB as Address), 'wrong counterparty')
    assert(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.toBN().eq(ticketData.amount.toBN()), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')

    let exponent = randomInteger(0, 7)
    let index = randomInteger(0, signedTicket.length - 1)

    signedTicket[index] = signedTicket[index] ^ (1 << exponent)

    if (await signedTicket.verify(userAPubKey)) {
      // @TODO change to assert.fail
      console.log(`found invalid signature, <${u8aToHex(signedTicket)}>, byte #${index}, bit #${exponent}`)
    }
  })
})
