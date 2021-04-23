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
import * as db from './db'
import { isConfirmedBlock, snapshotComparator } from './utils'
import Debug from 'debug'

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

      if (eventName != 'ChannelUpdate') {
        throw new Error('bad event name')
      }

      await this.onChannelUpdated(event as Event<'ChannelUpdate'>)

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

  private async onChannelUpdated(event: Event<'ChannelUpdate'>): Promise<void> {
    const channel = ChannelEntry.fromSCEvent(event)
    await db.updateChannel(this.connector.db, channel.getId(), channel)
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
      return [source, await pubKeyToPeerId(partyBPubKey.serialize()), channel.partyABalance]
    } else {
      const partyBBalance = channel.partyBBalance
      return [source, await pubKeyToPeerId(partyAPubKey.serialize()), partyBBalance]
    }
  }

  public async getRandomChannel() {
    const channels = await this.getChannels(async (channel) => {
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
