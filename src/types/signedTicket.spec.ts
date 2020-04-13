import assert from 'assert'
import { randomBytes } from 'crypto'
import { stringToU8a } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { Ticket, Hash, TicketEpoch, Balance, Signature, SignedTicket } from '.'
import * as utils from '../utils'
import { DEMO_ACCOUNTS } from '../config'

const [userA, userB] = DEMO_ACCOUNTS.map(str => stringToU8a(str))
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
    onChainSecret
  }
}

describe('test signedTicket construction', function() {
  it('should create new signedTicket using struct', async function() {
    const ticketData = await generateTicketData()

    const ticket = new Ticket(undefined, ticketData)
    const signature = await utils.sign(await ticket.hash, userA).then(res => {
      return new Signature({
        bytes: res.buffer,
        offset: res.byteOffset
      })
    })

    const signedTicket = new SignedTicket(undefined, {
      signature,
      ticket
    })

    assert(signedTicket.ticket.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(signedTicket.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicket.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicket.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicket.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(signedTicket.ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })

  it('should create new signedTicket using array', async function() {
    const ticketData = await generateTicketData()

    const ticket = new Ticket(undefined, ticketData)
    const signature = await utils.sign(await ticket.hash, userA).then(res => {
      return new Signature({
        bytes: res.buffer,
        offset: res.byteOffset
      })
    })

    const signedTicketA = new SignedTicket(undefined, {
      signature,
      ticket
    })
    const signedTicketB = new SignedTicket({
      bytes: signedTicketA.buffer,
      offset: signedTicketA.byteOffset
    })

    assert(signedTicketB.ticket.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(signedTicketB.ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(signedTicketB.ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(signedTicketB.ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(signedTicketB.ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(signedTicketB.ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })

  it('should verify signedTicket', async function() {
    const ticketData = await generateTicketData()
    const ticket = new Ticket(undefined, ticketData)

    const signature = await utils.sign(await ticket.hash, userA).then(res => {
      return new Signature({
        bytes: res.buffer,
        offset: res.byteOffset
      })
    })

    const signedTicket = new SignedTicket(undefined, {
      signature,
      ticket
    })

    const signer = new Hash(await signedTicket.signer)
    const userAPubKey = await utils.privKeyToPubKey(userA)

    assert(signer.eq(userAPubKey), 'signer incorrect')
  })
})
