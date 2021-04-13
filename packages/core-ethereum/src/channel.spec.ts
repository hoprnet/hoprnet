import { randomBytes } from 'crypto'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import assert from 'assert'
import { durations } from '@hoprnet/hopr-utils'
import { getAddresses } from '@hoprnet/hopr-ethereum'
import { createNode, fundAccount, advanceBlock } from './utils/testing'
import BN from 'bn.js'
import { Balance, Ticket, Address, Hash, UnacknowledgedTicket, PublicKey } from './types'
import CoreConnector from '.'
import Channel from './channel'
import * as testconfigs from './config.spec'
import * as configs from './config'
import { providers, ethers } from 'ethers'
import { HoprToken__factory, HoprToken } from './contracts'

const { arrayify } = ethers.utils
const DEFAULT_WIN_PROB = 1

// @TODO: rewrite legacy tests
describe('test Channel class', function () {
  const ganache = new Ganache()

  let provider: providers.WebSocketProvider
  let hoprToken: HoprToken
  let partyA: PublicKey
  let partyB: PublicKey
  let partyAConnector: CoreConnector
  let partyBConnector: CoreConnector
  let funderWallet: ethers.Wallet

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

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()

    provider = new providers.WebSocketProvider(configs.DEFAULT_URI)
    hoprToken = HoprToken__factory.connect(getAddresses().localhost?.HoprToken, provider)
  })

  after(async function () {
    await ganache.stop()
  })

  beforeEach(async function () {
    this.timeout(durations.seconds(10))

    funderWallet = new ethers.Wallet(testconfigs.FUND_ACCOUNT_PRIVATE_KEY).connect(provider)
    partyA = PublicKey.fromPrivKey(arrayify(testconfigs.DEMO_ACCOUNTS[1]))
    await fundAccount(funderWallet, hoprToken, partyA.toAddress().toHex())
    partyB = PublicKey.fromPrivKey(arrayify(testconfigs.DEMO_ACCOUNTS[2]))
    await fundAccount(funderWallet, hoprToken, partyB.toAddress().toHex())

    partyAConnector = await createNode(arrayify(testconfigs.DEMO_ACCOUNTS[1]))
    await partyAConnector.initOnchainValues()
    await partyAConnector.start()

    partyBConnector = await createNode(arrayify(testconfigs.DEMO_ACCOUNTS[2]))
    await partyBConnector.initOnchainValues()
    await partyBConnector.start()
  })

  afterEach(async function () {
    await Promise.all([partyAConnector.stop(), partyBConnector.stop()])
  })

  it('should create a channel and submit tickets', async function () {
    this.timeout(durations.minutes(1))

    const firstTicket = await getTicketData({
      counterparty: partyA.toAddress()
    })

    const partyAChannel = new Channel(partyAConnector, partyA, partyB)
    await partyAChannel.open(new Balance(new BN(123)))
    await advanceBlock(provider)
    await advanceBlock(provider)

    const signedTicket = await partyAChannel.createTicket(
      new Balance(new BN(1)),
      firstTicket.challenge,
      firstTicket.winProb
    )
    const unacknowledgedTicket = new UnacknowledgedTicket(signedTicket, firstTicket.secretA)
    const firstAckedTicket = await partyBConnector.account.acknowledge(unacknowledgedTicket, firstTicket.secretB)

    assert(partyA.eq(signedTicket.getSigner()), `Check that signer is recoverable`)

    const partyAIndexerChannels = await partyAConnector.indexer.getChannels()
    assert(
      partyAIndexerChannels[0].partyA.eq(partyA.toAddress()) && partyAIndexerChannels[0].partyB.eq(partyB.toAddress()),
      `Channel record should make it into the database and its db-key should lead to the Address of the counterparty.`
    )

    const partyBChannel = new Channel(partyBConnector, partyB, partyA)
    assert((await partyAChannel.getState()).getStatus() === 'OPEN', `Checks that party A considers the channel open.`)
    assert((await partyBChannel.getState()).getStatus() === 'OPEN', `Checks that party A considers the channel open.`)
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
      const ackedTicket = await partyBConnector.account.acknowledge(nextUnacknowledgedTicket, ticketData.secretB)

      if (ackedTicket !== null) {
        const result = await partyBChannel.submitTicket(ackedTicket)
        assert(result.status === 'SUCCESS', 'ticket redeemption was not a success')
      }
    }
  })
})
