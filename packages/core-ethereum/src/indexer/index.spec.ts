import type CoreConnector from '..'
import assert from 'assert'
import BN from 'bn.js'
import Web3 from 'web3'
import { Ganache } from '@hoprnet/hopr-testing'
import { migrate, fund, addresses, abis } from '@hoprnet/hopr-ethereum'
import { durations, u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'
import { stringToU8a } from '@hoprnet/hopr-utils'
import * as testconfigs from '../config.spec'
import * as configs from '../config'
import { time, isPartyA } from '../utils'
import { Account, getPrivKeyData, createAccountAndFund, createNode } from '../utils/testing.spec'
import { HoprToken } from '../tsc/web3/HoprToken'
import { HoprChannels } from '../tsc/web3/HoprChannels'
import { Public } from '../types'
import { publicKeyConvert } from 'secp256k1'
import { randomBytes } from 'crypto'

const HoprTokenAbi = abis.HoprToken
const HoprChannelsAbi = abis.HoprChannels
const CLOSURE_DURATION = durations.days(3)

// @TODO: remove legacy tests
// @TODO: add more tests
describe('test indexer', function () {
  this.timeout(durations.minutes(5))

  const ganache = new Ganache()
  let web3: Web3
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
    await fund(`--address ${addresses?.localhost?.HoprToken} --accounts-to-fund 4`)

    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, addresses?.localhost?.HoprToken)
    hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, addresses?.localhost?.HoprChannels)

    userA = await getPrivKeyData(stringToU8a(testconfigs.FUND_ACCOUNT_PRIVATE_KEY))
    // userA < userB
    userB = await createAccountAndFund(web3, hoprToken, userA, testconfigs.DEMO_ACCOUNTS[1])
    // userC < userA
    userC = await createAccountAndFund(web3, hoprToken, userA, testconfigs.DEMO_ACCOUNTS[2])
    //
    userD = await createAccountAndFund(web3, hoprToken, userA, testconfigs.DEMO_ACCOUNTS[3])
    connector = await createNode(userA.privKey, undefined, 8)

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

      //const uncompressedPubKeyA = publicKeyConvert(userA.pubKey, false).slice(1)
      const uncompressedPubKeyB = publicKeyConvert(userB.pubKey, false).slice(1)

      // await connector.hoprChannels.methods
      //   .init(
      //     u8aToHex(uncompressedPubKeyA.slice(0, 32)),
      //     u8aToHex(uncompressedPubKeyA.slice(32, 64)),
      //     u8aToHex(randomBytes(27))
      //   )
      //   .send({
      //     from: userA.address.toHex(),
      //   })

      await connector.hoprChannels.methods
        .init(
          u8aToHex(uncompressedPubKeyB.slice(0, 32)),
          u8aToHex(uncompressedPubKeyB.slice(32, 64)),
          u8aToHex(randomBytes(27))
        )
        .send({
          from: userB.address.toHex()
        })

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          web3.eth.abi.encodeParameters(['address', 'address'], [userA.address.toHex(), userB.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 200e3
        })
      await hoprChannels.methods.openChannel(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3
      })
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 0, 'check Channels.store')
    })

    it('should store channel & blockNumber correctly', async function () {
      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 1, 'check Channels.store')
    })

    it('should find all channels', async function () {
      let partyA: Public, partyB: Public
      if (isPartyA(await userA.pubKey.toAddress(), await userB.pubKey.toAddress())) {
        partyA = userA.pubKey
        partyB = userB.pubKey
      } else {
        partyA = userB.pubKey
        partyB = userA.pubKey
      }

      const blockNumber = await web3.eth.getBlockNumber()
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 1, 'check Channels.store')
      // @ts-ignore
      // const latestConfirmedBlockNumber = await connector.indexer.getLatestConfirmedBlockNumber()
      const [channel] = channels
      assert(u8aEquals(channel.partyA, partyA), 'check Channels.store')
      assert(u8aEquals(channel.partyB, partyB), 'check Channels.store')
      assert(channel.channelEntry.blockNumber.lt(new BN(blockNumber)), 'check Channels.store')
      // assert(latestConfirmedBlockNumber < blockNumber, 'check Channels.store')
    })

    it('should find channel using partyA', async function () {
      let partyA: Public, partyB: Public
      if (isPartyA(await userA.pubKey.toAddress(), await userB.pubKey.toAddress())) {
        partyA = userA.pubKey
        partyB = userB.pubKey
      } else {
        partyA = userB.pubKey
        partyB = userA.pubKey
      }

      const channels = await connector.indexer.getChannelEntries(partyA)
      assert.equal(channels.length, 1, 'check Channels.get')
      const [channel] = channels
      assert(u8aEquals(channel.partyA, partyA), 'check Channels.get')
      assert(u8aEquals(channel.partyB, partyB), 'check Channels.get')
    })
    it('should find channel using partyB', async function () {
      let partyA: Public, partyB: Public
      if (isPartyA(await userA.pubKey.toAddress(), await userB.pubKey.toAddress())) {
        partyA = userA.pubKey
        partyB = userB.pubKey
      } else {
        partyA = userB.pubKey
        partyB = userA.pubKey
      }

      const channels = await connector.indexer.getChannelEntries(partyB)
      const [channel] = channels
      assert.equal(channels.length, 1, 'check Channels.get')
      assert(u8aEquals(channel.partyA, partyA), 'check Channels.get')
      assert(u8aEquals(channel.partyB, partyB), 'check Channels.get')
    })

    it('should find channel using partyA & partyB', async function () {
      let partyA: Public, partyB: Public
      if (isPartyA(await userA.pubKey.toAddress(), await userB.pubKey.toAddress())) {
        partyA = userA.pubKey
        partyB = userB.pubKey
      } else {
        partyA = userB.pubKey
        partyB = userA.pubKey
      }

      const channel = await connector.indexer.getChannelEntry(partyA, partyB)
      assert(!!channel, 'check Channels.getChannelEntry')
    })
    it('should store another channel', async function () {
      this.timeout(durations.seconds(5))
      const uncompressedPubKeyC = publicKeyConvert(userC.pubKey, false).slice(1)

      await connector.hoprChannels.methods
        .init(
          u8aToHex(uncompressedPubKeyC.slice(0, 32)),
          u8aToHex(uncompressedPubKeyC.slice(32, 64)),
          u8aToHex(randomBytes(27))
        )
        .send({
          from: userC.address.toHex()
        })

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          web3.eth.abi.encodeParameters(['address', 'address'], [userA.address.toHex(), userC.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 200e3
        })

      await hoprChannels.methods.openChannel(userC.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3
      })
      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 2, 'check Channels.store')
      const channelsUsingPartyA = await connector.indexer.getChannelEntries(userA.pubKey)
      assert.equal(channelsUsingPartyA.length, 2, 'check Channels.get')
      const channelsUsingPartyB = await connector.indexer.getChannelEntries(userB.pubKey)
      assert.equal(channelsUsingPartyB.length, 1, 'check Channels.get')
    })

    it('should not delete channel before confirmations', async function () {
      await hoprChannels.methods.initiateChannelClosure(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3
      })
      await time.increase(web3, Math.floor(CLOSURE_DURATION / 1e3))
      await hoprChannels.methods.claimChannelClosure(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3
      })
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should "ZERO" channel', async function () {
      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 2, 'check Channels.store')

      let partyA: Public, partyB: Public
      if (isPartyA(await userA.pubKey.toAddress(), await userB.pubKey.toAddress())) {
        partyA = userA.pubKey
        partyB = userB.pubKey
      } else {
        partyA = userB.pubKey
        partyB = userA.pubKey
      }

      const channel = await connector.indexer.getChannelEntry(partyA, partyB)
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
      const uncompressedPubKeyD = publicKeyConvert(userD.pubKey, false).slice(1)

      await connector.hoprChannels.methods
        .init(
          u8aToHex(uncompressedPubKeyD.slice(0, 32)),
          u8aToHex(uncompressedPubKeyD.slice(32, 64)),
          u8aToHex(randomBytes(27))
        )
        .send({
          from: userD.address.toHex()
        })

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          web3.eth.abi.encodeParameters(['address', 'address'], [userA.address.toHex(), userD.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 200e3
        })
      await hoprChannels.methods.openChannel(userD.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3
      })
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should not index new channel', async function () {
      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should start indexer and index new channel', async function () {
      this.timeout(durations.seconds(5))
      await connector.indexer.start()
      assert(connector.indexer.status === 'started', 'could not start indexer')
      const channels = await connector.indexer.getChannelEntries()
      assert.equal(channels.length, 3, 'check Channels.store')
    })
  })
})
