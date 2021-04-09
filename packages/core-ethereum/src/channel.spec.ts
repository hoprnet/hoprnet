import { randomBytes } from 'crypto'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import assert from 'assert'
import { stringToU8a, u8aConcat, durations, PromiseValue } from '@hoprnet/hopr-utils'
import { getAddresses } from '@hoprnet/hopr-ethereum'
import { getPrivKeyData, createAccountAndFund, createNode, Account } from './utils/testing'
import { createChallenge, isPartyA, hash } from './utils'
import BN from 'bn.js'
import { Balance, Ticket, Address } from './types'
import CoreConnector from '.'
import Channel from './channel'
import * as testconfigs from './config.spec'
import * as configs from './config'
import { providers } from 'ethers'
import { HoprToken__factory, HoprToken } from './contracts'

const DEFAULT_WIN_PROB = 1

// @TODO: rewrite legacy tests
describe('test Channel class', function () {
  const ganache = new Ganache()

  let provider: providers.JsonRpcProvider
  let hoprToken: HoprToken
  let partyA: Account
  let partyB: Account
  let partyAConnector: CoreConnector
  let partyBConnector: CoreConnector
  let funder: PromiseValue<ReturnType<typeof getPrivKeyData>>

  async function getTicketData({
    counterparty,
    winProb = DEFAULT_WIN_PROB
  }: {
    counterparty: Address
    winProb?: number
  }) {
    const secretA = randomBytes(32)
    const secretB = randomBytes(32)
    const challenge = await createChallenge(secretA, secretB)

    return {
      secretA,
      secretB,
      response: await hash(u8aConcat(secretA, secretB)),
      winProb,
      counterparty,
      challenge
    }
  }

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()

    provider = new providers.JsonRpcProvider(configs.DEFAULT_URI)
    hoprToken = HoprToken__factory.connect(getAddresses().localhost?.HoprToken, provider)
  })

  after(async function () {
    await ganache.stop()
  })

  beforeEach(async function () {
    this.timeout(durations.seconds(10))

    funder = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    const userA = await createAccountAndFund(provider, hoprToken, funder, testconfigs.DEMO_ACCOUNTS[1])
    const userB = await createAccountAndFund(provider, hoprToken, funder, testconfigs.DEMO_ACCOUNTS[2])
    ;[partyA, partyB] = isPartyA(userA.address, userB.address) ? [userA, userB] : [userB, userA]

    partyAConnector = await createNode(partyA.privKey.serialize())
    await partyAConnector.initOnchainValues()
    await partyAConnector.start()

    partyBConnector = await createNode(partyB.privKey.serialize())
    await partyBConnector.initOnchainValues()
    await partyBConnector.start()
  })

  afterEach(async function () {
    await Promise.all([partyAConnector.stop(), partyBConnector.stop()])
  })

  it('should create a channel and submit tickets', async function () {
    this.timeout(durations.minutes(1))

    const firstTicket = await getTicketData({
      counterparty: partyA.address
    })

    const partyAChannel = new Channel(partyAConnector, partyA.pubKey, partyB.pubKey)
    await partyAChannel.open(new Balance(new BN(123)))

    const signedTicket = await partyAChannel.createTicket(
      new Balance(new BN(1)),
      firstTicket.challenge,
      firstTicket.winProb
    )

    const firstAckedTicket = await partyBConnector.account.acknowledge(signedTicket, firstTicket.response)

    assert(partyA.pubKey.eq(signedTicket.getSigner()), `Check that signer is recoverable`)

    const partyAIndexerChannels = await partyAConnector.indexer.getChannels()
    assert(
      partyAIndexerChannels[0].partyA.eq(partyA.address) && partyAIndexerChannels[0].partyB.eq(partyB.address),
      `Channel record should make it into the database and its db-key should lead to the Address of the counterparty.`
    )

    const partyBChannel = new Channel(partyBConnector, partyB.pubKey, partyA.pubKey)
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
        counterparty: partyA.address,
        winProb: 1
      })
      nextSignedTicket = await partyAChannel.createTicket(
        new Balance(new BN(1)),
        ticketData.challenge,
        ticketData.winProb
      )
      const ackedTicket = await partyBConnector.account.acknowledge(nextSignedTicket, ticketData.response)

      if (ackedTicket !== null) {
        const result = await partyBChannel.submitTicket(ackedTicket)
        assert(result.status === 'SUCCESS', 'ticket redeemption was not a success')
      }
    }
  })
})
