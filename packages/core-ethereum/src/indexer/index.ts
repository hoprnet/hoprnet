import type PeerId from 'peer-id'
import type { LevelUp } from 'levelup'
import type { providers as Providers } from 'ethers'
import type { Event, EventNames } from './types'
import type { HoprChannels } from '../contracts'
import EventEmitter from 'events'
import chalk from 'chalk'
import BN from 'bn.js'
import Heap from 'heap-js'
import { pubKeyToPeerId, randomChoice } from '@hoprnet/hopr-utils'
import { Address, ChannelEntry, Hash, PublicKey, Balance, Snapshot } from '../types'
import * as reducers from './reducers'
import * as db from './db'
import { isConfirmedBlock, isSyncing, snapshotComparator } from './utils'
import Debug from 'debug'
import { Channel } from '..'

export type RoutingChannel = [source: PeerId, destination: PeerId, stake: Balance]

const log = Debug('hopr-core-ethereum:indexer')
const getSyncPercentage = (n: number, max: number) => ((n * 100) / max).toFixed(2)

/**
 * Indexes HoprChannels smart contract and stores to the DB,
 * all channels in the network.
 * Also keeps track of the latest block number.
 */
class Indexer extends EventEmitter {
  public status: 'started' | 'restarting' | 'stopped' = 'stopped'
  public latestBlock: number = 0 // latest known on-chain block number
  private unconfirmedEvents = new Heap<Event<any>>(snapshotComparator)

  constructor(
    private ops: {
      genesisBlock: number
      maxConfirmations: number
      blockRange: number
    },
    private api: {
      db: LevelUp
      provider: Providers.WebSocketProvider
      hoprChannels: HoprChannels
    }
  ) {
    super()
  }

  /**
   * Starts indexing.
   */
  public async start(): Promise<void> {
    if (this.status === 'started') return
    log(`Starting indexer...`)

    // wipe indexer, do not use in production
    // await this.wipe()

    const [latestSavedBlock, latestOnChainBlock] = await Promise.all([
      db.getLatestBlockNumber(this.api.db),
      this.api.provider.getBlockNumber()
    ])
    this.latestBlock = latestOnChainBlock

    log('Latest saved block %d', latestSavedBlock)
    log('Latest on-chain block %d', latestOnChainBlock)

    // go back 'MAX_CONFIRMATIONS' blocks in case of a re-org at time of stopping
    let fromBlock = latestSavedBlock
    if (fromBlock - this.ops.maxConfirmations > 0) {
      fromBlock = fromBlock - this.ops.maxConfirmations
    }
    // no need to query before HoprChannels existed
    if (fromBlock < this.ops.genesisBlock) {
      fromBlock = this.ops.genesisBlock
    }

    log(
      'Starting to index from block %d, sync progress %d%',
      fromBlock,
      getSyncPercentage(fromBlock - this.ops.genesisBlock, latestOnChainBlock - this.ops.genesisBlock)
    )

    // get past events
    const lastBlock = await this.processPastEvents(fromBlock, latestOnChainBlock, this.ops.blockRange)
    fromBlock = lastBlock

    log('Subscribing to events from block %d', fromBlock)

    // subscribe to new blocks
    this.api.provider
      .on('block', (blockNumber: number) => {
        this.onNewBlock({ number: blockNumber })
      })
      .on('error', (error: any) => {
        log(chalk.red(`etherjs error: ${error}`))
        this.restart()
      })

    // subscribe to all HoprChannels events
    this.api.hoprChannels
      .on('*', (event: Event<any>) => {
        this.onNewEvents([event])
      })
      .on('error', (error: any) => {
        log(chalk.red(`etherjs error: ${error}`))
        this.restart()
      })

    this.status = 'started'
    log(chalk.green('Indexer started!'))
  }

  /**
   * Stops indexing.
   */
  public async stop(): Promise<void> {
    if (this.status === 'stopped') return
    log(`Stopping indexer...`)

    this.api.provider.removeAllListeners()
    this.api.hoprChannels.removeAllListeners()

    this.status = 'stopped'
    log(chalk.green('Indexer stopped!'))
  }

