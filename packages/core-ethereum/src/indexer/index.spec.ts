import { BigNumber } from 'ethers'
import assert from 'assert'
import { ChannelEntry, Hash, ChannelStatus, defer, PublicKey } from '@hoprnet/hopr-utils'

import { expectAccountsToBeEqual, expectChannelsToBeEqual, PARTY_A, PARTY_B, PARTY_B_MULTIADDR } from './fixtures.js'
import * as fixtures from './fixtures.js'
import { type Event, IndexerStatus } from './types.js'
import { useFixtures } from './index.mock.js'
import { SendTransactionStatus } from '../ethereum.js'

describe('test indexer', function () {
  it('should start indexer', async function () {
    const { indexer, chain } = await useFixtures()

    await indexer.start(chain, 0)
    assert.strictEqual(indexer.status, IndexerStatus.STARTED)
  })

  it('should stop indexer', async function () {
    const { indexer, chain, hoprChannels, hoprToken, provider } = await useFixtures()

    await indexer.start(chain, 0)

    // Make sure that it assigns event listeners
    assert(hoprChannels.listeners('error').length > 0)
    assert(hoprToken.listeners('error').length > 0)
    assert(provider.listeners('error').length > 0)
    assert(provider.listeners('block').length > 0)

    indexer.stop()

    // Make sure that it does the cleanup properly
    assert(hoprChannels.listeners('error').length == 0)
    assert(hoprToken.listeners('error').length == 0)
    assert(provider.listeners('error').length == 0)
    assert(provider.listeners('block').length == 0)

    assert.strictEqual(indexer.status, IndexerStatus.STOPPED)
  })

  it('should restart the indexer', async function () {
    const { indexer, chain, hoprChannels, hoprToken, provider } = await useFixtures()

    await indexer.start(chain, 0)

    for (let i = 0; i < 5; i++) {
      await indexer.restart()
    }

    indexer.stop()

    // Make sure that it does the cleanup properly
    assert(hoprChannels.listeners('error').length == 0)
    assert(hoprToken.listeners('error').length == 0)
    assert(provider.listeners('error').length == 0)
    assert(provider.listeners('block').length == 0)
  })

  it('should process 1 past event', async function () {
    const { indexer, OPENED_CHANNEL, chain, db } = await useFixtures({
      latestBlockNumber: 2,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 1) {
        blockProcessed.resolve()
      }
    })

    await indexer.start(chain, 0)

    await blockProcessed.promise

    const account = await indexer.getAccount(fixtures.PARTY_A().to_address())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    assert.rejects(() => db.getChannel(OPENED_CHANNEL.get_id()))
  })

  it('should process all past events', async function () {
    const { indexer, chain } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
    })

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 2) {
        blockProcessed.resolve()
      }
    })

    await indexer.start(chain, 0)

    await blockProcessed.promise

    const account = await indexer.getAccount(fixtures.PARTY_A().to_address())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const account2 = await indexer.getAccount(fixtures.PARTY_B().to_address())
    expectAccountsToBeEqual(account2, fixtures.PARTY_B_INITIALIZED_ACCOUNT)
  })

  it('should continue processing events', async function () {
    const { indexer, newEvent, newBlock, OPENED_CHANNEL, chain, db } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT]
    })
    await indexer.start(chain, 0)

    const blockProcessed = defer<void>()

    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 4) {
        blockProcessed.resolve()
      }
    })

    // confirmations == 1
    newBlock()

    newEvent(fixtures.OPENED_EVENT)
    newBlock()

    await blockProcessed.promise

    const channel = await db.getChannel(OPENED_CHANNEL.get_id())
    expectChannelsToBeEqual(channel, OPENED_CHANNEL)
  })

  it('should get public key of addresses', async function () {
    const { indexer, chain } = await useFixtures({
      latestBlockNumber: 2,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT]
    })

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 1) {
        blockProcessed.resolve()
      }
    })

    await indexer.start(chain, 0)

    await blockProcessed.promise

    const pubKey = await indexer.getPublicKeyOf(fixtures.PARTY_A().to_address())
    assert(pubKey.eq(fixtures.PARTY_A()))
  })

  it('should get all data from DB', async function () {
    const { indexer, OPENED_CHANNEL, chain, db } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 3) {
        blockProcessed.resolve()
      }
    })

    await indexer.start(chain, 0)

    await blockProcessed.promise

    const account = await indexer.getAccount(fixtures.PARTY_A().to_address())
    expectAccountsToBeEqual(account, fixtures.PARTY_A_INITIALIZED_ACCOUNT)

    const channel = await db.getChannel(OPENED_CHANNEL.get_id())
    expectChannelsToBeEqual(channel, OPENED_CHANNEL)

    const channels = await db.getChannels()
    assert.strictEqual(channels.length, 1, 'expected channels')
    expectChannelsToBeEqual(channels[0], OPENED_CHANNEL)

    const channelsFromPartyA = await db.getChannelsFrom(fixtures.PARTY_A().to_address())
    assert.strictEqual(channelsFromPartyA.length, 1)
    expectChannelsToBeEqual(channelsFromPartyA[0], OPENED_CHANNEL)

    const channelsOfPartyB = await db.getChannelsFrom(fixtures.PARTY_B().to_address())
    assert.strictEqual(channelsOfPartyB.length, 0)
  })

  it('should handle provider error by restarting', async function () {
    const { indexer, provider, chain } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start(chain, 0)

    const indexerStopped = defer<void>()
    indexer.on('status', (status: string) => {
      if (status === 'stopped') {
        indexerStopped.resolve()
      }
    })

    provider.emit('error', new Error('MOCK'))

    await indexerStopped.promise

    // Indexer is either stopped or restarting
    assert([IndexerStatus.STOPPED, IndexerStatus.RESTARTING].includes(indexer.status))

    const started = defer<void>()
    indexer.on('status', (status: string) => {
      if (status === 'started') started.resolve()
    })
    await started.promise
    assert.strictEqual(indexer.status, IndexerStatus.STARTED)
  })

  it('should handle provider error and resend queuing transactions', async function () {
    const { indexer, provider, chain } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start(chain, 0)

    const indexerStopped = defer<void>()
    indexer.on('status', (status: string) => {
      if (status === 'stopped') {
        indexerStopped.resolve()
      }
    })

    provider.emit('error', new Error('ECONNRESET'))

    await indexerStopped.promise

    // Indexer is either stopped or restarting
    assert([IndexerStatus.STOPPED, IndexerStatus.RESTARTING].includes(indexer.status))

    const started = defer<void>()
    indexer.on('status', (status: string) => {
      if (status === 'started') started.resolve()
    })
    await started.promise
    assert.strictEqual(indexer.status, IndexerStatus.STARTED)
  })

  it('should contract error by restarting', async function () {
    const { indexer, hoprChannels, chain } = await useFixtures({
      latestBlockNumber: 4,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT, fixtures.OPENED_EVENT]
    })

    await indexer.start(chain, 0)

    hoprChannels.emit('error', new Error('MOCK'))

    const started = defer<void>()
    indexer.on('status', (status: string) => {
      if (status === 'started') started.resolve()
    })
    await started.promise
    assert.strictEqual(indexer.status, IndexerStatus.STARTED)
  })

  it('should emit events on updated channels', async function () {
    this.timeout(5000)
    const { indexer, newEvent, newBlock, chain } = await useFixtures({
      latestBlockNumber: 3,
      pastEvents: [fixtures.PARTY_A_INITIALIZED_EVENT, fixtures.PARTY_B_INITIALIZED_EVENT],
      id: fixtures.PARTY_A()
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
        source: PARTY_B().to_address().to_hex(),
        destination: PARTY_A().to_address().to_hex(),
        newState: {
          balance: BigNumber.from('3'),
          commitment: new Hash(new Uint8Array({ length: Hash.size() })).to_hex(),
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
        source: PARTY_B().to_address().to_hex(),
        destination: PARTY_A().to_address().to_hex(),
        newState: {
          balance: BigNumber.from('3'),
          commitment: Hash.create([new TextEncoder().encode('commA')]).to_hex(),
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
        source: PARTY_B().to_address().to_hex(),
        destination: PARTY_A().to_address().to_hex(),
        newState: {
          balance: BigNumber.from('0'),
          commitment: new Hash(new Uint8Array({ length: Hash.size() })).to_hex(),
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

    const blockProcessed = defer<void>()

    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber === 4) {
        blockProcessed.resolve()
      }
    })

    // confirmations == 1
    newBlock()

    newEvent(fixtures.PARTY_A_INITIALIZED_EVENT)
    newEvent(fixtures.PARTY_B_INITIALIZED_EVENT)
    newEvent(fixtures.COMMITTED_EVENT) // setting commited first to test event sorting
    newEvent(fixtures.OPENED_EVENT)

    newBlock()

    await blockProcessed.promise

    const channel = await db.getChannel(COMMITTED_CHANNEL.get_id())
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
      id: fixtures.PARTY_A()
    })
    // sender node has pending ticket...
    await db.markPending(fixtures.oneLargeTicket)
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '2')

    const blockMined = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 5) {
        blockMined.resolve()
      }
    })
    await indexer.start(chain, 0)

    // confirmations == 1
    newBlock()

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()

    await blockMined.promise
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')
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
      id: fixtures.PARTY_A()
    })
    // sender node has pending ticket...
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 5) {
        blockProcessed.resolve()
      }
    })
    await indexer.start(chain, 0)

    // confirmations == 1
    newBlock()

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()

    await blockProcessed.promise
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')
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
      id: fixtures.PARTY_B()
    })
    // recipient node has no ticket...
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 5) {
        blockProcessed.resolve()
      }
    })

    await indexer.start(chain, 0)

    // confirmations == 1
    newBlock()

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()

    await blockProcessed.promise

    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')
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
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 5) {
        blockProcessed.resolve()
      }
    })
    await indexer.start(chain, 0)

    // confirmations == 1
    newBlock()

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()

    await blockProcessed.promise

    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')
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
      id: fixtures.PARTY_A()
    })
    // sender node has some pending tickets, but not the entire history...
    await db.markPending(fixtures.oneSmallTicket)
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '1')

    const blockProcessed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 5) {
        blockProcessed.resolve()
      }
    })
    await indexer.start(chain, 0)

    // confirmations == 1
    newBlock()

    newEvent(fixtures.UPDATED_WHEN_REDEEMED_EVENT)
    newEvent(fixtures.TICKET_REDEEMED_EVENT)
    newBlock()

    await blockProcessed.promise
    assert.equal((await db.getPendingBalanceTo(PARTY_A().to_address())).to_string(), '0')
    assert.equal((await db.getPendingBalanceTo(PARTY_B().to_address())).to_string(), '0')
  })

  it('should process Transfer events and reduce balance', async function () {
    const { indexer, chain, newBlock, newTokenEvent, db } = await useFixtures({
      latestBlockNumber: 0,
      pastEvents: [],
      id: fixtures.PARTY_A()
    })

    const secondBlockProcessed = defer<void>()
    const thirdBlockProcessed = defer<void>()

    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 1) {
        secondBlockProcessed.resolve()
      } else if (blockNumber == 2) {
        thirdBlockProcessed.resolve()
      }
    })

    await indexer.start(chain, 0)

    assert.equal((await db.getHoprBalance()).to_string(), '0')

    // confirmations == 1
    newBlock()

    newTokenEvent(fixtures.PARTY_A_TRANSFER_INCOMING) // +3
    newBlock()

    await secondBlockProcessed.promise

    newTokenEvent(fixtures.PARTY_A_TRANSFER_OUTGOING) // -1
    newBlock()

    await thirdBlockProcessed.promise

    assert.equal((await db.getHoprBalance()).to_string(), '2')
  })

  it('should process first 2 registry events and account be registered and eligible', async function () {
    const { db, chain, indexer, newBlock } = await useFixtures({
      latestBlockNumber: 10,
      pastHoprRegistryEvents: [fixtures.PARTY_A_REGISTERED, fixtures.PARTY_A_ELEGIBLE],
      id: fixtures.PARTY_A()
    })

    const processed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 10) processed.resolve()
    })
    await indexer.start(chain, 0)

    newBlock()
    await processed.promise
    assert(await db.getAccountFromNetworkRegistry(PublicKey.from_peerid_str(PARTY_B_MULTIADDR.getPeerId())))
    assert(await db.isEligible(fixtures.PARTY_A().to_address()))
  })

  it('should process first 4 registry events and account not be eligible', async function () {
    const { db, chain, indexer, newBlock } = await useFixtures({
      latestBlockNumber: 10,
      pastHoprRegistryEvents: [fixtures.PARTY_A_REGISTERED, fixtures.PARTY_A_ELEGIBLE, fixtures.PARTY_A_NOT_ELEGIBLE],
      id: fixtures.PARTY_A()
    })

    const processed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 10) processed.resolve()
    })
    await indexer.start(chain, 0)

    newBlock()
    await processed.promise
    assert(await db.getAccountFromNetworkRegistry(PublicKey.from_peerid_str(PARTY_B_MULTIADDR.getPeerId())))
    assert((await db.isEligible(fixtures.PARTY_A().to_address())) === false)
  })

  it('should process all registry events and account not be registered but be eligible', async function () {
    const { db, chain, indexer, newBlock } = await useFixtures({
      latestBlockNumber: 10,
      pastHoprRegistryEvents: [
        fixtures.PARTY_A_REGISTERED,
        fixtures.PARTY_A_ELEGIBLE,
        fixtures.PARTY_A_NOT_ELEGIBLE,
        fixtures.PARTY_A_ELEGIBLE_2,
        fixtures.PARTY_A_DEREGISTERED
      ],
      id: fixtures.PARTY_A()
    })

    const processed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 10) processed.resolve()
    })
    await indexer.start(chain, 0)

    newBlock()
    await processed.promise
    assert.rejects(() => db.getAccountFromNetworkRegistry(PublicKey.from_peerid_str(PARTY_B_MULTIADDR.getPeerId())))
    assert(await db.isEligible(fixtures.PARTY_A().to_address()))
  })

  it('should process register enabled', async function () {
    const { db, chain, indexer, newBlock } = await useFixtures({
      latestBlockNumber: 3,
      pastHoprRegistryEvents: [fixtures.REGISTER_ENABLED],
      id: fixtures.PARTY_A()
    })

    const processed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 3) processed.resolve()
    })
    await indexer.start(chain, 0)

    newBlock()
    await processed.promise
    assert(await db.isNetworkRegistryEnabled())
  })

  it('should process register disabled', async function () {
    const { db, chain, indexer, newBlock } = await useFixtures({
      latestBlockNumber: 3,
      pastHoprRegistryEvents: [fixtures.REGISTER_ENABLED, fixtures.REGISTER_DISABLED],
      id: fixtures.PARTY_A()
    })

    const processed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 3) processed.resolve()
    })
    await indexer.start(chain, 0)

    newBlock()
    await processed.promise
    assert((await db.isNetworkRegistryEnabled()) === false)
  })

  it('should resend queuing transactions when more native tokens are received', async function () {
    const { chain, indexer, newBlock } = await useFixtures({
      latestBlockNumber: 3,
      pastHoprRegistryEvents: [fixtures.REGISTER_ENABLED, fixtures.REGISTER_DISABLED],
      id: fixtures.PARTY_A()
    })

    let trySendTransaction: boolean = false
    chain.sendTransaction = async () => {
      trySendTransaction = true
      return {
        code: SendTransactionStatus.SUCCESS,
        tx: {
          hash: '0x123',
          confirmations: 0,
          nonce: 3,
          gasLimit: BigNumber.from('1000'),
          data: '0x',
          value: BigNumber.from('0')
        }
      }
    }

    const processed = defer<void>()
    indexer.on('block-processed', (blockNumber: number) => {
      if (blockNumber == 3) processed.resolve()
    })
    await indexer.start(chain, 0)

    newBlock()
    await processed.promise
    assert(trySendTransaction)
  })
})
