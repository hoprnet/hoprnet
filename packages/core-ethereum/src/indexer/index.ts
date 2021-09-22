import BN from 'bn.js'
import Debug from 'debug'
import Heap from 'heap-js'
import PeerId from 'peer-id'
import chalk from 'chalk'
import { EventEmitter } from 'events'
import { Multiaddr } from 'multiaddr'
import {
  randomChoice,
  HoprDB,
  stringToU8a,
  ChannelStatus,
  Address,
  ChannelEntry,
  Defer,
  AccountEntry,
  Hash,
  PublicKey,
  Snapshot,
  u8aConcat
} from '@hoprnet/hopr-utils'

import type { ChainWrapper } from '../ethereum'
import type { Event, EventNames } from './types'
import { Commitment } from '../commitment'
import { isConfirmedBlock, snapshotComparator } from './utils'

const log = Debug('hopr-core-ethereum:indexer')
const getSyncPercentage = (n: number, max: number) => ((n * 100) / max).toFixed(2)
const ANNOUNCEMENT = 'Announcement'

/**
 * Indexes HoprChannels smart contract and stores to the DB,
 * all channels in the network.
 * Also keeps track of the latest block number.
 */
class Indexer extends EventEmitter {
  public status: 'started' | 'restarting' | 'stopped' = 'stopped'
  public latestBlock: number = 0 // latest known on-chain block number
  private unconfirmedEvents = new Heap<Event<any>>(snapshotComparator)
  private pendingCommitments: Map<string, Defer<void>>
  private chain: ChainWrapper
  private genesisBlock: number

  constructor(
    private address: Address,
    private db: HoprDB,
    private maxConfirmations: number,
    private blockRange: number
  ) {
    super()
    this.pendingCommitments = new Map<string, Defer<void>>()
  }

