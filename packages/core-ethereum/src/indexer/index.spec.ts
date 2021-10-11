import type { providers as Providers, Wallet } from 'ethers'
import type { HoprChannels } from '@hoprnet/hopr-ethereum'
import type { Event } from './types'
import type { ChainWrapper } from '../ethereum'
import assert from 'assert'
import EventEmitter from 'events'
import Indexer from '.'
import { Address, ChannelEntry, Defer, Hash, HoprDB, generateChannelId, ChannelStatus } from '@hoprnet/hopr-utils'
import { expectAccountsToBeEqual, expectChannelsToBeEqual } from './fixtures'
import * as fixtures from './fixtures'
import { PARTY_A, PARTY_B } from '../fixtures'
import { BigNumber } from 'ethers'

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

const createHoprChannelsMock = (ops: { pastEvents?: Event<any>[] } = {}) => {
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

    async queryFilter() {
      return pastEvents
    }
  }

  const hoprChannels = new FakeChannels() as unknown as HoprChannels

  return {
    hoprChannels,
    pubkeys,
    newEvent(event: Event<any>) {
      pastEvents.push(event)
      hoprChannels.emit('*', event)
    }
  }
}

const createChainMock = (
  provider: Providers.WebSocketProvider,
  hoprChannels: HoprChannels,
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
    unsubscribe: () => {
      provider.removeAllListeners()
      hoprChannels.removeAllListeners()
    },
    getChannels: () => hoprChannels,
    getWallet: () => account ?? fixtures.ACCOUNT_A,
    setCommitment: (counterparty: Address, commitment: Hash) =>
      hoprChannels.bumpChannel(counterparty.toHex(), commitment.toHex())
  } as unknown as ChainWrapper
}

const useFixtures = async (ops: { latestBlockNumber?: number; pastEvents?: Event<any>[] } = {}) => {
  const latestBlockNumber = ops.latestBlockNumber ?? 0
  const pastEvents = ops.pastEvents ?? []

  const db = HoprDB.createMock()
  const { provider, newBlock } = createProviderMock({ latestBlockNumber })
  const { hoprChannels, newEvent } = createHoprChannelsMock({ pastEvents })
  const chain = createChainMock(provider, hoprChannels)
  return {
    db,
    provider,
    newBlock,
    hoprChannels,
    newEvent,
    indexer: new Indexer(Address.fromString(fixtures.ACCOUNT_A.address), db, 1, 5),
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

  it('should process 1 past event', async function () {
    const { indexer, OPENED_CHANNEL, chain } = await useFixtures({
      latestBlockNumber: 2,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })
    await indexer.start(chain, 0)

    const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    assert.rejects(() => indexer.getChannel(OPENED_CHANNEL.getId()))
  })

  it('should process all past events', async function () {
    const { indexer, chain } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
    })
    await indexer.start(chain, 0)

    const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const account2 = await indexer.getAccount(fixtures.PARTY_B.toAddress())
    expectAccountsToBeEqual(account2, fixtures.PARTY_B_INITIALIZED_ACCOUNT)
  })

  it('should continue processing events', async function () {
    const { indexer, newEvent, newBlock, OPENED_CHANNEL, chain } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.OPENED_EVENT)
    newBlock()

    const blockMined = new Defer()

    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 4) blockMined.resolve()
    })

    await blockMined.promise

    const channel = await indexer.getChannel(OPENED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, OPENED_CHANNEL)
  })

  it('should get public key of addresses', async function () {
    const { indexer, chain } = await useFixtures({
      latestBlockNumber: 2,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT]
    })

    await indexer.start(chain, 0)

    const pubKey = await indexer.getPublicKeyOf(fixtures.PARTY_A.toAddress())
    assert.strictEqual(pubKey.toHex(), fixtures.PARTY_A.toHex())
  })

  it('should get all data from DB', async function () {
    const { indexer, OPENED_CHANNEL, chain } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start(chain, 0)

    const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const channel = await indexer.getChannel(OPENED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, OPENED_CHANNEL)

    const channels = await indexer.getChannels()
    assert.strictEqual(channels.length, 1, 'expected channels')
    expectChannelsToBeEqual(channels[0], OPENED_CHANNEL)

    const channelsFromPartyA = await indexer.getChannelsFrom(fixtures.PARTY_A.toAddress())
    assert.strictEqual(channelsFromPartyA.length, 1)
    expectChannelsToBeEqual(channelsFromPartyA[0], OPENED_CHANNEL)

    const channelsOfPartyB = await indexer.getChannelsFrom(fixtures.PARTY_B.toAddress())
    assert.strictEqual(channelsOfPartyB.length, 0)
  })

  it('should handle provider error by restarting', async function () {
    const { indexer, provider, chain } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start(chain, 0)

    provider.emit('error', new Error('MOCK'))

    assert.strictEqual(indexer.status, 'stopped')

    const started = new Defer()
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

    const started = new Defer()
    indexer.on('status', (status: string) => {
      if (status === 'started') started.resolve()
    })
    await started.promise
    assert.strictEqual(indexer.status, 'started')
  })

  it('should emit events on updated channels', async function () {
    this.timeout(5000)
    const { indexer, newEvent, newBlock, chain } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
    })

    const opened = new Defer()
    const pendingIniated = new Defer()
    const closed = new Defer()

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
    const { indexer, newEvent, newBlock, COMMITTED_CHANNEL, chain } = await useFixtures({
      latestBlockNumber: 3
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.PARTY_A_INITIALIZED_EVENT)
    newEvent(fixtures.PARTY_B_INITIALIZED_EVENT)
    newEvent(fixtures.COMMITTED_EVENT) // setting commited first to test event sorting
    newEvent(fixtures.OPENED_EVENT)

    newBlock()

    const blockMined = new Defer()

    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 4) blockMined.resolve()
    })

    await blockMined.promise

    const channel = await indexer.getChannel(COMMITTED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, COMMITTED_CHANNEL)
  })
})
