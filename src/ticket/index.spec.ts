import assert from 'assert'
import { getPrivKeyData, generateUser, generateNode } from '../utils/testing'
import { randomBytes } from 'crypto'
import BN from 'bn.js'
import pipe from 'it-pipe'
import Web3 from 'web3'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import { HoprToken } from '../tsc/web3/HoprToken'
import { Await } from '../tsc/utils'
import { Channel as ChannelType, Balance, ChannelBalance, Hash, SignedChannel } from '../types'
import { ChannelStatus } from '../types/channel'
import CoreConnector from '..'
import Channel from '../channel'
import Ticket from '.'
import * as u8a from '../core/u8a'
import * as configs from '../config'

describe('test ticket generation and verification', function() {
  const web3 = new Web3(configs.DEFAULT_URI)
  const hoprToken: HoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
  let coreConnector: CoreConnector
  let counterpartysCoreConnector: CoreConnector
  let funder: Await<ReturnType<typeof getPrivKeyData>>

  beforeEach(async function() {
    funder = await getPrivKeyData(u8a.stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY))
    const userA = await generateUser(web3, funder, hoprToken)
    const userB = await generateUser(web3, funder, hoprToken)

    coreConnector = await generateNode(userA.privKey)
    counterpartysCoreConnector = await generateNode(userB.privKey)

    await coreConnector.db.clear()
    await counterpartysCoreConnector.db.clear()
  })

  it('should store ticket', async function() {
    const channelType = new ChannelType(undefined, {
      balance: new ChannelBalance(undefined, {
        balance: new BN(123),
        balance_a: new BN(122)
      }),
      status: ChannelStatus.FUNDING
    })

    const channelId = new Hash(
      await coreConnector.utils.getId(
        coreConnector.self.onChainKeyPair.publicKey,
        counterpartysCoreConnector.self.onChainKeyPair.publicKey
      )
    )

    const signedChannel = await SignedChannel.create(counterpartysCoreConnector, undefined, { channel: channelType })

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
          offset: result.byteOffset
        })
      }
    )

    const preImage = randomBytes(32)
    const hash = await coreConnector.utils.hash(preImage)

    const signedTicket = await channel.ticket.create(channel, new Balance(1), new Hash(hash))
    assert(u8a.u8aEquals(await signedTicket.signer, coreConnector.self.publicKey), `Check that signer is recoverable`)

    const signedChannelCounterparty = await SignedChannel.create(coreConnector, undefined, { channel: channelType })
    assert(
      u8a.u8aEquals(await signedChannelCounterparty.signer, coreConnector.self.publicKey),
      `Check that signer is recoverable.`
    )

    await Ticket.store(coreConnector, channelId, signedTicket)

    const storedSignedTicket = new Uint8Array(
      await coreConnector.db.get(Buffer.from(coreConnector.dbKeys.Ticket(channelId, signedTicket.ticket.challenge)))
    )
    assert(u8a.u8aEquals(signedTicket, storedSignedTicket), `Check that signedTicket is stored correctly`)
  })

  it('should store tickets, and retrieve them in a map', async function() {
    const channelType = new ChannelType(undefined, {
      balance: new ChannelBalance(undefined, {
        balance: new BN(123),
        balance_a: new BN(122)
      }),
      status: ChannelStatus.FUNDING
    })

    const channelId = new Hash(
      await coreConnector.utils.getId(
        coreConnector.self.onChainKeyPair.publicKey,
        counterpartysCoreConnector.self.onChainKeyPair.publicKey
      )
    )

    const signedChannel = await SignedChannel.create(counterpartysCoreConnector, undefined, { channel: channelType })

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
          offset: result.byteOffset
        })
      }
    )

    const hashA = await coreConnector.utils.hash(randomBytes(32))
    const hashB = await coreConnector.utils.hash(randomBytes(32))
    const signedTicketA = await channel.ticket.create(channel, new Balance(1), new Hash(hashA))
    const signedTicketB = await channel.ticket.create(channel, new Balance(1), new Hash(hashB))

    await Promise.all([
      Ticket.store(coreConnector, channelId, signedTicketA),
      Ticket.store(coreConnector, channelId, signedTicketB),
      Ticket.store(coreConnector, new Uint8Array(Hash.SIZE).fill(0x00), signedTicketB)
    ])

    const storedSignedTickets = await Ticket.get(coreConnector, channelId)
    assert(storedSignedTickets.size === 2, `Check getting signedTickets`)

    const storedSignedTicketA = storedSignedTickets.get(u8a.u8aToHex(signedTicketA.ticket.challenge))
    assert(u8a.u8aEquals(signedTicketA, storedSignedTicketA), `Check that signedTicketA is stored correctly`)

    const storedSignedTicketB = storedSignedTickets.get(u8a.u8aToHex(signedTicketB.ticket.challenge))
    assert(u8a.u8aEquals(signedTicketB, storedSignedTicketB), `Check that signedTicketB is stored correctly`)
  })
})