  /**
   * Starts indexing.
   */
  public async start(chain: ChainWrapper, genesisBlock: number): Promise<void> {
    if (this.status === 'started') return
    log(`Starting indexer...`)
    this.chain = chain
    this.genesisBlock = genesisBlock

    const [latestSavedBlock, latestOnChainBlock] = await Promise.all([
      await this.db.getLatestBlockNumber(),
      this.chain.getLatestBlockNumber()
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
    if (fromBlock < this.genesisBlock) {
      fromBlock = this.genesisBlock
    }

    log(
      'Starting to index from block %d, sync progress %d%',
      fromBlock,
      getSyncPercentage(fromBlock - this.genesisBlock, latestOnChainBlock - this.genesisBlock)
    )

    this.chain.subscribeBlock((b) => {
      this.onNewBlock(b)
    })
    this.chain.subscribeError((error: any) => {
      log(chalk.red(`etherjs error: ${error}`))
      this.restart()
    })
    this.chain.subscribeChannelEvents((e) => {
      this.onNewEvents([e])
    })

    // get past events
    const lastBlock = await this.processPastEvents(fromBlock, latestOnChainBlock, this.blockRange)
    fromBlock = lastBlock

    log('Subscribing to events from block %d', fromBlock)

    this.status = 'started'
    this.emit('status', 'started')
    log(chalk.green('Indexer started!'))
  }

  /**
   * Stops indexing.
   */
  public async stop(): Promise<void> {
    if (this.status === 'stopped') return
    log(`Stopping indexer...`)

    this.chain.unsubscribe()

    this.status = 'stopped'
    this.emit('status', 'stopped')
    log(chalk.green('Indexer stopped!'))
  }

  private async restart(): Promise<void> {
    if (this.status === 'restarting') return
    log('Indexer restaring')

    try {
      this.status = 'restarting'

      await this.stop()
      await this.start(this.chain, this.genesisBlock)
    } catch (err) {
      this.status = 'stopped'
      this.emit('status', 'stopped')

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
        events = await this.chain.getChannels().queryFilter('*' as any, fromBlock, toBlock)
      } catch (error) {
        failedCount++

        if (failedCount > 5) {
          throw error
        }

        continue
      }

      this.onNewEvents(events)
      await this.onNewBlock(toBlock)
      failedCount = 0
      fromBlock = toBlock

      log(
        'Sync progress %d% @ block %d',
        getSyncPercentage(fromBlock - this.genesisBlock, maxToBlock - this.genesisBlock),
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
  private async onNewBlock(blockNumber: number): Promise<void> {
    log('Indexer got new block %d', blockNumber)
    this.emit('block', blockNumber)

    // update latest block
    if (this.latestBlock < blockNumber) {
      this.latestBlock = blockNumber
    }

    let lastSnapshot = await this.db.getLatestConfirmedSnapshotOrUndefined()
    const confirmedEvents = []

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    while (
      this.unconfirmedEvents.length > 0 &&
      isConfirmedBlock(this.unconfirmedEvents.top(1)[0].blockNumber, blockNumber, this.maxConfirmations)
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
      confirmedEvents.push(event)
    }

    // Sort announcements first, so we have a record of address => publickeys
    // when processing other updates.
    confirmedEvents.sort((a, b) => {
      if (a.event === ANNOUNCEMENT) {
        return b.event === ANNOUNCEMENT ? 0 : -1
      }
      return b.event === ANNOUNCEMENT ? 1 : 0
    })

    for (const event of confirmedEvents) {
      const eventName = event.event as EventNames

      try {
        if (eventName === ANNOUNCEMENT) {
          await this.onAnnouncement(event as Event<'Announcement'>, new BN(blockNumber.toPrecision()))
        } else if (eventName === 'ChannelUpdated') {
          await this.onChannelUpdated(event as Event<'ChannelUpdated'>)
        } else {
          log(`ignoring event '${eventName}'`)
        }
      } catch (err) {
        log('error processing event:', event, err)
      }

      lastSnapshot = new Snapshot(new BN(event.blockNumber), new BN(event.transactionIndex), new BN(event.logIndex))
      await this.db.updateLatestConfirmedSnapshot(lastSnapshot)
    }

    await this.db.updateLatestBlockNumber(new BN(blockNumber))
    this.emit('block-processed', blockNumber)
  }

  /**
   * Called whenever we receive new events.
   * @param events
   */
  private onNewEvents(events: Event<any>[]): void {
    this.unconfirmedEvents.addAll(events)
  }

  private async onAnnouncement(event: Event<'Announcement'>, blockNumber: BN): Promise<void> {
    // publicKey given by the SC is verified
    const publicKey = PublicKey.fromUncompressedPubKey(
      // add uncompressed key identifier
      u8aConcat(new Uint8Array([4]), stringToU8a(event.args.publicKey))
    )
    const multiaddr = new Multiaddr(stringToU8a(event.args.multiaddr))
      // remove "p2p" and corresponding peerID
      .decapsulateCode(421)
      // add new peerID
      .encapsulate(`/p2p/${publicKey.toPeerId().toB58String()}`)
    const address = Address.fromString(event.args.account)
    const account = new AccountEntry(address, multiaddr, blockNumber)
    if (!account.getPublicKey().toAddress().eq(address)) {
      throw Error("Multiaddr in announcement does not match sender's address")
    }
    if (!account.getPeerId() || !PeerId.isPeerId(account.getPeerId())) {
      throw Error('Peer ID in multiaddr is null')
    }
    log('New node announced', account.address.toHex(), account.multiAddr.toString())
    this.emit('peer', {
      id: account.getPeerId(),
      multiaddrs: [account.multiAddr]
    })
    await this.db.updateAccount(account)
  }

  private async onChannelUpdated(event: Event<'ChannelUpdated'>): Promise<void> {
    const channel = await ChannelEntry.fromSCEvent(event, (a: Address) => this.getPublicKeyOf(a))

    log(channel.toString())
    await this.db.updateChannel(channel.getId(), channel)
    this.emit('channel-update', channel)

    if (channel.source.toAddress().eq(this.address) || channel.destination.toAddress().eq(this.address)) {
      this.emit('own-channel-updated', channel)

      if (channel.destination.toAddress().eq(this.address)) {
        // Channel _to_ us
        if (channel.status === ChannelStatus.WaitingForCommitment) {
          await this.onOwnUnsetCommitment(channel)
        } else if (channel.status === ChannelStatus.Open) {
          this.resolveCommitmentPromise(channel.getId())
        }
      }
    }
  }

  private onOwnUnsetCommitment(channel: ChannelEntry) {
    if (!channel.destination.toAddress().eq(this.address)) {
      throw new Error('shouldnt be called unless we are the destination')
    }
    log(`Found channel ${chalk.yellow(channel.getId().toHex())} to us with unset commitment. Setting commitment`)

    const setCommitment = (commitment: Hash): Promise<string> => {
      try {
        return this.chain.setCommitment(channel.source.toAddress(), commitment)
      } catch (e) {
        log('Error setting commitment', e)
        // TODO: defer to channel strategy for this, and allow for retries.
      }
    }

    return new Commitment(
      setCommitment,
      async () => (await this.getChannel(channel.getId())).commitment,
      this.db,
      channel.getId(),
      this
    ).initialize()
  }

  public waitForCommitment(channelId: Hash): Promise<void> {
    let waiting = this.pendingCommitments.get(channelId.toHex())

    if (waiting != undefined) {
      return waiting.promise
    }

    waiting = new Defer()

    this.pendingCommitments.set(channelId.toHex(), waiting)
  }

  private resolveCommitmentPromise(channelId: Hash) {
    this.pendingCommitments.get(channelId.toHex())?.resolve()
  }

  public async getAccount(address: Address) {
    return this.db.getAccount(address)
  }

  public async getChannel(channelId: Hash) {
    return this.db.getChannel(channelId)
  }

  public async getChannels(filter?: (channel: ChannelEntry) => boolean) {
    return this.db.getChannels(filter)
  }

  public async getChannelsFrom(address: Address) {
    return this.db.getChannels((channel) => {
      return address.eq(channel.source.toAddress())
    })
  }

  public async getChannelsTo(address: Address) {
    return this.db.getChannels((channel) => {
      return address.eq(channel.destination.toAddress())
    })
  }

  public async getPublicKeyOf(address: Address): Promise<PublicKey> {
    const account = await this.db.getAccount(address)
    if (account) {
      return account.getPublicKey()
    }
    throw new Error('Could not find public key for address - have they announced? -' + address.toHex())
  }

  public async getAnnouncedAddresses(): Promise<Multiaddr[]> {
    return (await this.db.getAccounts()).map((account: AccountEntry) => account.multiAddr)
  }

  public async getPublicNodes(): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }[]> {
    return (await this.db.getAccounts((account: AccountEntry) => account.containsRouting())).map(
      (account: AccountEntry) => ({
        id: account.getPublicKey().toPeerId(),
        multiaddrs: [account.multiAddr]
      })
    )
  }

  /**
   * Returns a random open channel.
   * NOTE: channels with status 'PENDING_TO_CLOSE' are not included
   * @returns an open channel
   */
  public async getRandomOpenChannel(): Promise<ChannelEntry> {
    const channels = await this.getChannels((channel) => channel.status === ChannelStatus.Open)

    if (channels.length === 0) {
      log('no open channels exist in indexer')
      return undefined
    }

    return randomChoice(channels)
  }

  /**
   * Returns peer's open channels.
   * NOTE: channels with status 'PENDING_TO_CLOSE' are not included
   * @param source peer
   * @returns peer's open channels
   */
  public async getOpenChannelsFrom(source: PublicKey): Promise<ChannelEntry[]> {
    return await this.getChannelsFrom(source.toAddress()).then((channels) =>
      channels.filter((channel) => channel.status === ChannelStatus.Open)
    )
  }
}

export default Indexer
