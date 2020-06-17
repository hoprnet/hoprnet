import { Ganache, migrate } from '@hoprnet/hopr-ethereum'
import assert from 'assert'
import { u8aToHex, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { getPrivKeyData, createAccountAndFund, createNode } from '../utils/testing'
import { randomBytes } from 'crypto'
import BN from 'bn.js'
import pipe from 'it-pipe'
import Web3 from 'web3'
import { HoprToken } from '../tsc/web3/HoprToken'
import { Await } from '../tsc/utils'
import {
  AccountId,
  Balance,
  Channel as ChannelType,
  ChannelStatus,
  ChannelBalance,
  Hash,
  SignedTicket,
  SignedChannel,
} from '../types'
import CoreConnector from '..'
import * as testconfigs from '../config.spec'
import * as configs from '../config'

describe('test ticket generation and verification', function () {
  const ganache = new Ganache()
  let web3: Web3
  let hoprToken: HoprToken
  let coreConnector: CoreConnector
  let counterpartysCoreConnector: CoreConnector
  let funder: Await<ReturnType<typeof getPrivKeyData>>

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

  afterEach(async function () {
    await Promise.all([counterpartysCoreConnector.stop(), coreConnector.stop()])
  })

  beforeEach(async function () {
    funder = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    const userA = await createAccountAndFund(web3, hoprToken, funder)
    const userB = await createAccountAndFund(web3, hoprToken, funder)

    coreConnector = await createNode(userA.privKey)
    counterpartysCoreConnector = await createNode(userB.privKey)
    await coreConnector.db.clear()
    await counterpartysCoreConnector.db.clear()
  })

  it('should store ticket', async function () {
    const channelType = new ChannelType(undefined, {
      balance: new ChannelBalance(undefined, {
        balance: new BN(123),
        balance_a: new BN(122),
      }),
      status: ChannelStatus.FUNDING,
    })

    const channelId = new Hash(
      await coreConnector.utils.getId(
        new AccountId(coreConnector.account.keys.onChain.pubKey),
        new AccountId(counterpartysCoreConnector.account.keys.onChain.pubKey)
      )
    )

    const signedChannel = await counterpartysCoreConnector.channel.createSignedChannel(undefined, {
      channel: channelType,
    })

    const channel = await coreConnector.channel.create(
      counterpartysCoreConnector.account.keys.onChain.pubKey,
      async () => counterpartysCoreConnector.account.keys.onChain.pubKey,
      signedChannel.channel.balance,
      async () => {
        const result = await pipe(
          [(await coreConnector.channel.createSignedChannel(undefined, { channel: channelType })).subarray()],
          counterpartysCoreConnector.channel.handleOpeningRequest(),
          async (source: AsyncIterable<any>) => {
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

    const preImage = randomBytes(32)
    const hash = await coreConnector.utils.hash(preImage)

    const signedTicket = (await channel.ticket.create(new Balance(1), new Hash(hash))) as SignedTicket
    assert(
      u8aEquals(await signedTicket.signer, coreConnector.account.keys.onChain.pubKey),
      `Check that signer is recoverable`
    )

    const signedChannelCounterparty = await coreConnector.channel.createSignedChannel(undefined, {
      channel: channelType,
    })
    assert(
      u8aEquals(await signedChannelCounterparty.signer, coreConnector.account.keys.onChain.pubKey),
      `Check that signer is recoverable.`
    )

    await coreConnector.tickets.store(channelId, signedTicket)

    const storedSignedTicket = new Uint8Array(
      await coreConnector.db.get(Buffer.from(coreConnector.dbKeys.Ticket(channelId, signedTicket.ticket.challenge)))
    )

    assert(u8aEquals(signedTicket, storedSignedTicket), `Check that signedTicket is stored correctly`)
  })

  it('should store tickets, and retrieve them in a map', async function () {
    const channelType = new ChannelType(undefined, {
      balance: new ChannelBalance(undefined, {
        balance: new BN(123),
        balance_a: new BN(122),
      }),
      status: ChannelStatus.FUNDING,
    })

    const channelId = new Hash(
      await coreConnector.utils.getId(
        new AccountId(coreConnector.account.keys.onChain.pubKey),
        new AccountId(counterpartysCoreConnector.account.keys.onChain.pubKey)
      )
    )

    const signedChannel = await counterpartysCoreConnector.channel.createSignedChannel(undefined, {
      channel: channelType,
    })

    const channel = await coreConnector.channel.create(
      counterpartysCoreConnector.account.keys.onChain.pubKey,
      async () => counterpartysCoreConnector.account.keys.onChain.pubKey,
      signedChannel.channel.balance,
      async () => {
        const result = await pipe(
          [(await coreConnector.channel.createSignedChannel(undefined, { channel: channelType })).subarray()],
          counterpartysCoreConnector.channel.handleOpeningRequest(),
          async (source: AsyncIterable<any>) => {
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

    const hashA = await coreConnector.utils.hash(randomBytes(32))
    const hashB = await coreConnector.utils.hash(randomBytes(32))
    const signedTicketA = await channel.ticket.create(new Balance(1), new Hash(hashA))
    const signedTicketB = await channel.ticket.create(new Balance(1), new Hash(hashB))

    await Promise.all([
      coreConnector.tickets.store(channelId, signedTicketA),
      coreConnector.tickets.store(channelId, signedTicketB),
      coreConnector.tickets.store(new Hash(new Uint8Array(Hash.SIZE).fill(0x00)), signedTicketB),
    ])

    const storedSignedTickets = await coreConnector.tickets.get(channelId)
    assert(storedSignedTickets.size === 2, `Check getting signedTickets`)

    const storedSignedTicketA = storedSignedTickets.get(u8aToHex(signedTicketA.ticket.challenge))
    assert(u8aEquals(signedTicketA, storedSignedTicketA), `Check that signedTicketA is stored correctly`)

    const storedSignedTicketB = storedSignedTickets.get(u8aToHex(signedTicketB.ticket.challenge))
    assert(u8aEquals(signedTicketB, storedSignedTicketB), `Check that signedTicketB is stored correctly`)
  })
})
