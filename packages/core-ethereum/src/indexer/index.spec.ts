import type CoreConnector from '..'
import assert from 'assert'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund, getAddresses } from '@hoprnet/hopr-ethereum'
import { durations, u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'
import { stringToU8a } from '@hoprnet/hopr-utils'
import * as testconfigs from '../config.spec'
import * as configs from '../config'
import { getId } from '../utils'
import { Account, getPrivKeyData, createAccountAndFund, createNode } from '../utils/testing'
import { publicKeyConvert } from 'secp256k1'
import { randomBytes } from 'crypto'
import { Hash } from '../types'
import { advanceBlockTo, increaseTime } from '../utils/testing'
import { ethers, providers } from 'ethers'
import { HoprToken__factory, HoprToken, HoprChannels__factory, HoprChannels } from '../contracts'

const abiCoder = new ethers.utils.AbiCoder()
const CLOSURE_DURATION = durations.days(3)

// @TODO: remove legacy tests
// @TODO: add more tests
describe('test indexer', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let provider: providers.JsonRpcProvider
  let hoprToken: HoprToken
  let hoprChannels: HoprChannels
  let connector: CoreConnector
  let userA: Account
  let userB: Account
  let userC: Account
  let userD: Account

  before(async function () {
    this.timeout(durations.minutes(1))

    await ganache.start()
    await migrate()
    await fund(`--address ${getAddresses()?.localhost?.HoprToken} --accounts-to-fund 4`)

    provider = new providers.JsonRpcProvider(configs.DEFAULT_URI)
    hoprToken = HoprToken__factory.connect(getAddresses().localhost?.HoprToken, provider)
    hoprChannels = HoprChannels__factory.connect(getAddresses().localhost?.HoprChannels, provider)

    userA = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    // userA < userB
    userB = await createAccountAndFund(provider, hoprToken, userA, testconfigs.DEMO_ACCOUNTS[1])
    // userC < userA
    userC = await createAccountAndFund(provider, hoprToken, userA, testconfigs.DEMO_ACCOUNTS[2])
    //
    userD = await createAccountAndFund(provider, hoprToken, userA, testconfigs.DEMO_ACCOUNTS[3])
    connector = await createNode(userA.privKey.serialize(), undefined, 8)

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

      const uncompressedPubKeyB = publicKeyConvert(userB.pubKey.serialize(), false).slice(1)

      await connector.hoprChannels.methods
        .initializeAccount(u8aToHex(uncompressedPubKeyB), u8aToHex(randomBytes(Hash.SIZE)))
        .send({
          from: userB.address.toHex()
        })

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          abiCoder.encode(['bool', 'address', 'address'], [true, userA.address.toHex(), userB.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 300e3
        })

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
        u8aEquals(channel.partyA.serialize(), userA.address.serialize()) ||
          u8aEquals(channel.partyA.serialize(), userB.address.serialize()),
        'check Channels.store'
      )
      assert(
        u8aEquals(channel.partyB.serialize(), userA.address.serialize()) ||
          u8aEquals(channel.partyB.serialize(), userB.address.serialize()),
        'check Channels.store'
      )
    })

    it('should find channel using partyA', async function () {
      const expectedChannelId = await getId(userA.address, userB.address)
      const channels = await connector.indexer.getChannelsOf(userA.address)
      assert.equal(channels.length, 1, 'check Channels.get')
      assert(expectedChannelId.eq(await channels[0].getId()), 'check Channels.get')
    })

    it('should find channel using partyB', async function () {
      const expectedChannelId = await getId(userA.address, userB.address)
      const channels = await connector.indexer.getChannelsOf(userB.address)
      assert.equal(channels.length, 1, 'check Channels.get')
      assert(expectedChannelId.eq(await channels[0].getId()), 'check Channels.get')
    })

    it('should find channel using partyA & partyB', async function () {
      const channel = await connector.indexer.getChannel(await getId(userA.address, userB.address))
      assert(!!channel, 'check Channels.getChannelEntry')
    })
    it('should store another channel', async function () {
      this.timeout(durations.seconds(5))
      const uncompressedPubKeyC = publicKeyConvert(userC.pubKey.serialize(), false).slice(1)

      await connector.hoprChannels.methods
        .initializeAccount(u8aToHex(uncompressedPubKeyC), u8aToHex(randomBytes(Hash.SIZE)))
        .send({
          from: userC.address.toHex()
        })

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          abiCoder.encode(['bool', 'address', 'address'], [true, userA.address.toHex(), userC.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 300e3
        })

      const currentBlockNumber = await provider.getBlockNumber()
      await advanceBlockTo(provider, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')
      const channelsUsingPartyA = await connector.indexer.getChannelsOf(userA.address)
      assert.equal(channelsUsingPartyA.length, 2, 'check Channels.get')
      const channelsUsingPartyB = await connector.indexer.getChannelsOf(userB.address)
      assert.equal(channelsUsingPartyB.length, 1, 'check Channels.get')
    })

    it('should not delete channel before confirmations', async function () {
      await hoprChannels.methods.initiateChannelClosure(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 300e3
      })
      await increaseTime(provider, Math.floor(CLOSURE_DURATION / 1e3))
      await hoprChannels.methods.finalizeChannelClosure(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 300e3
      })
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should "ZERO" channel', async function () {
      const currentBlockNumber = await provider.getBlockNumber()
      await advanceBlockTo(provider, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannels()
      assert.equal(channels.length, 2, 'check Channels.store')

      const channel = await connector.indexer.getChannel(await getId(userA.address, userB.address))
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
      const uncompressedPubKeyD = publicKeyConvert(userD.pubKey.serialize(), false).slice(1)

      await connector.hoprChannels.methods
        .initializeAccount(u8aToHex(uncompressedPubKeyD), u8aToHex(randomBytes(Hash.SIZE)))
        .send({
          from: userD.address.toHex()
        })

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          abiCoder.encode(['bool', 'address', 'address'], [true, userA.address.toHex(), userD.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 300e3
        })

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
