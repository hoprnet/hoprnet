import { BigNumber } from 'ethers'
import assert from 'assert'
import { ChannelEntry, Hash, ChannelStatus, defer } from '@hoprnet/hopr-utils'

import { expectAccountsToBeEqual, expectChannelsToBeEqual } from './fixtures'
import * as fixtures from './fixtures'
import { PARTY_A, PARTY_B } from '../fixtures'
import type { Event } from './types'
import { useFixtures } from './index.mock'

describe('test indexer', function () {
  it('should start indexer', async function () {
    const { indexer, chain } = await useFixtures()

    await indexer.start(chain, 0)
    assert.strictEqual(indexer.status, 'started')
  })

  it('should stop indexer', async function () {
    const { indexer, chain, hoprChannels, hoprToken, provider } = await useFixtures()

    await indexer.start(chain, 0)

    // Make sure that it assigns event listeners
    assert(hoprChannels.listeners('*').length > 0)
    assert(hoprToken.listeners('*').length > 0)
    assert(hoprChannels.listeners('error').length > 0)
    assert(hoprToken.listeners('error').length > 0)
    assert(provider.listeners('error').length > 0)
    assert(provider.listeners('block').length > 0)

    indexer.stop()

    // Make sure that it does the cleanup properly
    assert(hoprChannels.listeners('*').length == 0)
    assert(hoprToken.listeners('*').length == 0)
    assert(hoprChannels.listeners('error').length == 0)
    assert(hoprToken.listeners('error').length == 0)
    assert(provider.listeners('error').length == 0)
    assert(provider.listeners('block').length == 0)

    assert.strictEqual(indexer.status, 'stopped')
  })

  it('should restart the indexer', async function () {
    const { indexer, chain, hoprChannels, hoprToken, provider } = await useFixtures()

    await indexer.start(chain, 0)

    for (let i = 0; i < 5; i++) {
      await indexer.restart()
    }

    indexer.stop()

    // Make sure that it does the cleanup properly
    assert(hoprChannels.listeners('*').length == 0)
    assert(hoprToken.listeners('*').length == 0)
    assert(hoprChannels.listeners('error').length == 0)
    assert(hoprToken.listeners('error').length == 0)
    assert(provider.listeners('error').length == 0)
    assert(provider.listeners('block').length == 0)
  })

  it('should process 1 past event', async function () {
    const { indexer, OPENED_CHANNEL, chain, db } = await useFixtures({
      latestBlockNumber: 2,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })
    await indexer.start(chain, 0)

    const account = await indexer.getAccount(fixtures.PARTY_A.toAddress())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    assert.rejects(() => db.getChannel(OPENED_CHANNEL.getId()))
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
    const { indexer, newEvent, newBlock, OPENED_CHANNEL, chain, db } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.OPENED_EVENT)
    newBlock()

    const blockMined = defer<void>()

    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 4) blockMined.resolve()
    })

    await blockMined.promise

    const channel = await db.getChannel(OPENED_CHANNEL.getId())
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

  it('should emit events on updated channels', async function () {
    this.timeout(5000)
    const { indexer, newEvent, newBlock, chain } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT],
      id: fixtures.PARTY_A
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
    const { indexer, newEvent, newBlock, COMMITTED_CHANNEL, chain, db } = await useFixtures({
      latestBlockNumber: 3
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.PARTY_A_INITIALIZED_EVENT)
    newEvent(fixtures.PARTY_B_INITIALIZED_EVENT)
    newEvent(fixtures.COMMITTED_EVENT) // setting commited first to test event sorting
    newEvent(fixtures.OPENED_EVENT)

    newBlock()

    const blockMined = defer<void>()

    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 4) blockMined.resolve()
    })

    await blockMined.promise

    const channel = await db.getChannel(COMMITTED_CHANNEL.getId())
    expectChannelsToBeEqual(channel, COMMITTED_CHANNEL)
  })

  it('should process TicketRedeemed event and reduce outstanding balance for sender', async function () {
    const { indexer, newEvent, newBlock, chain, db } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [
        fixtures.PARTY_A_INITIALIZED_EVENT,
        fixtures.PARTY_B_INITIALIZED_EVENT,
        fixtures.OPENED_EVENT,
        fixtures.COMMITTED_EVENT
      ],
      id: fixtures.PARTY_A
    })
    // sender node has pending ticket...
    await db.markPending(fixtures.oneLargeTicket)
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '2')

    const blockMined = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 7) blockMined.resolve()
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()
    newBlock()
    newBlock()

    await blockMined.promise
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')
  })

  it('should process TicketRedeemed event and not reduce outstanding balance for sender when db has no outstanding balance', async function () {
    const { indexer, newEvent, newBlock, chain, db } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [
        fixtures.PARTY_A_INITIALIZED_EVENT,
        fixtures.PARTY_B_INITIALIZED_EVENT,
        fixtures.OPENED_EVENT,
        fixtures.COMMITTED_EVENT
      ],
      id: fixtures.PARTY_A
    })
    // sender node has pending ticket...
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')

    const blockMined = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 7) blockMined.resolve()
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()
    newBlock()
    newBlock()

    await blockMined.promise
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')
  })

  it('should process TicketRedeemed event and not reduce outstanding balance for recipient', async function () {
    const { indexer, newEvent, newBlock, chain, db } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [
        fixtures.PARTY_A_INITIALIZED_EVENT,
        fixtures.PARTY_B_INITIALIZED_EVENT,
        fixtures.OPENED_EVENT,
        fixtures.COMMITTED_EVENT
      ],
      id: fixtures.PARTY_B
    })
    // recipient node has no ticket...
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')

    const blockMined = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 7) blockMined.resolve()
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()
    newBlock()
    newBlock()

    await blockMined.promise

    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')
  })

  it('should process TicketRedeemed event and not reduce outstanding balance for a third node', async function () {
    const { indexer, newEvent, newBlock, chain, db } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [
        fixtures.PARTY_A_INITIALIZED_EVENT,
        fixtures.PARTY_B_INITIALIZED_EVENT,
        fixtures.OPENED_EVENT,
        fixtures.COMMITTED_EVENT
      ]
    })
    // recipient node has no ticket...
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')

    const blockMined = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 7) blockMined.resolve()
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()
    newBlock()
    newBlock()

    await blockMined.promise

    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')
  })

  it('should process TicketRedeemed event and reduce outstanding balance to zero for sender when some history is missing', async function () {
    const { indexer, newEvent, newBlock, chain, db } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [
        fixtures.PARTY_A_INITIALIZED_EVENT,
        fixtures.PARTY_B_INITIALIZED_EVENT,
        fixtures.OPENED_EVENT,
        fixtures.COMMITTED_EVENT
      ],
      id: fixtures.PARTY_A
    })
    // sender node has some pending tickets, but not the entire history...
    await db.markPending(fixtures.oneSmallTicket)
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '1')

    const blockMined = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 7) blockMined.resolve()
    })
    await indexer.start(chain, 0)

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()
    newBlock()
    newBlock()

    await blockMined.promise
    assert.equal((await db.getPendingBalanceTo(PARTY_A.toAddress())).toString(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B.toAddress())).toString(), '0')
  })
})
