import assert from 'assert'
import { stringToU8a, u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { getPrivKeyData, generateUser, generateNode } from '../utils/testing'
import { randomBytes } from 'crypto'
import BN from 'bn.js'
import pipe from 'it-pipe'
import Web3 from 'web3'
import { HoprToken } from '../tsc/web3/HoprToken'
import { Await } from '../tsc/utils'
import { AccountId, Channel as ChannelType, Balance, ChannelBalance, Hash, SignedChannel } from '../types'
import { ChannelStatus } from '../types/channel'
import CoreConnector from '..'
import Channel from '.'
import * as configs from '../config'

describe('test ticket generation and verification', function () {
  const web3 = new Web3(configs.DEFAULT_URI)
  const hoprToken: HoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
  const channels = new Map<string, ChannelType>()
  const preChannels = new Map<string, ChannelType>()
  let coreConnector: CoreConnector
  let counterpartysCoreConnector: CoreConnector
  let funder: Await<ReturnType<typeof getPrivKeyData>>

  beforeEach(async function () {
    channels.clear()
    preChannels.clear()

    funder = await getPrivKeyData(stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY))
    const userA = await generateUser(web3, funder, hoprToken)
    const userB = await generateUser(web3, funder, hoprToken)

    coreConnector = await generateNode(userA.privKey)
    counterpartysCoreConnector = await generateNode(userB.privKey)
  })

  it('should create a valid ticket', async function () {
    const channelType = new ChannelType(undefined, {
      balance: new ChannelBalance(undefined, {
        balance: new BN(123),
        balance_a: new BN(122),
      }),
      status: ChannelStatus.FUNDING,
    })

    const channelId = await coreConnector.utils.getId(
      new AccountId(coreConnector.self.onChainKeyPair.publicKey),
      new AccountId(counterpartysCoreConnector.self.onChainKeyPair.publicKey)
    )

    const signedChannel = await SignedChannel.create(counterpartysCoreConnector, undefined, { channel: channelType })

    preChannels.set(u8aToHex(channelId), channelType)

    const channel = await Channel.create(
      coreConnector,
      counterpartysCoreConnector.self.publicKey,
      async () => counterpartysCoreConnector.self.onChainKeyPair.publicKey,
      signedChannel.channel.balance,
      async () => {
        const result = await pipe(
          [(await SignedChannel.create(coreConnector, undefined, { channel: channelType })).subarray()],
          Channel.handleOpeningRequest(counterpartysCoreConnector),
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

    const preImage = randomBytes(32)
    const hash = await coreConnector.utils.hash(preImage)

    const ticket = await channel.ticket.create(channel, new Balance(1), new Hash(hash))
    assert(u8aEquals(await ticket.signer, coreConnector.self.publicKey), `Check that signer is recoverable`)

    const signedChannelCounterparty = await SignedChannel.create(coreConnector, undefined, { channel: channelType })
    assert(
      u8aEquals(await signedChannelCounterparty.signer, coreConnector.self.publicKey),
      `Check that signer is recoverable.`
    )

    counterpartysCoreConnector.db.put(
      Buffer.from(coreConnector.dbKeys.Channel(coreConnector.self.onChainKeyPair.publicKey)),
      Buffer.from(signedChannelCounterparty)
    )

    const dbChannels = (await counterpartysCoreConnector.channel.getAll(
      counterpartysCoreConnector,
      async (arg: any) => arg,
      async (arg: any) => Promise.all(arg)
    )) as Channel[]

    assert(
      u8aEquals(dbChannels[0].counterparty, coreConnector.self.onChainKeyPair.publicKey),
      `Channel record should make it into the database and its db-key should lead to the AccountId of the counterparty.`
    )

    const counterpartysChannel = await Channel.create(
      counterpartysCoreConnector,
      coreConnector.self.publicKey,
      () => Promise.resolve(coreConnector.self.onChainKeyPair.publicKey),
      signedChannel.channel.balance,
      () => Promise.resolve(signedChannelCounterparty)
    )

    assert(
      await coreConnector.channel.isOpen(coreConnector, counterpartysCoreConnector.self.onChainKeyPair.publicKey),
      `Checks that party A considers the channel open.`
    )
    assert(
      await counterpartysCoreConnector.channel.isOpen(
        counterpartysCoreConnector,
        coreConnector.self.onChainKeyPair.publicKey
      ),
      `Checks that party B considers the channel open.`
    )

    await channel.testAndSetNonce(new Uint8Array(1).fill(0xff)), `Should be able to set nonce.`

    assert.rejects(
      () => channel.testAndSetNonce(new Uint8Array(1).fill(0xff)),
      `Should reject when trying to set nonce twice.`
    )

    assert(await counterpartysChannel.ticket.verify(counterpartysChannel, ticket), `Ticket signature must be valid.`)
  })
})
