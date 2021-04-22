import type { providers as Providers } from 'ethers'
import type { HoprChannels } from '../contracts'
import type { Event } from './types'
import type { TypedEvent } from '../contracts/commons'
import assert from 'assert'
import EventEmitter from 'events'
import LevelUp from 'levelup'
import MemDown from 'memdown'
import Indexer from '.'
import { expectAccountsToBeEqual, expectChannelsToBeEqual } from './fixtures'
import * as fixtures from './fixtures'

const createProviderMock = (ops: { latestBlockNumber?: number } = {}) => {
  let latestBlockNumber = ops.latestBlockNumber ?? 0

  const provider = (new EventEmitter() as unknown) as Providers.WebSocketProvider
  provider.getBlockNumber = async (): Promise<number> => latestBlockNumber

  return {
    provider,
    newBlock() {
      latestBlockNumber++
      // provider.emit('block', latestBlockNumber)
    }
  }
}

const createHoprChannelsMock = (ops: { pastEvents?: Event<any>[] } = {}) => {
  const pastEvents = ops.pastEvents ?? []

  const hoprChannels = (new EventEmitter() as unknown) as HoprChannels
  hoprChannels.queryFilter = async (): Promise<TypedEvent<any>[]> => pastEvents

  return {
    hoprChannels,
    newEvent(event: Event<any>) {
      hoprChannels.emit('*', event)
    }
  }
}

const useFixtures = (ops: { latestBlockNumber?: number; pastEvents?: Event<any>[] } = {}) => {
  const latestBlockNumber = ops.latestBlockNumber ?? 0
  const pastEvents = ops.pastEvents ?? []

  const db = new LevelUp(MemDown())
  const { provider, newBlock } = createProviderMock({ latestBlockNumber })
  const { hoprChannels, newEvent } = createHoprChannelsMock({ pastEvents })

  const indexer = new Indexer(
    {
      genesisBlock: 0,
      maxConfirmations: 1,
      blockRange: 5
    },
    {
      db,
      provider,
      hoprChannels
    }
  )

  return {
    db,
    provider,
    newBlock,
    hoprChannels,
    newEvent,
    indexer
  }
}

describe.only('test indexer', function () {
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

    const account = await indexer.getAccount(fixtures.partyA.toAddress())
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

    const account = await indexer.getAccount(fixtures.partyA.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const channel = await indexer.getChannel(fixtures.FUNDED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, fixtures.FUNDED_CHANNEL)
  })

  it.only('should continue processing events', async function () {
    const { indexer, provider, newBlock } = useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.FUNDED_EVENT]
    })
    await indexer.start()

    // @ts-ignore
    indexer.onNewEvents([fixtures.OPENED_EVENT])

    newBlock()
    console.log(await provider.getBlockNumber())
    // @ts-ignore
    await indexer.onNewBlock({ number: await provider.getBlockNumber() })

    console.log(indexer.latestBlock)

    const channel = await indexer.getChannel(fixtures.OPENED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, fixtures.OPENED_CHANNEL)
  })
})
