import assert from 'assert'
import { randomBytes } from 'crypto'
import BN from 'bn.js'
import { stringToU8a, randomInteger } from '@hoprnet/hopr-utils'
import { AccountId, Ticket, Hash, TicketEpoch, Balance } from '.'
import * as utils from '../utils'
import { DEMO_ACCOUNTS } from '../config'

const [userA, userB] = DEMO_ACCOUNTS.map((str) => new AccountId(stringToU8a(str)))
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

describe('test ticket construction', function () {
  it('should create new ticket using struct', async function () {
    const ticketData = await generateTicketData()

    const ticket = new Ticket(undefined, ticketData)

    assert(ticket.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })

  it('should create new ticket using array', async function () {
    const ticketData = await generateTicketData()

    const ticketA = new Ticket(undefined, ticketData)
    const ticketB = new Ticket({
      bytes: ticketA.buffer,
      offset: ticketA.byteOffset,
    })

    assert(ticketB.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(ticketB.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(ticketB.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(ticketB.amount.eq(ticketData.amount), 'wrong amount')
    assert(ticketB.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(ticketB.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })

  it('should create new ticket out of continous memory', async function () {
    const ticketData = await generateTicketData()

    const offset = randomInteger(1, 31)
    const array = new Uint8Array(Ticket.SIZE + offset)

    const ticket = new Ticket(
      {
        bytes: array.buffer,
        offset: array.byteOffset + offset,
      },
      ticketData
    )

    assert(ticket.channelId.eq(ticketData.channelId), 'wrong channelId')
    assert(ticket.challenge.eq(ticketData.challenge), 'wrong challenge')
    assert(ticket.epoch.eq(ticketData.epoch), 'wrong epoch')
    assert(ticket.amount.eq(ticketData.amount), 'wrong amount')
    assert(ticket.winProb.eq(ticketData.winProb), 'wrong winProb')
    assert(ticket.onChainSecret.eq(ticketData.onChainSecret), 'wrong onChainSecret')
  })
})
