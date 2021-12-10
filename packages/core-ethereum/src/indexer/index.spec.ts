import { providers as Providers, Wallet } from 'ethers'
import type { HoprChannels, HoprToken } from '@hoprnet/hopr-ethereum'
import type { ChannelEventNames, Event } from './types'
import type { ChainWrapper } from '../ethereum'
import assert from 'assert'
import EventEmitter from 'events'
import Indexer from '.'
import { Address, ChannelEntry, Hash, HoprDB, generateChannelId, ChannelStatus, defer } from '@hoprnet/hopr-utils'
import { expectAccountsToBeEqual, expectChannelsToBeEqual } from './fixtures'
import * as fixtures from './fixtures'
import { PARTY_A, PARTY_B, UNIT_TEST_MAX_CONFIRMATIONS } from '../fixtures'
import { BigNumber } from 'ethers'
import { getConfirmedBlockNumberOrUndefined } from './utils'
import { expect } from 'chai'

const txRequest = {
  to: fixtures.ACCOUNT_B.address,
  data: '0x0',
  value: 0,
  nonce: 0,
  gasPrice: 1
}

const createProviderMock = (ops: { latestBlockNumber?: number } = {}) => {
  let latestBlockNumber = ops.latestBlockNumber ?? 0

  const provider = new EventEmitter() as unknown as Providers.WebSocketProvider
  provider.getBlockNumber = async (): Promise<number> => latestBlockNumber

  return {
    provider,
    newBlock() {
      latestBlockNumber++
      provider.emit('block', latestBlockNumber)
    }
  }
}

const createHoprChannelsMock = (ops: { pastEvents?: Event<ChannelEventNames>[] } = {}) => {
  const pastEvents = ops.pastEvents ?? []
  const channels: any = {}
  const pubkeys: any = {}

  const handleEvent = (ev) => {
    if (ev.event == 'ChannelUpdated') {
      const updateEvent = ev as Event<'ChannelUpdated'>

      const eventChannelId = generateChannelId(
        Address.fromString(updateEvent.args.source),
        Address.fromString(updateEvent.args.destination)
      )
      channels[eventChannelId.toHex()] = updateEvent.args.newState
    } else if (ev.event == 'Announce') {
      pubkeys[ev.args.account] = ev.args.multiaddr
    } else {
      //throw new Error("MISSING EV HANDLER IN TEST")
    }
  }

  class FakeChannels extends EventEmitter {
    async channels(channelId: string) {
      for (let ev of pastEvents) {
        handleEvent(ev)
      }
      return channels[channelId]
    }

    async bumpChannel(_counterparty: string, _comm: string) {
      let newEvent = {
        event: 'ChannelUpdated',
        transactionHash: '',
        blockNumber: 3,
        transactionIndex: 0,
        logIndex: 0,
        args: {
          source: PARTY_B.toAddress().toHex(),
          destination: PARTY_A.toAddress().toHex(),
          newState: {
            balance: BigNumber.from('3'),
            commitment: Hash.create(new TextEncoder().encode('commA')).toHex(),
            ticketEpoch: BigNumber.from('1'),
            ticketIndex: BigNumber.from('0'),
            status: 2,
            channelEpoch: BigNumber.from('0'),
            closureTime: BigNumber.from('0')
          }
        } as any
      } as Event<'ChannelUpdated'>
      handleEvent(newEvent)
      this.emit('*', newEvent)
    }

    async queryFilter(_filter, fromBlock, toBlock) {
      return pastEvents.filter((e) => e.blockNumber >= fromBlock && e.blockNumber <= toBlock)
    }
  }

  const hoprChannels = new FakeChannels() as unknown as HoprChannels

  return {
    hoprChannels,
    pubkeys,
    newEvent(event: Event<ChannelEventNames>) {
      pastEvents.push(event)
      hoprChannels.emit('*', event)
    },
    reorg(blockNumber: number) {
      const indexes = pastEvents.map((pe, i) => {
        if (pe.blockNumber === blockNumber) return i
      })
      indexes.forEach((i) => pastEvents.splice(i, 1))
    }
  }
}
const createHoprTokenMock = () => {
  class FakeToken extends EventEmitter {
    async transfer() {
      let newEvent = {
        event: 'Transfer',
        transactionHash: '',
        blockNumber: 8,
        transactionIndex: 0,
        logIndex: 0,
        args: {
          source: PARTY_A.toAddress().toHex(),
          destination: PARTY_B.toAddress().toHex(),
          balance: BigNumber.from('1')
        } as any
      } as Event<'Transfer'>
      this.emit('*', newEvent)
    }

    async queryFilter() {
      return []
    }
  }

  const hoprToken = new FakeToken() as unknown as HoprToken

  return {
    hoprToken,
    newEvent(event: Event<any>) {
      hoprToken.emit('*', event)
    }
  }
}

