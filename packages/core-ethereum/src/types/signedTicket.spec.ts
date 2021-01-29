import assert from 'assert'
import { randomBytes } from 'crypto'
import { stringToU8a, randomInteger, u8aToHex } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { AccountId, Ticket, Hash, TicketEpoch, Balance, Signature, SignedTicket } from '.'
import { pubKeyToAccountId, privKeyToPubKey } from '../utils'
import * as testconfigs from '../config.spec'
import { SIGNED_TICKET_SIZE } from './signedTicket'

const WIN_PROB = new BN(1)

const generateTicketData = async (receiver: AccountId) => {
  const challenge = new Hash(randomBytes(32))
  const epoch = new TicketEpoch(0)
  const amount = new Balance(15)
  const winProb = new Hash(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
  const onChainSecret = new Hash(randomBytes(27))
  const channelIteration = new TicketEpoch(0)

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
      async (str: string) => await pubKeyToAccountId(await privKeyToPubKey(stringToU8a(str)))
    )
  )

  const [userAPrivKey] = testconfigs.DEMO_ACCOUNTS.slice(0, 2).map((str: string) => stringToU8a(str))

  const userAPubKey = await privKeyToPubKey(stringToU8a(testconfigs.DEMO_ACCOUNTS[0]))

  it('should create new signedTicket using struct', async function () {
    const ticketData = await generateTicketData(userB)

    const ticket = new Ticket(undefined, ticketData)

    const signature = new Signature()

    await ticket.sign(userAPrivKey, undefined, {
      bytes: signature.buffer,
      offset: signature.byteOffset
    })

    const signedTicket = new SignedTicket(undefined, {
      signature,
      ticket
    })

    assert(await signedTicket.verify(userAPubKey))

    assert(new Hash(await signedTicket.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedTicket.ticket.counterparty.eq(userB), 'wrong counterparty')
    assert(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')

    let exponent = randomInteger(0, 8)
    let index = randomInteger(0, signedTicket.serialize().length)

    signedTicket[index] = signedTicket[index] ^ (1 << exponent)

    if (await signedTicket.verify(userAPubKey)) {
      console.log(
        `found invalid signature, <${u8aToHex(signedTicket.serialize())}>, byte #${index}, bit #${exponent}`,
        await signedTicket.verify(userAPubKey)
      )
    }
  })

  it('should create new signedTicket using array', async function () {
    const ticketData = await generateTicketData(userB)

    const ticket = new Ticket(undefined, ticketData)

    const signedTicketA = new SignedTicket(undefined, {
      ticket
    })

    ticket.sign(userAPrivKey, undefined, {
      bytes: signedTicketA.serialize().buffer,
      offset: signedTicketA.signatureOffset
    })

    assert(await signedTicketA.verify(userAPubKey))

    const signedTicketB = new SignedTicket({
      bytes: signedTicketA.serialize().buffer,
      offset: signedTicketA.serialize().byteOffset
    })

    assert(await signedTicketB.verify(userAPubKey))

    assert(new Hash(await signedTicketA.signer).eq(userAPubKey), 'signer incorrect')
    assert(new Hash(await signedTicketB.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedTicketB.ticket.counterparty.eq(userB), 'wrong counterparty')
    assert(signedTicketB.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicketB.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicketB.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicketB.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')

    let exponentA = randomInteger(0, 8)
    let indexA = randomInteger(0, signedTicketA.serialize().length)

    signedTicketA[indexA] = signedTicketA[indexA] ^ (1 << exponentA)

    if (await signedTicketA.verify(userAPubKey)) {
      console.log(
        `found invalid signature, <${u8aToHex(signedTicketA.serialize())}>, byte #${indexA}, bit #${exponentA}`,
        await signedTicketA.verify(userAPubKey)
      )
    }

    let exponentB = randomInteger(0, 8)
    let indexB = randomInteger(0, signedTicketB.serialize().length)

    signedTicketB[indexB] = signedTicketB[indexB] ^ (1 << exponentB)

    if (await signedTicketB.verify(userAPubKey)) {
      console.log(
        `found invalid signature, <${u8aToHex(signedTicketB.serialize())}>, byte #${indexB}, bit #${exponentB}`,
        await signedTicketB.verify(userAPubKey)
      )
    }
  })

  it('should create new signedTicket out of continous memory', async function () {
    const ticketData = await generateTicketData(userB)

    const ticket = new Ticket(undefined, ticketData)

    const signature = new Signature()

    await ticket.sign(userAPrivKey, undefined, {
      bytes: signature.buffer,
      offset: signature.byteOffset
    })

    const offset = randomInteger(1, 31)

    const array = new Uint8Array(SIGNED_TICKET_SIZE + offset).fill(0x00)

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

    assert(new Hash(await signedTicket.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedTicket.ticket.counterparty.eq(userB), 'wrong counterparty')
    assert(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')

    let exponent = randomInteger(0, 8)
    let index = randomInteger(0, signedTicket.serialize().length)

    signedTicket[index] = signedTicket[index] ^ (1 << exponent)

    if (await signedTicket.verify(userAPubKey)) {
      console.log(
        `found invalid signature, <${u8aToHex(signedTicket.serialize())}>, byte #${index}, bit #${exponent}`,
        await signedTicket.verify(userAPubKey)
      )
    }
  })
})
