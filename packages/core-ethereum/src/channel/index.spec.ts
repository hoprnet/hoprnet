import { randomBytes } from 'crypto'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate } from '@hoprnet/hopr-ethereum'
import assert from 'assert'
import { stringToU8a, u8aEquals, u8aConcat, durations } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { getPrivKeyData, createAccountAndFund, createNode } from '../utils/testing.spec'
import { createChallenge, hash } from '../utils'
import BN from 'bn.js'
import pipe from 'it-pipe'
import Web3 from 'web3'
import { HoprToken } from '../tsc/web3/HoprToken'
import { Await } from '../tsc/utils'
import { Channel as ChannelType, ChannelStatus, ChannelBalance, ChannelState } from '../types/channel'
import { AcknowledgedTicket, Balance, SignedChannel, SignedTicket } from '../types'
import CoreConnector from '..'
import Channel from '.'
import * as testconfigs from '../config.spec'
import * as configs from '../config'

const DEFAULT_WIN_PROB = 1

describe('test Channel class', function () {
  const ganache = new Ganache()

  let web3: Web3
  let hoprToken: HoprToken
  let coreConnector: CoreConnector
  let counterpartysCoreConnector: CoreConnector
  let funder: Await<ReturnType<typeof getPrivKeyData>>

  async function getTicketData(winProb: number = DEFAULT_WIN_PROB) {
    const secretA = randomBytes(32)
    const secretB = randomBytes(32)
    const challenge = await createChallenge(secretA, secretB)

    return {
      secretA,
      secretB,
      response: await hash(u8aConcat(secretA, secretB)),
      winProb,
      challenge,
    }
  }

  before(async function () {
    this.timeout(60e3)

    await ganache.start()
    await migrate()

    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
  })

  after(async function () {
    await ganache.stop()
  })

  beforeEach(async function () {
    this.timeout(10e3)

    funder = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    const userA = await createAccountAndFund(web3, hoprToken, funder, testconfigs.DEMO_ACCOUNTS[1])
    const userB = await createAccountAndFund(web3, hoprToken, funder, testconfigs.DEMO_ACCOUNTS[2])

    coreConnector = await createNode(userA.privKey)
    await coreConnector.initOnchainValues()
    await coreConnector.start()

    counterpartysCoreConnector = await createNode(userB.privKey)
    await counterpartysCoreConnector.initOnchainValues()
    await counterpartysCoreConnector.start()
  })

  afterEach(async function () {
    await Promise.all([coreConnector.stop(), counterpartysCoreConnector.stop()])
  })

  it('should create a channel and submit tickets', async function () {
    this.timeout(durations.minutes(1))

    const channelBalance = new ChannelBalance(undefined, {
      balance: new BN(123),
      balance_a: new BN(122),
    })

    const channel = await coreConnector.channel.create(
      counterpartysCoreConnector.account.keys.onChain.pubKey,
      async () => counterpartysCoreConnector.account.keys.onChain.pubKey,
      channelBalance,
      async (channelBalance: ChannelBalance) => {
        const result = await pipe(
          [
            (
              await coreConnector.channel.createSignedChannel(undefined, {
                channel: new ChannelType(undefined, {
                  balance: channelBalance,
                  state: new ChannelState(undefined, { state: ChannelStatus.FUNDING }),
                }),
              })
            ).subarray(),
          ],
          counterpartysCoreConnector.channel.handleOpeningRequest.bind(counterpartysCoreConnector.channel),
          async (source: AsyncIterable<Uint8Array>) => {
            let result: Uint8Array
            for await (const msg of source) {
              if (result! == null) {
                result = msg.slice()
                return result
              } else {
                continue
              }
            }
          }
        )

        return new SignedChannel({
          bytes: result.buffer,
          offset: result.byteOffset,
        })
      }
    )

    const firstTicket = await getTicketData()

    const firstAckedTicket = new AcknowledgedTicket(coreConnector, undefined, {
      response: firstTicket.response,
    })
    const signedTicket = await channel.ticket.create(new Balance(1), firstTicket.challenge, firstTicket.winProb, {
      bytes: firstAckedTicket.buffer,
      offset: firstAckedTicket.signedTicketOffset,
    })

    assert(
      u8aEquals(await signedTicket.signer, coreConnector.account.keys.onChain.pubKey),
      `Check that signer is recoverable`
    )

    const dbChannels = (await counterpartysCoreConnector.channel.getAll(
      async (arg: any) => arg,
      async (arg: any) => Promise.all(arg)
    )) as Channel[]

    assert(
      u8aEquals(dbChannels[0].counterparty, coreConnector.account.keys.onChain.pubKey),
      `Channel record should make it into the database and its db-key should lead to the AccountId of the counterparty.`
    )

    const counterpartysChannel = await counterpartysCoreConnector.channel.create(
      coreConnector.account.keys.onChain.pubKey,
      () => Promise.resolve(coreConnector.account.keys.onChain.pubKey)
    )

    assert(
      await coreConnector.channel.isOpen(counterpartysCoreConnector.account.keys.onChain.pubKey),
      `Checks that party A considers the channel open.`
    )
    assert(
      await counterpartysCoreConnector.channel.isOpen(coreConnector.account.keys.onChain.pubKey),
      `Checks that party B considers the channel open.`
    )

    assert(
      await counterpartysCoreConnector.account.reservePreImageIfIsWinning(firstAckedTicket),
      `ticket must be winning`
    )

    await channel.testAndSetNonce(new Uint8Array(1).fill(0xff)), `Should be able to set nonce.`

    assert.rejects(
      () => channel.testAndSetNonce(new Uint8Array(1).fill(0xff)),
      `Should reject when trying to set nonce twice.`
    )

    assert(await counterpartysChannel.ticket.verify(signedTicket), `Ticket signature must be valid.`)

    const hashedSecretBefore = await counterpartysChannel.coreConnector.account.onChainSecret

    await counterpartysCoreConnector.channel.tickets.submit(firstAckedTicket)

    const hashedSecretAfter = await counterpartysChannel.coreConnector.account.onChainSecret

    assert(!hashedSecretBefore.eq(hashedSecretAfter), 'Ticket redemption must alter on-chain secret.')

    let errThrown = false
    try {
      await counterpartysCoreConnector.channel.tickets.submit(firstAckedTicket)
    } catch (err) {
      errThrown = true
    }

    assert(errThrown, 'Ticket must lose its validity after being submitted')

    const ATTEMPTS = 20

    let ticketData
    let nextSignedTicket: SignedTicket

    for (let i = 0; i < ATTEMPTS; i++) {
      ticketData = await getTicketData(0.5)
      let ackedTicket = new AcknowledgedTicket(counterpartysCoreConnector, undefined, {
        response: ticketData.response,
      })

      nextSignedTicket = await channel.ticket.create(new Balance(1), ticketData.challenge, ticketData.winProb, {
        bytes: ackedTicket.buffer,
        offset: ackedTicket.signedTicketOffset,
      })

      assert(await counterpartysChannel.ticket.verify(nextSignedTicket), `Ticket signature must be valid.`)

      if (await counterpartysCoreConnector.account.reservePreImageIfIsWinning(ackedTicket)) {
        console.log(`ticket submitted`)
        await counterpartysCoreConnector.channel.tickets.submit(ackedTicket)

        assert(ackedTicket.redeemed, 'ticket should get marked as redeemed')
      }
    }
  })
})
