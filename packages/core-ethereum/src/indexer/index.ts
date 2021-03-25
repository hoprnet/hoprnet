import type { Subscription } from 'web3-core-subscriptions'
import type PeerId from 'peer-id'
import type { Indexer as IIndexer, RoutingChannel } from '@hoprnet/hopr-core-connector-interface'
import type { ContractEventEmitter } from '../tsc/web3/types'
import type HoprEthereum from '..'
import type { Event, EventNames } from './types'
import EventEmitter from 'events'
import chalk from 'chalk'
import BN from 'bn.js'
import Heap from 'heap-js'
import { pubKeyToPeerId, randomChoice } from '@hoprnet/hopr-utils'
import { Address, ChannelEntry, Hash, Public, Balance, Snapshot } from '../types'
import { getId, Log as DebugLog } from '../utils'
import * as reducers from './reducers'
import * as db from './db'
import { isConfirmedBlock, isSyncing, snapshotComparator } from './utils'

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
  public status: 'started' | 'restarting' | 'stopped' = 'stopped'
  public latestBlock: number = 0 // latest known on-chain block number
  private publicKeys = new Map<string, Public>() // TODO: maybe we dont need this
  private newBlocksSubscription: Subscription<any>
  private newHoprChannelsEvents: ContractEventEmitter<any>
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

    // wipe indexer, do not use in production
    // await this.wipe()

    const [latestSavedBlock, latestOnChainBlock] = await Promise.all([
      await db.getLatestBlockNumber(this.connector.db),
      web3.eth.getBlockNumber()
    ])
    this.latestBlock = latestOnChainBlock

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

    // get past logs
    const lastBlock = await this.processPastEvents(fromBlock, latestOnChainBlock, BLOCK_RANGE)
    fromBlock = lastBlock

    log('Subscribing to events from block %d', fromBlock)

    // subscribe to events
    this.newBlocksSubscription = web3.eth
      .subscribe('newBlockHeaders')
      .on('error', (error) => {
        log(chalk.red(`web3 error: ${error.message}`))
        this.restart()
      })
      .on('data', (block) => {
        log('New block %d', block.number)
        this.onNewBlock(block)
      })

    this.newHoprChannelsEvents = hoprChannels.events
      .allEvents({
        fromBlock
      })
      .on('error', (error) => {
        log(chalk.red(`web3 error: ${error.message}`))
        this.restart()
      })
      .on('data', (event: Event<any>) => this.onNewEvents([event]))

    this.status = 'started'
    log(chalk.green('Indexer started!'))
  }

  /**
   * Stops indexing.
   */
  public async stop(): Promise<void> {
    if (this.status === 'stopped') return
    log(`Stopping indexer...`)

    await this.newBlocksSubscription.unsubscribe()
    this.newHoprChannelsEvents.removeAllListeners()

    this.status = 'stopped'
    log(chalk.green('Indexer stopped!'))
  }

  /**
   * @returns returns true if it's syncing
   */
  public async isSyncing(): Promise<boolean> {
    const [onChainBlock, lastKnownBlock] = await Promise.all([
      this.connector.web3.eth.getBlockNumber(),
      db.getLatestBlockNumber(this.connector.db)
    ])

    return isSyncing(onChainBlock, lastKnownBlock)
  }

  private async restart(): Promise<void> {
    if (this.status === 'restarting') return
    log('Indexer restaring')

    try {
      this.status = 'restarting'

      await this.stop()
      await this.start()
    } catch (err) {
      this.status = 'stopped'

      log(chalk.red('Failed to restart: %s', err.message))
    }
  }

  // /**
  //  * Wipes all indexer related stored data in the DB.
  //  * @deprecated do not use this in production
  //  */
  // private async wipe(): Promise<void> {
  //   await this.connector.db.batch(
  //     (await getChannelEntries(this.connector.db)).map(({ partyA, partyB }) => ({
  //       type: 'del',
  //       key: Buffer.from(this.connector.dbKeys.ChannelEntry(partyA, partyB))
  //     }))
  //   )
  //   await this.connector.db.del(Buffer.from(this.connector.dbKeys.LatestConfirmedSnapshot()))
  //   await this.connector.db.del(Buffer.from(this.connector.dbKeys.LatestBlockNumber()))

  //   log('wiped indexer data')
  // }

  /**
   * Query past events, this will loop until it gets all blocks from {toBlock} to {fromBlock}.
   * If we exceed response pull limit, we switch into quering smaller chunks.
   * TODO: optimize DB and fetch requests
   * @param fromBlock
   * @param toBlock
   * @param blockRange
   * @return past events and last queried block
   */
  private async processPastEvents(fromBlock: number, maxToBlock: number, maxBlockRange: number): Promise<number> {
    let failedCount = 0

    while (fromBlock < maxToBlock) {
      const blockRange = failedCount > 0 ? Math.floor(maxBlockRange / 4 ** failedCount) : maxBlockRange
      // should never be above maxToBlock
      let toBlock = fromBlock + blockRange
      if (toBlock > maxToBlock) toBlock = maxToBlock

      // log(
      //   `${failedCount > 0 ? 'Re-quering' : 'Quering'} past events from %d to %d: %d`,
      //   fromBlock,
      //   toBlock,
      //   toBlock - fromBlock
      // )

      let events: Event<any>[] = []

      try {
        events = await this.connector.hoprChannels.getPastEvents('allEvents', {
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

      this.onNewEvents(events)
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

    let lastSnapshot = await db.getLatestConfirmedSnapshot(this.connector.db)

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    while (
      this.unconfirmedEvents.length > 0 &&
      isConfirmedBlock(this.unconfirmedEvents.top(1)[0].blockNumber, block.number, this.maxConfirmations)
    ) {
      const event = this.unconfirmedEvents.pop()
      log('Processing event %s', event.event)
      // log(chalk.blue(event.blockNumber.toString(), event.transactionIndex.toString(), event.logIndex.toString()))

      const lastSnapshotComparison = snapshotComparator(event, {
        blockNumber: lastSnapshot.blockNumber.toNumber(),
        logIndex: lastSnapshot.logIndex.toNumber(),
        transactionIndex: lastSnapshot.transactionIndex.toNumber()
      })

      // check if this is a duplicate or older than last snapshot
      // ideally we would have detected if this snapshot was indeed processed,
      // at the moment we don't keep all events stored as we intend to keep
      // this indexer very simple
      if (lastSnapshotComparison === 0 || lastSnapshotComparison < 0) {
        continue
      }

      const eventName = event.event as EventNames

      if (eventName === 'AccountInitialized') {
        await this.onAccountInitialized(event as Event<'AccountInitialized'>)
      } else if (eventName === 'AccountSecretUpdated') {
        await this.onAccountSecretUpdated(event as Event<'AccountSecretUpdated'>)
      } else if (eventName === 'ChannelFunded') {
        await this.onChannelFunded(event as Event<'ChannelFunded'>)
      } else if (eventName === 'ChannelOpened') {
        await this.onChannelOpened(event as Event<'ChannelOpened'>)
      } else if (eventName === 'TicketRedeemed') {
        await this.onTicketRedeemed(event as Event<'TicketRedeemed'>)
      } else if (eventName === 'ChannelPendingToClose') {
        await this.onChannelPendingToClose(event as Event<'ChannelPendingToClose'>)
      } else if (eventName === 'ChannelClosed') {
        await this.onChannelClosed(event as Event<'ChannelClosed'>)
      }

      lastSnapshot = new Snapshot(undefined, {
        blockNumber: new BN(event.blockNumber),
        transactionIndex: new BN(event.transactionIndex),
        logIndex: new BN(event.logIndex)
      })
    }

    await db.updateLatestBlockNumber(this.connector.db, new BN(block.number))
  }

  /**
   * Called whenever we receive new events.
   * @param events
   */
  private onNewEvents(events: Event<any>[]): void {
    this.unconfirmedEvents.addAll(events)
  }

  // on new events
  private async onAccountInitialized(event: Event<'AccountInitialized'>): Promise<void> {
    const accountId = Address.fromString(event.returnValues.account)
    const account = await reducers.onAccountInitialized(event)

    await db.updateAccount(this.connector.db, accountId, account)
  }

  private async onAccountSecretUpdated(event: Event<'AccountSecretUpdated'>): Promise<void> {
    const data = event.returnValues

    const accountId = Address.fromString(data.account)

    const storedAccount = await db.getAccount(this.connector.db, accountId)
    if (!storedAccount) {
      log(chalk.red('Could not find stored account %s !'), accountId.toHex())
      return
    }

    const account = await reducers.onAccountSecretUpdated(event, storedAccount)

    await db.updateAccount(this.connector.db, accountId, account)
  }

  private async onChannelFunded(event: Event<'ChannelFunded'>): Promise<void> {
    const data = event.returnValues

    const accountIdA = Address.fromString(data.accountA)
    const accountIdB = Address.fromString(data.accountB)
    const channelId = await getId(accountIdA, accountIdB)

    let storedChannel = await db.getChannel(this.connector.db, channelId)
    const channel = await reducers.onChannelFunded(event, storedChannel)

    // const channelId = await getId(recipientAddress, counterpartyAddress)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    await db.updateChannel(this.connector.db, channelId, channel)

    // log('Channel %s got funded by %s', chalk.green(channelId.toHex()), chalk.green(event.data.funder))
  }

  private async onChannelOpened(event: Event<'ChannelOpened'>): Promise<void> {
    const data = event.returnValues

    const openerAccountId = Address.fromString(data.opener)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = await getId(openerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    let storedChannel = await db.getChannel(this.connector.db, channelId)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channel = await reducers.onChannelOpened(event, storedChannel)

    await db.updateChannel(this.connector.db, channelId, channel)

    this.emit('channelOpened', {
      channelId,
      channel
    })

    // log('Channel %s got opened by %s', chalk.green(channelId.toHex()), chalk.green(openerAddress.toHex()))
  }

  private async onTicketRedeemed(event: Event<'TicketRedeemed'>): Promise<void> {
    const data = event.returnValues

    const redeemerAccountId = Address.fromString(data.redeemer)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = await getId(redeemerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await db.getChannel(this.connector.db, channelId)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channel = await reducers.onTicketRedeemed(event, storedChannel)

    await db.updateChannel(this.connector.db, channelId, channel)

    // log('Ticket redeemd in channel %s by %s', chalk.green(channelId.toHex()), chalk.green(redeemerAddress.toHex()))
  }

  private async onChannelPendingToClose(event: Event<'ChannelPendingToClose'>): Promise<void> {
    const data = event.returnValues

    const initiatorAccountId = Address.fromString(data.initiator)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = await getId(initiatorAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await db.getChannel(this.connector.db, channelId)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channel = await reducers.onChannelPendingToClose(event, storedChannel)

    await db.updateChannel(this.connector.db, channelId, channel)

    // log(
    //   'Channel closure initiated for %s by %s',
    //   chalk.green(channelId.toHex()),
    //   chalk.green(initiatorAddress.toHex())
    // )
  }

  private async onChannelClosed(event: Event<'ChannelClosed'>): Promise<void> {
    const data = event.returnValues

    const closerAccountId = Address.fromString(data.initiator)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = await getId(closerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await db.getChannel(this.connector.db, channelId)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel %s !'), channelId.toHex())
      return
    }

    const channel = await reducers.onChannelClosed(event, storedChannel)

    await db.updateChannel(this.connector.db, channelId, channel)

    this.emit('channelClosed', {
      channelId,
      channel
    })

    // log('Channel %s got closed by %s', chalk.green(channelId.toHex()), chalk.green(closerAddress.toHex()))
  }

  public async getAccount(address: Address): Promise<Account | undefined> {
    return db.getAccount(this.connector.db, address)
  }

  public async getChannel(channelId: Hash): Promise<ChannelEntry | undefined> {
    return db.getChannel(this.connector.db, channelId)
  }

  public async getChannels(filter?: (channel: ChannelEntry) => Promise<boolean>): Promise<ChannelEntry[]> {
    return db.getChannels(this.connector.db, filter)
  }

  public async getChannelsOf(address: Address): Promise<ChannelEntry[]> {
    return db.getChannels(this.connector.db, async (channel) => {
      const [accountA, accountB] = channel.parties
      return address.eq(accountA) || address.eq(accountB)
    })
  }

  // routing
  public async getPublicKeyOf(address: Address): Promise<Public | undefined> {
    if (this.publicKeys.has(address.toHex())) {
      return this.publicKeys.get(address.toHex())
    }

    const account = await db.getAccount(this.connector.db, address)
    if (account && account.publicKey) {
      this.publicKeys.set(address.toHex(), account.publicKey)
      return account.publicKey
    }

    return undefined
  }

  private async toIndexerChannel(source: PeerId, channel: ChannelEntry): Promise<RoutingChannel> {
    const sourcePubKey = new Public(source.pubKey.marshal())
    const [accountAPubKey, accountBPubKey] = await Promise.all(
      channel.parties.map((address) => this.getPublicKeyOf(address))
    )

    if (sourcePubKey.eq(accountAPubKey)) {
      return [source, await pubKeyToPeerId(accountBPubKey), new Balance(channel.partyABalance)]
    } else {
      const partyBBalance = new Balance(new Balance(channel.deposit).sub(new Balance(channel.partyABalance)))
      return [source, await pubKeyToPeerId(accountAPubKey), partyBBalance]
    }
  }

  public async getRandomChannel(): Promise<RoutingChannel | undefined> {
    const HACK = 14744510 // Arbitrarily chosen block for our testnet. Total hack.

    const channels = await this.getChannels(async (channel) => {
      // filter out channels older than hack
      if (!channel.openedAt.gtn(HACK)) return false
      // filter out channels with uninitialized parties
      const pubKeys = await Promise.all(channel.parties.map((address) => this.getPublicKeyOf(address)))
      return pubKeys.every((pubKeys) => pubKeys)
    })

    if (channels.length === 0) {
      log('no channels exist in indexer > hack')
      return undefined
    }

    log('picking random from %d channels', channels.length)
    const random = randomChoice(channels)
    const accountA = await this.getPublicKeyOf(random.parties[0])
    return this.toIndexerChannel(await pubKeyToPeerId(accountA), random) // TODO: find why we do this
  }

  public async getChannelsFromPeer(source: PeerId): Promise<RoutingChannel[]> {
    const sourcePubKey = new Public(source.pubKey.marshal())
    const channels = await this.getChannelsOf(await sourcePubKey.toAddress())

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

// HACK, get the genesis block number
// of HoprChannels for each chain
// TODO: get this data from `ethereum` package
const getHoprChannelsBlockNumber = (chainId: number): number => {
  switch (chainId) {
    case 3:
      return 9547931
    case 5:
      return 4260231
    case 56:
      return 2713229
    case 100:
      return 14744510
    case 137:
      return 7452411
    default:
      return 0
  }
}
