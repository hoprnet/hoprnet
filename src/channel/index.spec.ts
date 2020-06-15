import { Ganache, migrate } from '@hoprnet/hopr-ethereum'
import assert from 'assert'
import { stringToU8a, u8aToHex, u8aEquals, durations } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { getPrivKeyData, createAccountAndFund, createNode } from '../utils/testing'
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
  SignedChannel,
} from '../types'
import CoreConnector from '..'
import Channel from '.'
import * as configs from '../config'

describe('test Channel class', function () {
  const ganache = new Ganache()
  const channels = new Map<string, ChannelType>()
  const preChannels = new Map<string, ChannelType>()
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

  beforeEach(async function () {
    channels.clear()
    preChannels.clear()

    funder = await getPrivKeyData(stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY))
    const userA = await createAccountAndFund(web3, hoprToken, funder, configs.DEMO_ACCOUNTS[1])
    const userB = await createAccountAndFund(web3, hoprToken, funder, configs.DEMO_ACCOUNTS[2])

    coreConnector = await createNode(userA.privKey)
    counterpartysCoreConnector = await createNode(userB.privKey)
  })

  afterEach(async function () {
    await Promise.all([coreConnector.stop(), counterpartysCoreConnector.stop()])
  })

  it('should create a channel', async function () {
    this.timeout(durations.minutes(1))

    const channelType = new ChannelType(undefined, {
      balance: new ChannelBalance(undefined, {
        balance: new BN(123),
        balance_a: new BN(122),
      }),
      status: ChannelStatus.FUNDING,
    })

    const channelId = await coreConnector.utils.getId(
      new AccountId(coreConnector.account.keys.onChain.pubKey),
      new AccountId(counterpartysCoreConnector.account.keys.onChain.pubKey)
    )

    const signedChannel = await counterpartysCoreConnector.channel.createSignedChannel(undefined, {
      channel: channelType,
    })

    preChannels.set(u8aToHex(channelId), channelType)

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

    channels.set(u8aToHex(channelId), channelType)

    let secret: Uint8Array
    try {
      secret = await coreConnector.db.get(Buffer.from(coreConnector.dbKeys.OnChainSecret()))
    } catch (err) {
      throw err
    }

    const ticket = await channel.ticket.create(new Balance(1), new Hash())
    assert(
      u8aEquals(await ticket.signer, coreConnector.account.keys.onChain.pubKey),
      `Check that signer is recoverable`
    )

    const signedChannelCounterparty = await coreConnector.channel.createSignedChannel(undefined, {
      channel: channelType,
    })
    assert(
      u8aEquals(await signedChannelCounterparty.signer, coreConnector.account.keys.onChain.pubKey),
      `Check that signer is recoverable.`
    )

    counterpartysCoreConnector.db.put(
      Buffer.from(coreConnector.dbKeys.Channel(coreConnector.account.keys.onChain.pubKey)),
      Buffer.from(signedChannelCounterparty)
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
      () => Promise.resolve(coreConnector.account.keys.onChain.pubKey),
      signedChannel.channel.balance,
      () => Promise.resolve(signedChannelCounterparty)
    )

    assert(
      await coreConnector.channel.isOpen(counterpartysCoreConnector.account.keys.onChain.pubKey),
      `Checks that party A considers the channel open.`
    )
    assert(
      await counterpartysCoreConnector.channel.isOpen(coreConnector.account.keys.onChain.pubKey),
      `Checks that party B considers the channel open.`
    )

    await channel.testAndSetNonce(new Uint8Array(1).fill(0xff)), `Should be able to set nonce.`

    assert.rejects(
      () => channel.testAndSetNonce(new Uint8Array(1).fill(0xff)),
      `Should reject when trying to set nonce twice.`
    )

    assert(await counterpartysChannel.ticket.verify(ticket), `Ticket signature must be valid.`)
  })
})
