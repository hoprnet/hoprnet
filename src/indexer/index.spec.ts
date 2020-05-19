import assert from 'assert'
import BN from 'bn.js'
import Web3 from 'web3'
import { Ganache, migrate, fund } from '@hoprnet/hopr-ethereum'
import { durations } from '@hoprnet/hopr-utils'
import { stringToU8a } from '@hoprnet/hopr-utils'
import HoprTokenAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json'
import HoprChannelsAbi from '@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json'
import * as configs from '../config'
import { getParties, time, wait } from '../utils'
import { Account, getPrivKeyData, createAccountAndFund, createNode } from '../utils/testing'
import { HoprToken } from '../tsc/web3/HoprToken'
import { HoprChannels } from '../tsc/web3/HoprChannels'
import type CoreConnector from '..'

const CLOSURE_DURATION = durations.days(3)

describe('test indexer', function () {
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
    this.timeout(60e3)

    await ganache.start()
    await migrate()
    await fund()

    web3 = new Web3(configs.DEFAULT_URI)
    hoprToken = new web3.eth.Contract(HoprTokenAbi as any, configs.TOKEN_ADDRESSES.private)
    hoprChannels = new web3.eth.Contract(HoprChannelsAbi as any, configs.CHANNELS_ADDRESSES.private)

    userA = await getPrivKeyData(stringToU8a(configs.FUND_ACCOUNT_PRIVATE_KEY))
    // userA < userB
    userB = await createAccountAndFund(web3, hoprToken, userA, configs.DEMO_ACCOUNTS[1])
    // userC < userA
    userC = await createAccountAndFund(web3, hoprToken, userA, configs.DEMO_ACCOUNTS[2])
    //
    userD = await createAccountAndFund(web3, hoprToken, userA, configs.DEMO_ACCOUNTS[3])
    connector = await createNode(userA.privKey)

    await connector.start()
    await connector.db.clear()
  })

  after(async function () {
    await connector.stop()
    await ganache.stop()
  })

  context('intergration tests', function () {
    it('should not store channel before confirmations', async function () {
      this.timeout(5e3)

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          web3.eth.abi.encodeParameters(['address', 'address'], [userA.address.toHex(), userB.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 200e3,
        })

      await hoprChannels.methods.openChannel(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3,
      })

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 0, 'check Channels.store')
    })

    it('should store channel & blockNumber correctly', async function () {
      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 1, 'check Channels.store')
    })

    it('should find all channels', async function () {
      const [partyA, partyB] = getParties(userA.address, userB.address)

      const blockNumber = await web3.eth.getBlockNumber()
      const channels = await connector.indexer.get()
      assert.equal(channels.length, 1, 'check Channels.store')

      // @ts-ignore
      const latestConfirmedBlockNumber = await connector.indexer.getLatestConfirmedBlockNumber()

      const [channel] = channels
      assert(channel.partyA.eq(partyA), 'check Channels.store')
      assert(channel.partyB.eq(partyB), 'check Channels.store')
      assert(channel.channelEntry.blockNumber.lt(new BN(blockNumber)), 'check Channels.store')
      assert(latestConfirmedBlockNumber < blockNumber, 'check Channels.store')
    })

    it('should find channel using partyA', async function () {
      const [partyA, partyB] = getParties(userA.address, userB.address)

      const channels = await connector.indexer.get({
        partyA,
      })
      const [channel] = channels

      assert.equal(channels.length, 1, 'check Channels.get')
      assert(channel.partyA.eq(partyA), 'check Channels.get')
      assert(channel.partyB.eq(partyB), 'check Channels.get')
    })

    it('should find channel using partyB', async function () {
      const [partyA, partyB] = getParties(userA.address, userB.address)

      const channels = await connector.indexer.get({
        partyB,
      })
      const [channel] = channels

      assert.equal(channels.length, 1, 'check Channels.get')
      assert(channel.partyA.eq(partyA), 'check Channels.get')
      assert(channel.partyB.eq(partyB), 'check Channels.get')
    })

    it('should find channel using partyA & partyB', async function () {
      const [partyA, partyB] = getParties(userA.address, userB.address)

      const channels = await connector.indexer.get({
        partyA,
        partyB,
      })
      const [channel] = channels

      assert.equal(channels.length, 1, 'check Channels.get')
      assert(channel.partyA.eq(partyA), 'check Channels.get')
      assert(channel.partyB.eq(partyB), 'check Channels.get')
    })

    it('should store another channel', async function () {
      this.timeout(5e3)

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          web3.eth.abi.encodeParameters(['address', 'address'], [userA.address.toHex(), userC.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 200e3,
        })

      await hoprChannels.methods.openChannel(userC.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3,
      })

      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 2, 'check Channels.store')

      const channelsUsingPartyA = await connector.indexer.get({
        partyA: userA.address,
      })
      assert.equal(channelsUsingPartyA.length, 2, 'check Channels.get')

      const channelsUsingPartyB = await connector.indexer.get({
        partyB: userB.address,
      })
      assert.equal(channelsUsingPartyB.length, 1, 'check Channels.get')
    })

    it('should not delete channel before confirmations', async function () {
      await hoprChannels.methods.initiateChannelClosure(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3,
      })

      await time.increase(web3, Math.floor(CLOSURE_DURATION / 1e3))

      await hoprChannels.methods.claimChannelClosure(userB.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3,
      })

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 2, 'check Channels.store')
    })

    it('should delete channel', async function () {
      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 1, 'check Channels.store')
    })

    it('should stop indexer and open new channel', async function () {
      this.timeout(5e3)
      assert(await connector.indexer.stop(), 'could not stop indexer')

      await hoprToken.methods
        .send(
          hoprChannels.options.address,
          1,
          web3.eth.abi.encodeParameters(['address', 'address'], [userA.address.toHex(), userD.address.toHex()])
        )
        .send({
          from: userA.address.toHex(),
          gas: 200e3,
        })

      await hoprChannels.methods.openChannel(userD.address.toHex()).send({
        from: userA.address.toHex(),
        gas: 200e3,
      })

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 1, 'check Channels.store')
    })

    it('should not index new channel', async function () {
      const currentBlockNumber = await web3.eth.getBlockNumber()
      await time.advanceBlockTo(web3, currentBlockNumber + configs.MAX_CONFIRMATIONS)

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 1, 'check Channels.store')
    })

    it('should start indexer', async function () {
      this.timeout(5e3)

      assert(await connector.indexer.start(), 'could not start indexer')
      await wait(1e3)

      const channels = await connector.indexer.get()
      assert.equal(channels.length, 2, 'check Channels.store')
    })
  })

  context('unit tests', function () {
    beforeEach(async function () {
      await connector.db.clear()
    })

    it('should not store older channel according to blockNumber', async function () {
      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 2,
        transactionIndex: 0,
        logIndex: 0,
      } as any)

      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 0,
        logIndex: 0,
      } as any)

      const allChannels = await connector.indexer.get()
      assert.equal(allChannels.length, 1, 'check Channels.onOpenedChannel blockNumber')
      assert.equal(allChannels[0].channelEntry.blockNumber.toNumber(), 2, 'check Channels.onOpenedChannel blockNumber')
    })

    it('should not delete latest channel according to blockNumber', async function () {
      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 2,
        transactionIndex: 0,
        logIndex: 0,
      } as any)

      // @ts-ignore
      await connector.indexer.onClosedChannel({
        returnValues: {
          closer: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 0,
        logIndex: 0,
      } as any)

      const allChannels = await connector.indexer.get()
      assert.equal(allChannels.length, 1, 'check Channels.onClosedChannel blockNumber')
      assert.equal(allChannels[0].channelEntry.blockNumber.toNumber(), 2, 'check Channels.onClosedChannel blockNumber')
    })

    it('should not store older channel according to transactionIndex', async function () {
      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 2,
        logIndex: 0,
      } as any)

      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 1,
        logIndex: 0,
      } as any)

      const allChannels = await connector.indexer.get()
      assert.equal(allChannels.length, 1, 'check Channels.onOpenedChannel transactionIndex')
      assert.equal(
        allChannels[0].channelEntry.transactionIndex.toNumber(),
        2,
        'check Channels.onOpenedChannel transactionIndex'
      )
    })

    it('should not delete latest channel according to transactionIndex', async function () {
      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 2,
        logIndex: 0,
      } as any)

      // @ts-ignore
      await connector.indexer.onClosedChannel({
        returnValues: {
          closer: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 1,
        logIndex: 0,
      } as any)

      const allChannels = await connector.indexer.get()
      assert.equal(allChannels.length, 1, 'check Channels.onOpenedChannel transactionIndex')
      assert.equal(
        allChannels[0].channelEntry.transactionIndex.toNumber(),
        2,
        'check Channels.onOpenedChannel transactionIndex'
      )
    })

    it('should not store older channel according to logIndex', async function () {
      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 0,
        logIndex: 2,
      } as any)

      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 0,
        logIndex: 1,
      } as any)

      const allChannels = await connector.indexer.get()
      assert.equal(allChannels.length, 1, 'check Channels.onOpenedChannel logIndex')
      assert.equal(allChannels[0].channelEntry.logIndex.toNumber(), 2, 'check Channels.onOpenedChannel logIndex')
    })

    it('should not delete latest channel according to logIndex', async function () {
      // @ts-ignore
      await connector.indexer.onOpenedChannel({
        returnValues: {
          opener: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 0,
        logIndex: 2,
      } as any)

      // @ts-ignore
      await connector.indexer.onClosedChannel({
        returnValues: {
          closer: userA.address.toHex(),
          counterParty: userB.address.toHex(),
        },
        blockNumber: 1,
        transactionIndex: 0,
        logIndex: 1,
      } as any)

      const allChannels = await connector.indexer.get()
      assert.equal(allChannels.length, 1, 'check Channels.onOpenedChannel logIndex')
      assert.equal(allChannels[0].channelEntry.logIndex.toNumber(), 2, 'check Channels.onOpenedChannel logIndex')
    })
  })
})