const createChainMock = (
  provider: Providers.WebSocketProvider,
  hoprChannels: HoprChannels,
  hoprToken: HoprToken,
  account?: Wallet
): ChainWrapper => {
  return {
    getLatestBlockNumber: () => provider.getBlockNumber(),
    subscribeBlock: (cb) => provider.on('block', cb),
    subscribeError: (cb) => {
      provider.on('error', cb)
      hoprChannels.on('error', cb)
    },
    subscribeChannelEvents: (cb) => hoprChannels.on('*', cb),
    subscribeTokenEvents: (cb) => hoprToken.on('*', cb),
    getNativeTokenTransactionInBlock: async (_blockNumber: number, _isOutgoing: boolean = true) => [],
    updateConfirmedTransaction: (_hash: string) => {},
    unsubscribe: () => {
      provider.removeAllListeners()
      hoprChannels.removeAllListeners()
    },
    getChannels: () => hoprChannels,
    getToken: () => hoprToken,
    getWallet: () => account ?? fixtures.ACCOUNT_A,
    setCommitment: (counterparty: Address, commitment: Hash) =>
      hoprChannels.bumpChannel(counterparty.toHex(), commitment.toHex()),
    getAllQueuingTransactionRequests: () => [txRequest]
  } as unknown as ChainWrapper
}

const useFixtures = async (ops: { latestBlockNumber?: number; pastEvents?: Event<ChannelEventNames>[] } = {}) => {
  const latestBlockNumber = ops.latestBlockNumber ?? 0
  const pastEvents = ops.pastEvents ?? []

  const db = HoprDB.createMock()
  const { provider, newBlock } = createProviderMock({ latestBlockNumber })
  const { hoprChannels, newEvent, reorg } = createHoprChannelsMock({ pastEvents })
  const { hoprToken } = createHoprTokenMock()
  const chain = createChainMock(provider, hoprChannels, hoprToken)
  return {
    db,
    provider,
    newBlock,
    hoprChannels,
    newEvent,
    reorg,
    indexer: new Indexer(Address.fromString(fixtures.ACCOUNT_A.address), db, UNIT_TEST_MAX_CONFIRMATIONS, 5),
    chain,
    OPENED_CHANNEL: await ChannelEntry.fromSCEvent(fixtures.OPENED_EVENT, (a: Address) =>
      Promise.resolve(a.eq(PARTY_A.toAddress()) ? PARTY_A : PARTY_B)
    ),
    COMMITTED_CHANNEL: await ChannelEntry.fromSCEvent(fixtures.COMMITTED_EVENT, (a: Address) =>
      Promise.resolve(a.eq(PARTY_A.toAddress()) ? PARTY_A : PARTY_B)
    )
  }
}

