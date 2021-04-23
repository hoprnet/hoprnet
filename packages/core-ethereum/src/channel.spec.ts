/*
import { randomBytes } from 'crypto'
import assert from 'assert'
import { durations } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { Balance, Ticket, Address, Hash, UnacknowledgedTicket, PublicKey } from './types'
import Channel from './channel'

const DEFAULT_WIN_PROB = 1

describe('test Channel class', function () {
  let partyA: PublicKey
  let partyB: PublicKey

  async function getTicketData({
    counterparty,
    winProb = DEFAULT_WIN_PROB
  }: {
    counterparty: Address
    winProb?: number
  }) {
    const secretA = new Hash(randomBytes(32))
    const secretB = new Hash(randomBytes(32))
    const challenge = Hash.createChallenge(secretA.serialize(), secretB.serialize())

    return {
      secretA,
      secretB,
      winProb,
      counterparty,
      challenge
    }
  }

  beforeEach(async function () {
  })

  it('should create a channel and submit tickets', async function () {
    this.timeout(durations.minutes(1))

    const firstTicket = await getTicketData({
      counterparty: partyA.toAddress()
    })

    const partyAChannel = new Channel(mockConnectorA, partyA, partyB)
    const partyBChannel = new Channel(mockConnectorB, partyB, partyA)
    await partyAChannel.open(new Balance(new BN(123)))

    const signedTicket = await partyAChannel.createTicket(
      new Balance(new BN(1)),
      firstTicket.challenge,
      firstTicket.winProb
    )
    const unacknowledgedTicket = new UnacknowledgedTicket(signedTicket, firstTicket.secretA)
    const firstAckedTicket = await partyBChannel.acknowledge(unacknowledgedTicket, firstTicket.secretB)

    assert(partyA.eq(signedTicket.getSigner()), `Check that signer is recoverable`)

    const partyAIndexerChannels = await partyAConnector.indexer.getChannels()
    assert(
      partyAIndexerChannels[0].partyA.eq(partyA.toAddress()) && partyAIndexerChannels[0].partyB.eq(partyB.toAddress()),
      `Channel record should make it into the database and its db-key should lead to the Address of the counterparty.`
    )
    assert((await partyAChannel.getState()).status === 'OPEN', `Checks that party A considers the channel open.`)
    assert((await partyBChannel.getState()).status == 'OPEN', `Checks that party A considers the channel open.`)
    assert(firstAckedTicket, `ticket must be winning`)

    const hashedSecretBefore = await partyBConnector.account.getOnChainSecret()

    try {
      const result = await partyBChannel.submitTicket(firstAckedTicket)
      if (result.status === 'ERROR') {
        throw result.error
      } else if (result.status === 'FAILURE') {
        throw Error(result.message)
      }
    } catch (error) {
      throw error
    }

    const hashedSecretAfter = await partyBConnector.account.getOnChainSecret()
    assert(!hashedSecretBefore.eq(hashedSecretAfter), 'Ticket redemption must alter on-chain secret.')

    let errThrown = false
    try {
      const result = await partyBChannel.submitTicket(firstAckedTicket)
      if (result.status === 'ERROR' || result.status === 'FAILURE') {
        errThrown = true
      }
    } catch (err) {
      errThrown = true
    }

    assert(errThrown, 'Ticket must lose its validity after being submitted')

    const ATTEMPTS = 20

    let ticketData
    let nextSignedTicket: Ticket

    for (let i = 0; i < ATTEMPTS; i++) {
      ticketData = await getTicketData({
        counterparty: partyA.toAddress(),
        winProb: 1
      })
      nextSignedTicket = await partyAChannel.createTicket(
        new Balance(new BN(1)),
        ticketData.challenge,
        ticketData.winProb
      )
      const nextUnacknowledgedTicket = new UnacknowledgedTicket(nextSignedTicket, ticketData.secretA)
      const ackedTicket = await partyBChannel.acknowledge(nextUnacknowledgedTicket, ticketData.secretB)

      if (ackedTicket !== null) {
        const result = await partyBChannel.submitTicket(ackedTicket)
        assert(result.status === 'SUCCESS', 'ticket redeemption was not a success')
      }
    }
  })
})
*/
