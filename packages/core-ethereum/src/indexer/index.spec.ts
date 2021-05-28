import type { providers as Providers } from 'ethers'
import type { HoprChannels, TypedEvent } from '@hoprnet/hopr-ethereum'
import type { Event } from './types'
import type { ChainWrapper } from '../ethereum'
import assert from 'assert'
import EventEmitter from 'events'
import Indexer from '.'
import { ChannelEntry, HoprDB } from '@hoprnet/hopr-utils'
import { expectAccountsToBeEqual, expectChannelsToBeEqual } from './fixtures'
import Defer from 'p-defer'
import * as fixtures from './fixtures'

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

  const hoprChannels = new EventEmitter() as unknown as HoprChannels
  hoprChannels.queryFilter = async (): Promise<TypedEvent<any>[]> => pastEvents

  return {
    hoprChannels,
    newEvent(event: Event<any>) {
      hoprChannels.emit('*', event)
    }
  }
}

const createChainMock = (provider: Providers.WebSocketProvider, hoprChannels: HoprChannels): ChainWrapper => {
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
    getWallet: () => fixtures.ACCOUNT_A,
    setCommitment: async () => {}
  } as unknown as ChainWrapper
}

const useFixtures = (ops: { latestBlockNumber?: number; pastEvents?: Event<any>[] } = {}) => {
  const latestBlockNumber = ops.latestBlockNumber ?? 0
  const pastEvents = ops.pastEvents ?? []

  const db = HoprDB.createMock()
  const { provider, newBlock } = createProviderMock({ latestBlockNumber })
  const { hoprChannels, newEvent } = createHoprChannelsMock({ pastEvents })
  const chain = createChainMock(provider, hoprChannels)

  const indexer = new Indexer(0, db, chain, 1, 5)

  return {
    db,
    provider,
    newBlock,
    hoprChannels,
    newEvent,
    indexer
  }
}

describe('test indexer', function () {
  it('should start indexer', async function () {
    const { indexer } = useFixtures()

    await indexer.start()
    assert.strictEqual(indexer.status, 'started')
  })

  it('should stop indexer', async function () {
    const { indexer } = useFixtures()

    await indexer.start()
    await indexer.stop()
    assert.strictEqual(indexer.status, 'stopped')
  })

  it('should process 1 past event', async function () {
    const { indexer } = useFixtures({
      latestBlockNumber: 2,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT]
    })
    await indexer.start()

    const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const channel = await indexer.getChannel(fixtures.FUNDED_CHANNEL.getId())
    assert.strictEqual(typeof channel, 'undefined')
  })

  it('should process all past events', async function () {
    const { indexer } = useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT]
    })
    await indexer.start()

    const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const channel = await indexer.getChannel(fixtures.FUNDED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, fixtures.FUNDED_CHANNEL)
  })

  it('should continue processing events', async function () {
    const { indexer, newEvent, newBlock } = useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT]
    })
    await indexer.start()

    newEvent(fixtures.OPENED_EVENT)
    newBlock()

    setImmediate(async () => {
      const channel = await indexer.getChannel(fixtures.OPENED_CHANNEL.getId())
      expectChannelsToBeEqual(channel, fixtures.OPENED_CHANNEL)
    })
  })

  it('should get public key of addresses', async function () {
    const { indexer } = useFixtures({
      latestBlockNumber: 2,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT]
    })

    await indexer.start()

    const pubKey = await indexer.getPublicKeyOf(fixtures.PARTY_A.toAddress())
    assert.strictEqual(pubKey.toHex(), fixtures.PARTY_A.toHex())
  })

  it('should get all data from DB', async function () {
    const { indexer } = useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start()

    const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const channel = await indexer.getChannel(fixtures.OPENED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, fixtures.OPENED_CHANNEL)

    const channels = await indexer.getChannels()
    assert.strictEqual(channels.length, 1, 'expected channels')
    expectChannelsToBeEqual(channels[0], fixtures.OPENED_CHANNEL)

    const channelsOfPartyA = await indexer.getChannelsOf(fixtures.PARTY_A.toAddress())
    assert.strictEqual(channelsOfPartyA.length, 1)
    expectChannelsToBeEqual(channelsOfPartyA[0], fixtures.OPENED_CHANNEL)

    const channelsOfPartyB = await indexer.getChannelsOf(fixtures.PARTY_B.toAddress())
    assert.strictEqual(channelsOfPartyB.length, 1)
    expectChannelsToBeEqual(channelsOfPartyB[0], fixtures.OPENED_CHANNEL)
  })

  it('should handle provider error by restarting', async function () {
    const { indexer, provider } = useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start()

    provider.emit('error', new Error('MOCK'))

    setImmediate(async () => {
      assert.strictEqual(indexer.status, 'restarting')
    })
  })

  it('should contract error by restarting', async function () {
    const { indexer, hoprChannels } = useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start()

    hoprChannels.emit('error', new Error('MOCK'))

    setImmediate(async () => {
      assert.strictEqual(indexer.status, 'restarting')
    })
  })

  it('should emit events on updated channels', async function () {
    const { indexer, newEvent, newBlock } = useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT]
    })
    await indexer.start()

    const firstUpdate = Defer()
    const secondUpdate = Defer()

    indexer.on('own-channel-updated', (channel: ChannelEntry) => {
      if (channel.partyATicketEpoch.toBN().isZero()) {
        firstUpdate.resolve()
      }
      if (channel.partyATicketEpoch.toBN().eqn(1)) {
        secondUpdate.resolve()
      }
    })

    newEvent(fixtures.OPENED_EVENT)
    newBlock()

    newEvent(fixtures.COMMITMENT_SET)
    newBlock()

    await Promise.all([firstUpdate.promise, secondUpdate.promise])
  })
})