describe('test indexer', function () {
  describe('Start and restart', function () {
    it('should start indexer', async function () {
      const { indexer, chain } = await useFixtures()

      await indexer.start(chain, 0)
      assert.strictEqual(indexer.status, 'started')
    })

    it('should stop indexer', async function () {
      const { indexer, chain } = await useFixtures()

      await indexer.start(chain, 0)
      await indexer.stop()
      assert.strictEqual(indexer.status, 'stopped')
    })

    it('should handle provider error by restarting', async function () {
      const { indexer, provider, chain } = await useFixtures({
        latestBlockNumber: 4,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
      })

      await indexer.start(chain, 0)

      provider.emit('error', new Error('MOCK'))

      assert.strictEqual(indexer.status, 'stopped')

      const started = defer<void>()
      indexer.on('status', (status: string) => {
        if (status === 'started') started.resolve()
      })
      await started.promise
      assert.strictEqual(indexer.status, 'started')
    })

    it('should handle provider error and resend queuing transactions', async function () {
      const { indexer, provider, chain } = await useFixtures({
        latestBlockNumber: 4,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
      })

      await indexer.start(chain, 0)
      provider.emit('error', new Error('ECONNRESET'))

      assert.strictEqual(indexer.status, 'stopped')

      const started = defer<void>()
      indexer.on('status', (status: string) => {
        if (status === 'started') started.resolve()
      })
      await started.promise
      assert.strictEqual(indexer.status, 'started')
    })

    it('should contract error by restarting', async function () {
      const { indexer, hoprChannels, chain } = await useFixtures({
        latestBlockNumber: 4,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
      })

      await indexer.start(chain, 0)

      hoprChannels.emit('error', new Error('MOCK'))

      const started = defer<void>()
      indexer.on('status', (status: string) => {
        if (status === 'started') started.resolve()
      })
      await started.promise
      assert.strictEqual(indexer.status, 'started')
    })
  })

  describe('Process right blocks', function () {
    // When provider starts with the following block number, indexer should process latestBlock - UNIT_TEST_MAX_CONFIRMATIONS
    const providerStarterArray = [0, 1, 2, 3, 4]
    providerStarterArray.forEach((latestBlockNumber) => {
      it(`should process right blocks when provider starts from ${latestBlockNumber}`, async function () {
        const advance = 3 // number of blocks to advance
        const genesisBlock = 0
        const { indexer, newBlock, chain, provider } = await useFixtures({
          latestBlockNumber
        })

        const shouldProviderBlocks = Array.from({ length: advance }, (_v, i) => latestBlockNumber + i + 1)
        // [emit the last processed past block number (if applicable), and confirmed block number from all the new blocks]
        const shouldIndexerBlocks = [latestBlockNumber, ...shouldProviderBlocks]
          .map((v) => v - UNIT_TEST_MAX_CONFIRMATIONS)
          .filter((v) => v >= 0)
        const providerPromise = defer<void>()
        const indexerPromise = defer<void>()
        const indexerProcessedPromise = defer<void>()
        const providerBlocks = []
        const indexerBlocks = []
        const indexerProcessedBlocks = []
        provider.on('block', (blockNumber: number) => {
          providerBlocks.push(blockNumber)
          if (providerBlocks.length === shouldProviderBlocks.length) providerPromise.resolve()
        })
        indexer.on('block', (blockNumber: number) => {
          indexerBlocks.push(blockNumber)
          if (indexerBlocks.length === shouldProviderBlocks.length) indexerPromise.resolve()
        })
        indexer.on('block-processed', (blockNumber: number) => {
          indexerProcessedBlocks.push(blockNumber)
          if (indexerProcessedBlocks.length === shouldIndexerBlocks.length) indexerProcessedPromise.resolve()
        })

        await indexer.start(chain, genesisBlock)

        for (let index = 0; index < advance; index++) {
          newBlock()
        }

        await Promise.all([providerPromise.promise, indexerPromise.promise, indexerProcessedPromise.promise])

        assert.deepStrictEqual(providerBlocks, shouldProviderBlocks)
        assert.deepStrictEqual(indexerBlocks, shouldProviderBlocks)
        assert.deepStrictEqual(indexerProcessedBlocks, shouldIndexerBlocks)
      })
    })

    it('should catch re-orged events', async function () {
      const { indexer, newBlock, COMMITTED_CHANNEL, chain, db, newEvent } = await useFixtures({
        latestBlockNumber: 1,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
      })

      const blockMined = defer<void>()
      indexer.on('block-processed', (blockNumber: number) => {
        if (blockNumber === 2) blockMined.resolve()
      })

      await indexer.start(chain, 0) // genesisBlock: 0
      newBlock() // block nr. 2 => settle block nr. 0
      newBlock() // block nr. 3 => settle block nr. 1
      // block nr.2 gets re-orged. Missing events are included
      newEvent(fixtures.OPENED_EVENT)
      newEvent(fixtures.COMMITTED_EVENT)
      // reorg(fixtures.OPENED_EVENT.blockNumber)
      newBlock() // block nr. 4 => settle block nr. 2

      await blockMined.promise

      const channel = await db.getChannel(COMMITTED_CHANNEL.getId())
      expectChannelsToBeEqual(channel, COMMITTED_CHANNEL)
    })

    it('should ignore re-orged events', async function () {
      const { indexer, newBlock, COMMITTED_CHANNEL, chain, db, reorg } = await useFixtures({
        latestBlockNumber: 2,
        pastEvents: [
          fixtures.PARTY_A_INITIALIZED_EVENT,
          fixtures.PARTY_B_INITIALIZED_EVENT,
          fixtures.OPENED_EVENT,
          fixtures.COMMITTED_EVENT
        ]
      })

      const blockMined = defer<void>()
      const blockProcessed = defer<void>()
      const eventsCaught = [] // save the number of listened events into an array.
      indexer.on('block-processed', (blockNumber: number) => {
        if (blockNumber === 2) blockMined.resolve()
      })
      indexer.on('block', (blockNumber: number) => {
        eventsCaught.push(indexer.confirmedEvents.length)
        if (blockNumber === 4) blockProcessed.resolve()
      })
      await indexer.start(chain, 0) // genesisBlock: 0
      newBlock() // block nr. 3 => settle block nr. 1
      // block nr.2 gets re-orged. Some events are missing
      reorg(fixtures.OPENED_EVENT.blockNumber)
      newBlock() // block nr. 4 => settle block nr. 2
      await Promise.all([blockProcessed.promise, blockMined.promise])

      // Heard events should be an array of zeros
      assert.deepStrictEqual(eventsCaught, [0, 0])
    })
  })

  describe('Process events', function () {
    it('should process 1 past event', async function () {
      const { indexer, OPENED_CHANNEL, chain, db } = await useFixtures({
        latestBlockNumber: 4,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
      })

      await indexer.start(chain, 0)

      const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
      expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

      assert.rejects(() => db.getChannel(OPENED_CHANNEL.getId()))
    })

    it('should process all past events', async function () {
      const { indexer, chain, db, OPENED_CHANNEL } = await useFixtures({
        latestBlockNumber: 4,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
      })
      await indexer.start(chain, 0)

      const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
      expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

      const account2 = await indexer.getAccount(fixtures.PARTY_B.toAddress())
      expectAccountsToBeEqual(account2, fixtures.PARTY_B_INITIALIZED_ACCOUNT)

      const channel = await db.getChannel(OPENED_CHANNEL.getId())
      expectChannelsToBeEqual(channel, OPENED_CHANNEL)
    })

    it('should continue processing events', async function () {
      const { indexer, newEvent, newBlock, OPENED_CHANNEL, chain, db } = await useFixtures({
        latestBlockNumber: 1,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
      })

      const blockMined = defer<void>()
      indexer.on('block-processed', (blockNumber: number) => {
        if (blockNumber === 2) blockMined.resolve()
      })
      await indexer.start(chain, 0)
      newEvent(fixtures.OPENED_EVENT) // in block number 2
      newBlock() // block nr.2 => settle block nr. 0
      newBlock() // block nr.3 => settle block nr. 1
      newBlock() // block nr.4 => settle block nr. 2
      await blockMined.promise

      const channel = await db.getChannel(OPENED_CHANNEL.getId())
      expectChannelsToBeEqual(channel, OPENED_CHANNEL)
    })

    it('should get public key of addresses', async function () {
      const { indexer, chain, newBlock } = await useFixtures({
        latestBlockNumber: 2,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT]
      })

      const blockMined = defer<void>()
      indexer.on('block-processed', (blockNumber: number) => {
        if (blockNumber === 1) blockMined.resolve()
      })
      await indexer.start(chain, 0)
      newBlock() // block nr. 3 => settle block nr. 1
      await blockMined.promise

      const pubKey = await indexer.getPublicKeyOf(fixtures.PARTY_A.toAddress())
      assert.strictEqual(pubKey.toHex(), fixtures.PARTY_A.toHex())
    })

    it('should get all data from DB', async function () {
      const { indexer, OPENED_CHANNEL, chain, db } = await useFixtures({
        latestBlockNumber: 4,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
      })

      await indexer.start(chain, 0)

      const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
      expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

      const channel = await db.getChannel(OPENED_CHANNEL.getId())
      expectChannelsToBeEqual(channel, OPENED_CHANNEL)

      const channels = await db.getChannels()
      assert.strictEqual(channels.length, 1, 'expected channels')
      expectChannelsToBeEqual(channels[0], OPENED_CHANNEL)

      const channelsFromPartyA = await db.getChannelsFrom(fixtures.PARTY_A.toAddress())
      assert.strictEqual(channelsFromPartyA.length, 1)
      expectChannelsToBeEqual(channelsFromPartyA[0], OPENED_CHANNEL)

      const channelsOfPartyB = await db.getChannelsFrom(fixtures.PARTY_B.toAddress())
      assert.strictEqual(channelsOfPartyB.length, 0)
    })

    it('should emit events on updated channels', async function () {
      this.timeout(5000)
      const { indexer, newEvent, newBlock, chain } = await useFixtures({
        latestBlockNumber: 3,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
      })

      const opened = defer<void>()
      const pendingIniated = defer<void>()
      const closed = defer<void>()

      indexer.on('own-channel-updated', (channel: ChannelEntry) => {
        switch (channel.status) {
          case ChannelStatus.WaitingForCommitment:
            opened.resolve()
            break
          case ChannelStatus.PendingToClose: {
            pendingIniated.resolve()
            break
          }
          case ChannelStatus.Closed: {
            closed.resolve()
            break
          }
        }
      })

      await indexer.start(chain, 0)
      const ev = {
        event: 'ChannelUpdated',
        transactionHash: '',
        blockNumber: 2,
        transactionIndex: 0,
        logIndex: 0,
        args: {
          source: PARTY_B.toAddress().toHex(),
          destination: PARTY_A.toAddress().toHex(),
          newState: {
            balance: BigNumber.from('3'),
            commitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
            ticketEpoch: BigNumber.from('0'),
            ticketIndex: BigNumber.from('0'),
            status: 1,
            channelEpoch: BigNumber.from('0'),
            closureTime: BigNumber.from('0')
          }
        } as any
      } as Event<'ChannelUpdated'>
      // We are ACCOUNT_A - if B opens a channel to us, we should automatically
      // commit.
      newEvent(ev)

      newBlock()
      newBlock()
      await opened.promise

      const evClose = {
        event: 'ChannelUpdated',
        transactionHash: '',
        blockNumber: 5,
        transactionIndex: 0,
        logIndex: 0,
        args: {
          source: PARTY_B.toAddress().toHex(),
          destination: PARTY_A.toAddress().toHex(),
          newState: {
            balance: BigNumber.from('3'),
            commitment: Hash.create(new TextEncoder().encode('commA')).toHex(),
            ticketEpoch: BigNumber.from('1'),
            ticketIndex: BigNumber.from('0'),
            status: 3,
            channelEpoch: BigNumber.from('0'),
            closureTime: BigNumber.from('0'),
            closureByPartyA: true
          }
        } as any
      } as Event<'ChannelUpdated'>
      newEvent(evClose)
      newBlock()
      newBlock()

      await pendingIniated.promise

      const evClosed = {
        event: 'ChannelUpdated',
        transactionHash: '',
        blockNumber: 7,
        transactionIndex: 0,
        logIndex: 0,
        args: {
          source: PARTY_B.toAddress().toHex(),
          destination: PARTY_A.toAddress().toHex(),
          newState: {
            balance: BigNumber.from('0'),
            commitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
            ticketEpoch: BigNumber.from('0'),
            ticketIndex: BigNumber.from('0'),
            status: 0,
            channelEpoch: BigNumber.from('0'),
            closureTime: BigNumber.from('0'),
            closureByPartyA: false
          }
        } as any
      } as Event<'ChannelUpdated'>

      newEvent(evClosed)
      newBlock()
      newBlock()

      await closed.promise
    })

    it('should process events in the right order', async function () {
      this.timeout(5000)
      const { indexer, newEvent, newBlock, COMMITTED_CHANNEL, chain, db } = await useFixtures({
        latestBlockNumber: 3,
        pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
      })

      const blockMined = defer<void>()
      indexer.on('block-processed', (blockNumber: number) => {
        if (blockNumber === 2) blockMined.resolve()
      })
      await indexer.start(chain, 0)
      newEvent(fixtures.COMMITTED_EVENT) // setting commited first to test event sorting
      newEvent(fixtures.OPENED_EVENT)
      newBlock() // block nr. 4 => settle block nr. 2
      await blockMined.promise

      indexer.on('block-processed', (blockNumber: number) => {
        if (blockNumber === 4) blockMined.resolve()
      })

      const channel = await db.getChannel(COMMITTED_CHANNEL.getId())
      expectChannelsToBeEqual(channel, COMMITTED_CHANNEL)
    })
  })
})
