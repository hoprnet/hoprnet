import type CoreConnector from '..'
import assert from 'assert'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund, getAddresses } from '@hoprnet/hopr-ethereum'
import { durations, u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'
import * as testconfigs from '../config.spec'
import * as configs from '../config'
import { PublicKey } from '../types'
import { advanceBlockTo, increaseTime } from '../utils/testing'
import { ethers, providers } from 'ethers'
import { HoprToken__factory, HoprToken, HoprChannels__factory, HoprChannels } from '../contracts'
import { createNode, fundAccount } from '../utils/testing'
import { publicKeyConvert } from 'secp256k1'
import { randomBytes } from 'crypto'
import { Channel } from '..'
import { Hash } from '../types'

const { arrayify, AbiCoder } = ethers.utils
const abiCoder = new AbiCoder()
const CLOSURE_DURATION = durations.days(3)

// @TODO: remove legacy tests
// @TODO: add more tests
describe('test indexer', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let provider: providers.WebSocketProvider
  let hoprToken: HoprToken
  let hoprChannels: HoprChannels
  let connector: CoreConnector
  let userAWallet: ethers.Wallet
  let userA: PublicKey
  let userBWallet: ethers.Wallet
  let userB: PublicKey
  let userCWallet: ethers.Wallet
  let userC: PublicKey
  let userDWallet: ethers.Wallet
  let userD: PublicKey

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 4`)

    provider = new providers.WebSocketProvider(configs.DEFAULT_URI)
    hoprToken = HoprToken__factory.connect(getAddresses().localhost?.HoprToken, provider)
    hoprChannels = HoprChannels__factory.connect(getAddresses().localhost?.HoprChannels, provider)

    userAWallet = new ethers.Wallet(testconfigs.FUND_ACCOUNT_PRIVATE_KEY).connect(provider)
    userA = PublicKey.fromPrivKey(arrayify(userAWallet.privateKey))
    // userA < userB
    userBWallet = new ethers.Wallet(testconfigs.DEMO_ACCOUNTS[1]).connect(provider)
    userB = PublicKey.fromPrivKey(arrayify(userBWallet.privateKey))
    await fundAccount(userAWallet, hoprToken, userBWallet.address)
    // userC < userA
    userCWallet = new ethers.Wallet(testconfigs.DEMO_ACCOUNTS[2]).connect(provider)
    userC = PublicKey.fromPrivKey(arrayify(userCWallet.privateKey))
    await fundAccount(userAWallet, hoprToken, userCWallet.address)
    userDWallet = new ethers.Wallet(testconfigs.DEMO_ACCOUNTS[3]).connect(provider)
    userD = PublicKey.fromPrivKey(arrayify(userDWallet.privateKey))
    await fundAccount(userAWallet, hoprToken, userDWallet.address)
    connector = await createNode(arrayify(userAWallet.privateKey), undefined, 8)

    await connector.start()
    await connector.initOnchainValues()
    await connector.db.clear()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  context('intergration tests', function () {
    it('should not store channel before confirmations', async function () {
      this.timeout(durations.seconds(5))

      const uncompressedPubKeyB = publicKeyConvert(userB.serialize(), false).slice(1)

      await connector.hoprChannels
        .connect(userBWallet)
        .initializeAccount(u8aToHex(uncompressedPubKeyB), u8aToHex(randomBytes(Hash.SIZE)))

      await hoprToken
        .connect(userAWallet)
        .send(
          hoprChannels.address,
          1,
          abiCoder.encode(['bool', 'address', 'address'], [true, userA.toAddress().toHex(), userB.toAddress().toHex()])
        )

      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 0, 'check Channels.store')
    })

    it('should store channel & blockNumber correctly', async function () {
      const currentBlockNumber = await provider.getBlockNumber()
      await advanceBlockTo(provider, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 1, 'check Channels.store')
    })

    it('should find all channels', async function () {
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 1, 'check Channels.store')

      const [channel] = channels
      assert(
        u8aEquals(channel.partyA.serialize(), userA.toAddress().serialize()) ||
          u8aEquals(channel.partyA.serialize(), userB.toAddress().serialize()),
        'check Channels.store'
      )
      assert(
        u8aEquals(channel.partyB.serialize(), userA.toAddress().serialize()) ||
          u8aEquals(channel.partyB.serialize(), userB.toAddress().serialize()),
        'check Channels.store'
      )
    })

    it('should find channel using partyA', async function () {
      const expectedChannelId = Channel.generateId(userA.toAddress(), userB.toAddress())
      const channels = await connector.indexer.getChannelsOf(userA.toAddress())
      assert.equal(channels.length, 1, 'check Channels.get')
      assert(expectedChannelId.eq(await channels[0].getId()), 'check Channels.get')
    })

    it('should find channel using partyB', async function () {
      const expectedChannelId = Channel.generateId(userA.toAddress(), userB.toAddress())
      const channels = await connector.indexer.getChannelsOf(userB.toAddress())
      assert.equal(channels.length, 1, 'check Channels.get')
      assert(expectedChannelId.eq(await channels[0].getId()), 'check Channels.get')
    })

    it('should find channel using partyA & partyB', async function () {
      const channel = await connector.indexer.getChannel(Channel.generateId(userA.toAddress(), userB.toAddress()))
      assert(!!channel, 'check Channels.getChannelEntry')
    })
    it('should store another channel', async function () {
      this.timeout(durations.seconds(5))
      const uncompressedPubKeyC = publicKeyConvert(userC.serialize(), false).slice(1)

      await connector.hoprChannels
        .connect(userCWallet)
        .initializeAccount(u8aToHex(uncompressedPubKeyC), u8aToHex(randomBytes(Hash.SIZE)))

      await hoprToken
        .connect(userAWallet)
        .send(
          hoprChannels.address,
          1,
          abiCoder.encode(['bool', 'address', 'address'], [true, userA.toAddress().toHex(), userC.toAddress().toHex()]),
          {
            gasLimit: 300e3
          }
        )

      const currentBlockNumber = await provider.getBlockNumber()
      await advanceBlockTo(provider, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')
      const channelsUsingPartyA = await connector.indexer.getChannelsOf(userA.toAddress())
      assert.equal(channelsUsingPartyA.length, 2, 'check Channels.get')
      const channelsUsingPartyB = await connector.indexer.getChannelsOf(userB.toAddress())
      assert.equal(channelsUsingPartyB.length, 1, 'check Channels.get')
    })

    it('should not delete channel before confirmations', async function () {
      await hoprChannels.connect(userAWallet).initiateChannelClosure(userB.toAddress().toHex(), {
        gasLimit: 300e3
      })
      await increaseTime(provider, Math.floor(CLOSURE_DURATION / 1e3))
      await hoprChannels.connect(userAWallet).finalizeChannelClosure(userB.toAddress().toHex(), {
        gasLimit: 300e3
      })
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should "ZERO" channel', async function () {
      const currentBlockNumber = await provider.getBlockNumber()
      await advanceBlockTo(provider, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')

      const channel = await connector.indexer.getChannel(Channel.generateId(userA.toAddress(), userB.toAddress()))
      assert(!!channel, 'check Channels.getChannelEntry')
      assert(channel.deposit.isZero())
      assert(channel.partyABalance.isZero())
      assert(channel.closureTime.isZero())
      assert(!channel.closureByPartyA)
    })

    it('should stop indexer and open new channel', async function () {
      this.timeout(durations.seconds(5))
      await connector.indexer.stop()
      assert(connector.indexer.status === 'stopped', 'could not stop indexer')
      const uncompressedPubKeyD = publicKeyConvert(userD.serialize(), false).slice(1)

      await connector.hoprChannels
        .connect(userDWallet)
        .initializeAccount(u8aToHex(uncompressedPubKeyD), u8aToHex(randomBytes(Hash.SIZE)))

      await hoprToken
        .connect(userAWallet)
        .send(
          hoprChannels.address,
          1,
          abiCoder.encode(['bool', 'address', 'address'], [true, userA.toAddress().toHex(), userD.toAddress().toHex()]),
          {
            gasLimit: 300e3
          }
        )

      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should not index new channel', async function () {
      const currentBlockNumber = await provider.getBlockNumber()
      await advanceBlockTo(provider, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should start indexer and index new channel', async function () {
      this.timeout(durations.seconds(5))
      await connector.indexer.start()
      assert(connector.indexer.status === 'started', 'could not start indexer')
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 3, 'check Channels.store')
    })
  })
})
