import type { Subscription } from 'web3-core-subscriptions'
import type { Log } from 'web3-core'
import type PeerId from 'peer-id'
import type { Indexer as IIndexer, RoutingChannel, ChannelUpdate } from '@hoprnet/hopr-core-connector-interface'
import type HoprEthereum from '..'
import type { Event } from './topics'
import EventEmitter from 'events'
import chalk from 'chalk'
import BN from 'bn.js'
import Heap from 'heap-js'
import { pubKeyToPeerId, randomChoice } from '@hoprnet/hopr-utils'
import { ChannelEntry, Public, Balance, Snapshot } from '../types'
import { isPartyA, getId, Log as DebugLog } from '../utils'
import * as topics from './topics'
import * as reducers from './reducers'
import {
  isConfirmedBlock,
  getLatestBlockNumber,
  getChannelEntry,
  updateChannelEntry,
  getChannelEntries,
  updateLatestBlockNumber,
  snapshotComparator,
  getLatestConfirmedSnapshot
} from './utils'

const log = DebugLog(['indexer'])
const getSyncPercentage = (n: number, max: number) => ((n * 100) / max).toFixed(2)

// @TODO: add to constants
const BLOCK_RANGE = 2000
// @TODO: get this from somewhere else
let genesisBlock: number

/**
 * Indexes HoprChannels smart contract and stores to the DB,
 * all channels in the network.
 * Also keeps track of the latest block number.
 */
class Indexer extends EventEmitter implements IIndexer {
  public status: 'started' | 'stopped' = 'stopped'
  public latestBlock: number = 0 // latest known on-chain block number
  private subscriptions: {
    [K in 'NewBlocks' | 'NewLogs']?: Subscription<any>
  } = {}
  private unconfirmedEvents = new Heap<Event<any>>(snapshotComparator)

  constructor(private connector: HoprEthereum, private maxConfirmations: number) {
    super()
    genesisBlock = getHoprChannelsBlockNumber(this.connector.chainId)
  }

  /**
   * Starts indexing.
   */
  public async start(): Promise<void> {
    if (this.status === 'started') return
    log(`Starting indexer...`)

    const { web3, hoprChannels } = this.connector

    const [latestSavedBlock, latestOnChainBlock] = await Promise.all([
      await getLatestBlockNumber(this.connector.db),
      web3.eth.getBlockNumber()
    ])
    this.latestBlock = latestSavedBlock

    log('Latest saved block %d', latestSavedBlock)
    log('Latest on-chain block %d', latestOnChainBlock)

    // go back 'MAX_CONFIRMATIONS' blocks in case of a re-org at time of stopping
    let fromBlock = latestSavedBlock
    if (fromBlock - this.maxConfirmations > 0) {
      fromBlock = fromBlock - this.maxConfirmations
    }
    // no need to query before HoprChannels existed
    if (fromBlock < genesisBlock) {
      fromBlock = genesisBlock
    }

    log(
      'Starting to index from block %d, sync progress %d%',
      fromBlock,
      getSyncPercentage(fromBlock - genesisBlock, latestOnChainBlock - genesisBlock)
    )

    // if the difference between on-chain block and {fromBlock}
    // is larger than {BLOCK_RANGE}, we will query using `getLogs` rpc
    if (fromBlock + BLOCK_RANGE <= latestOnChainBlock) {
      const toBlock = latestOnChainBlock - this.maxConfirmations
      const lastBlock = await this.processPastLogs(fromBlock, toBlock, BLOCK_RANGE)
      fromBlock = lastBlock
    }

    log('Subscribing to events from block %d', fromBlock)

    // subscribe to events
    // @TODO: handle errors
    this.subscriptions['NewBlocks'] = web3.eth
      .subscribe('newBlockHeaders')
      .on('error', console.error)
      .on('data', (block) => {
        log('New block %d', block.number)
        this.onNewBlock(block)
      })

    this.subscriptions['NewLogs'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock
      })
      .on('error', console.error)
      .on('changed', (onChainLog) => this.onChangedLogs([onChainLog]))
      .on('data', (onChainLog) => this.onNewLogs([onChainLog]))

