import assert from 'assert'
import { randomBytes } from 'crypto'
import { stringToU8a, randomInteger } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { AccountId, Ticket, Hash, TicketEpoch, Balance, Signature, SignedTicket } from '.'
import * as utils from '../utils'
import * as testconfigs from '../config.spec'

const [userA, userB] = testconfigs.DEMO_ACCOUNTS.map((str: string) => new AccountId(stringToU8a(str)))
const WIN_PROB = new BN(1)

const generateTicketData = async () => {
  const channelId = new Hash(await utils.getId(userA, userB))
  const challenge = new Hash(randomBytes(32))
  const epoch = new TicketEpoch(0)
  const amount = new Balance(15)
  const winProb = new Hash(new BN(new Uint8Array(Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', Hash.SIZE))
  const onChainSecret = new Hash(randomBytes(32))

  return {
    channelId,
    challenge,
    epoch,
    amount,
    winProb,
    onChainSecret,
  }
}

describe('test signedTicket construction', async function () {
  const userAPubKey = await utils.privKeyToPubKey(userA)

  it('should create new signedTicket using struct', async function () {
    const ticketData = await generateTicketData()

    const ticket = new Ticket(undefined, ticketData)

    const signature = new Signature()

    await ticket.sign(userA, undefined, {
      bytes: signature.buffer,
      offset: signature.byteOffset,
    })

    const signedTicket = new SignedTicket(undefined, {
      signature,
      ticket,
    })

    assert(await signedTicket.verify(userAPubKey))

    assert(new Hash(await signedTicket.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedTicket.ticket.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(signedTicket.ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })

  it('should create new signedTicket using array', async function () {
    const ticketData = await generateTicketData()

    const ticket = new Ticket(undefined, ticketData)

    const signedTicketA = new SignedTicket(undefined, {
      ticket,
    })

    ticket.sign(userA, undefined, {
      bytes: signedTicketA.buffer,
      offset: signedTicketA.signatureOffset,
    })

    assert(await signedTicketA.verify(userAPubKey))

    const signedTicketB = new SignedTicket({
      bytes: signedTicketA.buffer,
      offset: signedTicketA.byteOffset,
    })

    assert(await signedTicketB.verify(userAPubKey))

    assert(new Hash(await signedTicketA.signer).eq(userAPubKey), 'signer incorrect')
    assert(new Hash(await signedTicketB.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedTicketB.ticket.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(signedTicketB.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicketB.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicketB.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicketB.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(signedTicketB.ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })

  it('should create new signedTicket out of continous memory', async function () {
    const ticketData = await generateTicketData()

    const ticket = new Ticket(undefined, ticketData)

    const signature = new Signature()

    await ticket.sign(userA, undefined, {
      bytes: signature.buffer,
      offset: signature.byteOffset,
    })

    const offset = randomInteger(1, 31)

    const array = new Uint8Array(SignedTicket.SIZE + offset).fill(0x00)

    const signedTicket = new SignedTicket(
      {
        bytes: array.buffer,
        offset: array.byteOffset + offset,
      },
      {
        ticket,
        signature,
      }
    )

    assert(await signedTicket.verify(userAPubKey))

    assert(new Hash(await signedTicket.signer).eq(userAPubKey), 'signer incorrect')

    assert(signedTicket.ticket.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(signedTicket.ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })
})