  /**
   * @returns returns true if it's syncing
   */
  public async isSyncing(): Promise<boolean> {
    const [onChainBlock, lastKnownBlock] = await Promise.all([
      this.api.provider.getBlockNumber(),
      db.getLatestBlockNumber(this.api.db)
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
  //   await this.api.db.batch(
  //     (await getChannelEntries(this.api.db)).map(({ partyA, partyB }) => ({
  //       type: 'del',
  //       key: Buffer.from(this.api.dbKeys.ChannelEntry(partyA, partyB))
  //     }))
  //   )
  //   await this.api.db.del(Buffer.from(this.api.dbKeys.LatestConfirmedSnapshot()))
  //   await this.api.db.del(Buffer.from(this.api.dbKeys.LatestBlockNumber()))

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
        // TODO: wildcard is supported but not properly typed
        events = await this.api.hoprChannels.queryFilter('*' as any, fromBlock, toBlock)
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
        getSyncPercentage(fromBlock - this.ops.genesisBlock, maxToBlock - this.ops.genesisBlock),
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

    let lastSnapshot = await db.getLatestConfirmedSnapshot(this.api.db)

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    while (
      this.unconfirmedEvents.length > 0 &&
      isConfirmedBlock(this.unconfirmedEvents.top(1)[0].blockNumber, block.number, this.ops.maxConfirmations)
    ) {
      const event = this.unconfirmedEvents.pop()
      log('Processing event %s', event.event)
      // log(chalk.blue(event.blockNumber.toString(), event.transactionIndex.toString(), event.logIndex.toString()))

      // if we find a previous snapshot, compare event's snapshot with last processed
      if (lastSnapshot) {
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
      }

      const eventName = event.event as EventNames

      try {
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
      } catch (err) {
        log(chalk.red('Error while reducing event'))
        throw err
      }

      lastSnapshot = new Snapshot(new BN(event.blockNumber), new BN(event.transactionIndex), new BN(event.logIndex))
      await db.updateLatestConfirmedSnapshot(this.api.db, lastSnapshot)
    }

    await db.updateLatestBlockNumber(this.api.db, new BN(block.number))
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
    const accountId = Address.fromString(event.args.account)
    const account = await reducers.onAccountInitialized(event)

    await db.updateAccount(this.api.db, accountId, account)
  }

  private async onAccountSecretUpdated(event: Event<'AccountSecretUpdated'>): Promise<void> {
    const data = event.args

    const accountId = Address.fromString(data.account)

    const storedAccount = await db.getAccount(this.api.db, accountId)
    if (!storedAccount) throw Error(`Could not find stored account ${accountId.toHex()} !`)

    const account = await reducers.onAccountSecretUpdated(event, storedAccount)

    await db.updateAccount(this.api.db, accountId, account)
  }

  private async onChannelFunded(event: Event<'ChannelFunded'>): Promise<void> {
    const data = event.args

    const accountIdA = Address.fromString(data.accountA)
    const accountIdB = Address.fromString(data.accountB)
    const channelId = Channel.generateId(accountIdA, accountIdB)

    let storedChannel = await db.getChannel(this.api.db, channelId)
    const channel = await reducers.onChannelFunded(event, storedChannel)

    // const channelId = await getId(recipientAddress, counterpartyAddress)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    await db.updateChannel(this.api.db, channelId, channel)

    // log('Channel %s got funded by %s', chalk.green(channelId.toHex()), chalk.green(event.data.funder))
  }

  private async onChannelOpened(event: Event<'ChannelOpened'>): Promise<void> {
    const data = event.args

    const openerAccountId = Address.fromString(data.opener)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = Channel.generateId(openerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    let storedChannel = await db.getChannel(this.api.db, channelId)
    if (!storedChannel) throw Error(`Could not find stored channel ${channelId.toHex()}`)

    const channel = await reducers.onChannelOpened(event, storedChannel)

    await db.updateChannel(this.api.db, channelId, channel)

    this.emit('channelOpened', channel)

    // log('Channel %s got opened by %s', chalk.green(channelId.toHex()), chalk.green(openerAddress.toHex()))
  }

  private async onTicketRedeemed(event: Event<'TicketRedeemed'>): Promise<void> {
    const data = event.args

    const redeemerAccountId = Address.fromString(data.redeemer)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = Channel.generateId(redeemerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await db.getChannel(this.api.db, channelId)
    if (!storedChannel) throw Error(`Could not find stored channel ${channelId.toHex()}`)

    const channel = await reducers.onTicketRedeemed(event, storedChannel)

    await db.updateChannel(this.api.db, channelId, channel)

    // log('Ticket redeemd in channel %s by %s', chalk.green(channelId.toHex()), chalk.green(redeemerAddress.toHex()))
  }

  private async onChannelPendingToClose(event: Event<'ChannelPendingToClose'>): Promise<void> {
    const data = event.args

    const initiatorAccountId = Address.fromString(data.initiator)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = Channel.generateId(initiatorAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await db.getChannel(this.api.db, channelId)
    if (!storedChannel) throw Error(`Could not find stored channel ${channelId.toHex()}`)

    const channel = await reducers.onChannelPendingToClose(event, storedChannel)

    await db.updateChannel(this.api.db, channelId, channel)

    // log(
    //   'Channel closure initiated for %s by %s',
    //   chalk.green(channelId.toHex()),
    //   chalk.green(initiatorAddress.toHex())
    // )
  }

  private async onChannelClosed(event: Event<'ChannelClosed'>): Promise<void> {
    const data = event.args

    const closerAccountId = Address.fromString(data.initiator)
    const counterpartyAccountId = Address.fromString(data.counterparty)
    const channelId = Channel.generateId(closerAccountId, counterpartyAccountId)
    // log('Processing event %s with channelId %s', event.name, channelId.toHex())

    const storedChannel = await db.getChannel(this.api.db, channelId)
    if (!storedChannel) throw Error(`Could not find stored channel ${channelId.toHex()}`)

    const channel = await reducers.onChannelClosed(event, storedChannel)

    await db.updateChannel(this.api.db, channelId, channel)

    this.emit('channelClosed', channel)

    // log('Channel %s got closed by %s', chalk.green(channelId.toHex()), chalk.green(closerAddress.toHex()))
  }

  public async getAccount(address: Address) {
    return db.getAccount(this.api.db, address)
  }

  public async getChannel(channelId: Hash) {
    return db.getChannel(this.api.db, channelId)
  }

  public async getChannels(filter?: (channel: ChannelEntry) => Promise<boolean>) {
    return db.getChannels(this.api.db, filter)
  }

  public async getChannelsOf(address: Address) {
    return db.getChannels(this.api.db, async (channel) => {
      return address.eq(channel.partyA) || address.eq(channel.partyB)
    })
  }

  // routing
  public async getPublicKeyOf(address: Address): Promise<PublicKey | undefined> {
    const account = await db.getAccount(this.api.db, address)
    if (account && account.publicKey) {
      return account.publicKey
    }

    return undefined
  }

  private async toIndexerChannel(source: PeerId, channel: ChannelEntry): Promise<RoutingChannel> {
    const sourcePubKey = new PublicKey(source.pubKey.marshal())
    const [partyAPubKey, partyBPubKey] = await Promise.all([
      this.getPublicKeyOf(channel.partyA),
      this.getPublicKeyOf(channel.partyB)
    ])

    if (sourcePubKey.eq(partyAPubKey)) {
      return [source, await pubKeyToPeerId(partyBPubKey.serialize()), new Balance(channel.partyABalance)]
    } else {
      const partyBBalance = new Balance(
        new Balance(channel.deposit).toBN().sub(new Balance(channel.partyABalance).toBN())
      )
      return [source, await pubKeyToPeerId(partyAPubKey.serialize()), partyBBalance]
    }
  }

  public async getRandomChannel() {
    const HACK = 14744510 // Arbitrarily chosen block for our testnet. Total hack.

    const channels = await this.getChannels(async (channel) => {
      // filter out channels older than hack
      if (!channel.openedAt.gtn(HACK)) return false
      // filter out channels with uninitialized parties
      const pubKeys = await Promise.all([this.getPublicKeyOf(channel.partyA), this.getPublicKeyOf(channel.partyB)])
      return pubKeys.every((pubKeys) => pubKeys)
    })

    if (channels.length === 0) {
      log('no channels exist in indexer > hack')
      return undefined
    }

    log('picking random from %d channels', channels.length)
    const random = randomChoice(channels)
    const partyA = await this.getPublicKeyOf(random.partyA)
    return this.toIndexerChannel(await pubKeyToPeerId(partyA.serialize()), random) // TODO: why do we pick partyA?
  }

  public async getChannelsFromPeer(source: PeerId) {
    const sourcePubKey = new PublicKey(source.pubKey.marshal())
    const channels = await this.getChannelsOf(await sourcePubKey.toAddress())

    let cout: RoutingChannel[] = []
    for (let channel of channels) {
      let directed = await this.toIndexerChannel(source, channel)
      if (directed[2].toBN().gtn(0)) {
        cout.push(directed)
      }
    }

    return cout
  }
}

export default Indexer