    this.status = 'started'
    log(chalk.green('Indexer started!'))
  }

  /**
   * Stops indexing.
   */
  public async stop(): Promise<void> {
    if (this.status === 'stopped') return
    log(`Stopping indexer...`)

    for (const subscription of Object.values(this.subscriptions)) {
      subscription.unsubscribe()
    }

    this.status = 'stopped'
    log(chalk.green('Indexer stopped!'))
  }

  /**
   * Query past logs, this will loop until it gets all blocks from {toBlock} to {fromBlock}.
   * If we exceed response log limit, we switch into quering smaller chunks.
   * TODO: optimize DB and fetch requests
   * @param fromBlock
   * @param toBlock
   * @param blockRange
   * @return past logs and last queried block
   */
  private async processPastLogs(fromBlock: number, maxToBlock: number, maxBlockRange: number): Promise<number> {
    let failedCount = 0

    while (fromBlock < maxToBlock) {
      const blockRange = failedCount > 0 ? Math.floor(maxBlockRange / 4 ** failedCount) : maxBlockRange
      // should never be above maxToBlock
      let toBlock = fromBlock + blockRange
      if (toBlock > maxToBlock) toBlock = maxToBlock

      // log(
      //   `${failedCount > 0 ? 'Re-quering' : 'Quering'} past logs from %d to %d: %d`,
      //   fromBlock,
      //   toBlock,
      //   toBlock - fromBlock
      // )

      let logs: Log[] = []

      try {
        logs = await this.connector.web3.eth.getPastLogs({
          address: this.connector.hoprChannels.options.address,
          fromBlock,
          toBlock
        })
      } catch (error) {
        failedCount++

        if (failedCount > 5) {
          console.error(error)
          throw error
        }

        continue
      }

      this.onNewLogs(logs)
      await this.onNewBlock({ number: toBlock })
      failedCount = 0
      fromBlock = toBlock

      log(
        'Sync progress %d% @ block %d',
        getSyncPercentage(fromBlock - genesisBlock, maxToBlock - genesisBlock),
        toBlock
      )
    }

    return fromBlock
  }

  /**
   * Called whenever a new block found.
   * This will update {this.latestBlock},
   * and processes events which are within
   * confirmed blocks.
   * @param block
   */
  private async onNewBlock(block: { number: number }): Promise<void> {
    // update latest block
    if (this.latestBlock < block.number) {
      this.latestBlock = block.number
    }

    let lastSnapshot = await getLatestConfirmedSnapshot(this.connector.db)

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    while (
      this.unconfirmedEvents.length > 0 &&
      isConfirmedBlock(this.unconfirmedEvents.top(1)[0].blockNumber.toNumber(), block.number, this.maxConfirmations)
    ) {
      const event = this.unconfirmedEvents.pop()
      // log('Found unconfirmed event %s', event.name)
      // log(chalk.blue(event.blockNumber.toString(), event.transactionIndex.toString(), event.logIndex.toString()))

      const lastSnapshotComparison = snapshotComparator(event, lastSnapshot)

      // check if this is a duplicate, this is ok
      if (lastSnapshotComparison === 0) {
        // log(chalk.gray('Found event which is a duplicate of the last confirmed event'))
        continue
      }
      // check if event found is older, this is not ok
      else if (lastSnapshotComparison < 0) {
        log(chalk.red('Found event which is older than last confirmed event!'))
        throw new Error('Found event which is older than last confirmed event!')
      }

      if (event.name === 'FundedChannel') {
        await this.onFundedChannel(event as Event<'FundedChannel'>)
      } else if (event.name === 'OpenedChannel') {
        await this.onOpenedChannel(event as Event<'OpenedChannel'>)
      } else if (event.name === 'RedeemedTicket') {
        await this.onRedeemedTicket(event as Event<'RedeemedTicket'>)
      } else if (event.name === 'InitiatedChannelClosure') {
        await this.onInitiatedChannelClosure(event as Event<'InitiatedChannelClosure'>)
      } else if (event.name === 'ClosedChannel') {
        await this.onClosedChannel(event as Event<'ClosedChannel'>)
      }

      lastSnapshot = new Snapshot(undefined, {
        blockNumber: event.blockNumber,
        transactionIndex: event.transactionIndex,
        logIndex: event.logIndex
      })
    }

    await updateLatestBlockNumber(this.connector.db, new BN(block.number))
  }

  /**
   * Called whenever we receive new logs.
   * Converts logs to events and adds them into a heap
   * of unconfirmed events.
   * @param logs
   */
  private onNewLogs(logs: Log[]): void {
    // we dont track all events, we ignore those who return undefined
    const events = logs.reduce<Event<any>[]>((result, log) => {
      const event = topics.logToEvent(log)
      if (event) result.push(event)
      return result
    }, [])
    this.unconfirmedEvents.addAll(events)
  }

  /**
   * Called whenever we receive changed logs.
   * It removes the event generated by the log from
   * the HEAP.
   * This may happen when a log is removed due to
   * chain re-org. This is a rare case.
   * @param logs
   */
  private onChangedLogs(logs: Log[]): void {
    for (const log of logs) {
      if (!log['removed']) continue

      const event = topics.logToEvent(log)
      if (!event) continue

      this.unconfirmedEvents.remove(event, (a, b) => {
        return a.logIndex.eq(b.logIndex)
      })
    }
  }

  // on new events
  private async onFundedChannel(event: Event<'FundedChannel'>): Promise<void> {
    const recipientAccountId = await event.data.recipient.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isRecipientPartyA = isPartyA(recipientAccountId, counterpartyAccountId)
    const partyA = isRecipientPartyA ? event.data.recipient : event.data.counterparty
    const partyB = isRecipientPartyA ? event.data.counterparty : event.data.recipient

    const storedChannel = await getChannelEntry(this.connector.db, partyA, partyB)
    const channelEntry = await reducers.onFundedChannel(event, storedChannel)

    // const channelId = await getId(recipientAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    await updateChannelEntry(this.connector.db, partyA, partyB, channelEntry)

    // log('Channel %s got funded by %s', chalk.green(channelId.toHex()), chalk.green(event.data.funder))
  }

  private async onOpenedChannel(event: Event<'OpenedChannel'>): Promise<void> {
    const openerAccountId = await event.data.opener.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isOpenerPartyA = isPartyA(openerAccountId, counterpartyAccountId)
    const partyA = isOpenerPartyA ? event.data.opener : event.data.counterparty
    const partyB = isOpenerPartyA ? event.data.counterparty : event.data.opener

    const channelId = await getId(openerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await getChannelEntry(this.connector.db, partyA, partyB)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channelEntry = await reducers.onOpenedChannel(event, storedChannel)

    await updateChannelEntry(this.connector.db, partyA, partyB, channelEntry)

    this.emit('channelOpened', {
      partyA,
      partyB,
      channelEntry
    })

    // log('Channel %s got opened by %s', chalk.green(channelId.toHex()), chalk.green(openerAccountId.toHex()))
  }

  private async onRedeemedTicket(event: Event<'RedeemedTicket'>): Promise<void> {
    const redeemerAccountId = await event.data.redeemer.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isRedeemerPartyA = isPartyA(redeemerAccountId, counterpartyAccountId)
    const partyA = isRedeemerPartyA ? event.data.redeemer : event.data.counterparty
    const partyB = isRedeemerPartyA ? event.data.counterparty : event.data.redeemer

    const channelId = await getId(redeemerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await getChannelEntry(this.connector.db, partyA, partyB)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channelEntry = await reducers.onRedeemedTicket(event, storedChannel)

    await updateChannelEntry(this.connector.db, partyA, partyB, channelEntry)

    // log('Ticket redeemd in channel %s by %s', chalk.green(channelId.toHex()), chalk.green(redeemerAccountId.toHex()))
  }

  private async onInitiatedChannelClosure(event: Event<'InitiatedChannelClosure'>): Promise<void> {
    const initiatorAccountId = await event.data.initiator.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isInitiatorPartyA = isPartyA(initiatorAccountId, counterpartyAccountId)
    const partyA = isInitiatorPartyA ? event.data.initiator : event.data.counterparty
    const partyB = isInitiatorPartyA ? event.data.counterparty : event.data.initiator

    const channelId = await getId(initiatorAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await getChannelEntry(this.connector.db, partyA, partyB)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channelEntry = await reducers.onInitiatedChannelClosure(event, storedChannel)

    await updateChannelEntry(this.connector.db, partyA, partyB, channelEntry)

    // log(
    //   'Channel closure initiated for %s by %s',
    //   chalk.green(channelId.toHex()),
    //   chalk.green(initiatorAccountId.toHex())
    // )
  }

  private async onClosedChannel(event: Event<'ClosedChannel'>): Promise<void> {
    const closerAccountId = await event.data.closer.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isCloserPartyA = isPartyA(closerAccountId, counterpartyAccountId)
    const partyA = isCloserPartyA ? event.data.closer : event.data.counterparty
    const partyB = isCloserPartyA ? event.data.counterparty : event.data.closer

    const channelId = await getId(closerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await getChannelEntry(this.connector.db, partyA, partyB)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channelEntry = await reducers.onClosedChannel(event, storedChannel)

    await updateChannelEntry(this.connector.db, partyA, partyB, channelEntry)

    this.emit('channelClosed', {
      partyA,
      partyB,
      channelEntry
    })

    // log('Channel %s got closed by %s', chalk.green(channelId.toHex()), chalk.green(closerAccountId.toHex()))
  }

  // DB related
  public async getChannelEntry(partyA: Public, partyB: Public): Promise<ChannelEntry | undefined> {
    return getChannelEntry(this.connector.db, partyA, partyB)
  }

  public async getChannelEntries(party?: Public, filter?: (node: Public) => boolean): Promise<ChannelUpdate[]> {
    return getChannelEntries(this.connector.db, party, filter)
  }

  // routing related
  private async toIndexerChannel(
    source: PeerId,
    { partyA, partyB, channelEntry }: ChannelUpdate
  ): Promise<RoutingChannel> {
    const sourcePubKey = new Public(source.pubKey.marshal())
    if (sourcePubKey.eq(partyA)) {
      return [source, await pubKeyToPeerId(partyB), new Balance(channelEntry.partyABalance)]
    } else {
      const partyBBalance = new Balance(new Balance(channelEntry.deposit).sub(new Balance(channelEntry.partyABalance)))
      return [source, await pubKeyToPeerId(partyA), partyBBalance]
    }
  }

  public async getRandomChannel(): Promise<RoutingChannel | undefined> {
    const HACK = 9514000 // Arbitrarily chosen block for our testnet. Total hack.
    const results = await getChannelEntries(this.connector.db)
    const filtered = results.filter((x) => x.channelEntry.blockNumber.gtn(HACK))
    if (filtered.length === 0) {
      log('no channels exist in indexer > hack')
      return undefined
    }

    log('picking random from ', filtered.length, ' channels')
    const random = randomChoice(filtered)
    return this.toIndexerChannel(await pubKeyToPeerId(random.partyA), random)
  }

  public async getChannelsFromPeer(source: PeerId): Promise<RoutingChannel[]> {
    const sourcePubKey = new Public(source.pubKey.marshal())
    const channels = await getChannelEntries(this.connector.db, sourcePubKey)
    let cout: RoutingChannel[] = []
    for (let channel of channels) {
      let directed = await this.toIndexerChannel(source, channel)
      if (directed[2].gtn(0)) {
        cout.push(directed)
      }
    }

    return cout
  }
}

export default Indexer

// HACK
const getHoprChannelsBlockNumber = (chainId: number): number => {
  switch (chainId) {
    case 3:
      return 9503420
    case 56:
      return 2713229
    case 137:
      return 7452411
    default:
      return 0
  }
}
