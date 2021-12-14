import BN from 'bn.js'
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
  AccountEntry,
  PublicKey,
  Snapshot,
  u8aConcat,
  debug,
  DeferType,
  retryWithBackoff
} from '@hoprnet/hopr-utils'

import type { ChainWrapper } from '../ethereum'
import type { Event, EventNames, IndexerEvents, TokenEvent, TokenEventNames } from './types'
import { isConfirmedBlock, snapshotComparator } from './utils'
import { errors, utils } from 'ethers'
import { INDEXER_TIMEOUT, MAX_TRANSACTION_BACKOFF } from '../constants'

const log = debug('hopr-core-ethereum:indexer')
const getSyncPercentage = (n: number, max: number) => ((n * 100) / max).toFixed(2)
const ANNOUNCEMENT = 'Announcement'
const backoffOption: Parameters<typeof retryWithBackoff>[1] = { maxDelay: MAX_TRANSACTION_BACKOFF }

/**
 * Indexes HoprChannels smart contract and stores to the DB,
 * all channels in the network.
 * Also keeps track of the latest block number.
 */
class Indexer extends EventEmitter {
  public status: 'started' | 'restarting' | 'stopped' = 'stopped'
  public latestBlock: number = 0 // latest known on-chain block number
  private unconfirmedEvents = new Heap<Event<any> | TokenEvent<any>>(snapshotComparator)
  private chain: ChainWrapper
  private genesisBlock: number

  constructor(
    private address: Address,
    private db: HoprDB,
    private maxConfirmations: number,
    private blockRange: number
  ) {
    super()
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
      this.db.getLatestBlockNumber(),
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

    this.chain.subscribeBlock(async (b) => {
      await this.onNewBlock(b) // exceptions are handled
    })

    this.chain.subscribeError(async (error: any) => {
      await this.onProviderError(error) // exceptions are handled
    })

    this.chain.subscribeChannelEvents((e) => {
      if (e.event === ANNOUNCEMENT || e.event === 'ChannelUpdated') {
        this.onNewEvents([e])
      }
    })
    this.chain.subscribeTokenEvents((e) => {
      if (
        e.event === 'Transfer' &&
        (e.topics[1] === utils.hexZeroPad(this.address.toHex(), 32) ||
          e.topics[2] === utils.hexZeroPad(this.address.toHex(), 32))
      ) {
        // save transfer events
        this.onNewEvents([e])
      }
    })

    // get past events
    fromBlock = await this.processPastEvents(fromBlock, latestOnChainBlock, this.blockRange)

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
      throw err
    }
  }

  /**
   * Query past events, this will loop until it gets all blocks from {toBlock} to {fromBlock}.
   * If we exceed response pull limit, we switch into quering smaller chunks.
   * TODO: optimize DB and fetch requests
   * @param fromBlock
   * @param maxToBlock
   * @param maxBlockRange
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
   * Called whenever there was a provider error.
   * Will restart the indexer if needed.
   * @param error
   * @private
   */
  private async onProviderError(error: any): Promise<void> {
    log(chalk.red(`etherjs error: ${error}`))

    try {
      // if provider connection issue
      if (
        [errors.SERVER_ERROR, errors.TIMEOUT, 'ECONNRESET', 'ECONNREFUSED'].some((err) =>
          [error?.code, String(error)].includes(err)
        )
      ) {
        log(chalk.blue('code error falls here', this.chain.getAllQueuingTransactionRequests().length))
        if (this.chain.getAllQueuingTransactionRequests().length > 0) {
          const wallet = this.chain.getWallet()
          await retryWithBackoff(
            () =>
              Promise.allSettled([
                ...this.chain.getAllQueuingTransactionRequests().map((request) => wallet.sendTransaction(request)),
                this.restart()
              ]),
            backoffOption
          )
        }
      } else {
        await retryWithBackoff(() => this.restart(), backoffOption)
      }
    }
    catch (err) {
      log(`error: exception while processing another provider error ${error}`, err)
    }
  }

  /**
   * Called whenever a new block found.
   * This will update {this.latestBlock},
   * and processes events which are within
   * confirmed blocks.
   * @param blockNumber
   */
  private async onNewBlock(blockNumber: number): Promise<void> {
    // NOTE: This function is also used in event handlers
    // where it cannot be 'awaited', so all exceptions need to be caught.

    log('Indexer got new block %d', blockNumber)
    this.emit('block', blockNumber)

    // update latest block
    if (this.latestBlock < blockNumber) {
      this.latestBlock = blockNumber
    }

    let lastSnapshot;
    try {
      lastSnapshot = await this.db.getLatestConfirmedSnapshotOrUndefined()

      // This new block marks a previous block
      // (blockNumber - this.maxConfirmations) is final.
      // Confirm native token transactions in that previous block.
      const nativeTxs = await this.chain.getNativeTokenTransactionInBlock(blockNumber - this.maxConfirmations, true)
      // update transaction manager
      if (nativeTxs.length > 0) {
        this.indexEvent('withdraw-native', nativeTxs)
        nativeTxs.forEach((nativeTx) => {
          this.chain.updateConfirmedTransaction(nativeTx)
        })
      }
    }
    catch (err) {
      log(`error: failed to retrieve information about block ${blockNumber} with finality ${this.maxConfirmations}`, err)
    }

    log('At the new block %d, there are %i unconfirmed events and ready to process %s, because the event was mined at %i (with finality %i)',
      blockNumber,
      this.unconfirmedEvents.length,
      this.unconfirmedEvents.length > 0
        ? isConfirmedBlock(this.unconfirmedEvents.top(1)[0].blockNumber, blockNumber, this.maxConfirmations)
        : null,
      this.unconfirmedEvents.length > 0 ? this.unconfirmedEvents.top(1)[0].blockNumber : 0,
      this.maxConfirmations
    )

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    while (
      this.unconfirmedEvents.length > 0 &&
      isConfirmedBlock(this.unconfirmedEvents.top(1)[0].blockNumber, blockNumber, this.maxConfirmations)
    ) {
      const event = this.unconfirmedEvents.pop()
      log('Processing event %s %s %s', event.event, blockNumber, this.maxConfirmations)

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

      const eventName = event.event as EventNames | TokenEventNames

      // update transaction manager
      this.chain.updateConfirmedTransaction(event.transactionHash)
      log('Event name %s and hash %s', eventName, event.transactionHash)
      try {
        if (eventName === ANNOUNCEMENT) {
          this.indexEvent('announce', [event.transactionHash])
          await this.onAnnouncement(event as Event<'Announcement'>, new BN(blockNumber.toPrecision()))
        } else if (eventName === 'ChannelUpdated') {
          await this.onChannelUpdated(event as Event<'ChannelUpdated'>)
        } else if (eventName === 'Transfer') {
          // handle HOPR token transfer
          this.indexEvent('withdraw-hopr', [event.transactionHash])
        } else {
          log(`ignoring event '${String(eventName)}'`)
        }
      } catch (err) {
        log('error processing event:', event, err)
      }

      try {
        lastSnapshot = new Snapshot(new BN(event.blockNumber), new BN(event.transactionIndex), new BN(event.logIndex))
        await this.db.updateLatestConfirmedSnapshot(lastSnapshot)
      }
      catch (err) {
        log(`error: failed to update latest confirmed snapshot in the database, eventBlockNum=${event.blockNumber}, txIdx=${event.transactionIndex}`, err)
      }
    }

    try {
      await this.db.updateLatestBlockNumber(new BN(blockNumber))
      this.emit('block-processed', blockNumber)
    }
    catch (err) {
      log(`error: failed to update database with latest block number ${blockNumber}`, err)
    }
  }

  /**
   * Called whenever we receive new events.
   * @param events
   */
  private onNewEvents(events: Event<any>[] | TokenEvent<any>[]): void {
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
    this.indexEvent('channel-updated', [event.transactionHash])
    log('channel-updated for hash %s', event.transactionHash)
    const channel = await ChannelEntry.fromSCEvent(event, (a: Address) => this.getPublicKeyOf(a))

    log(channel.toString())
    await this.db.updateChannel(channel.getId(), channel)

    let prevState
    try {
      prevState = await this.db.getChannel(channel.getId())
    } catch (e) {
      // Channel is new
    }

    if (prevState && channel.status == ChannelStatus.Closed && prevState.status != ChannelStatus.Closed) {
      log('channel was closed')
      this.onChannelClosed(channel)
    }

    this.emit('channel-update', channel)
    log('channel-update for channel %s', channel)

    if (channel.source.toAddress().eq(this.address) || channel.destination.toAddress().eq(this.address)) {
      this.emit('own-channel-updated', channel)

      if (channel.destination.toAddress().eq(this.address)) {
        // Channel _to_ us
        if (channel.status === ChannelStatus.WaitingForCommitment) {
          log('channel to us waiting for commitment', channel)
          this.emit('channel-waiting-for-commitment', channel)
        }
      }
    }
  }

  private async onChannelClosed(channel: ChannelEntry) {
    this.db.deleteAcknowledgedTicketsFromChannel(channel)
    this.emit('channel-closed', channel)
  }

  private indexEvent(indexerEvent: IndexerEvents, txHash: string[]) {
    this.emit(indexerEvent, txHash)
  }

  public async getAccount(address: Address) {
    return this.db.getAccount(address)
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
    const channels = await this.db.getChannels((channel) => channel.status === ChannelStatus.Open)

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
    return await this.db
      .getChannelsFrom(source.toAddress())
      .then((channels) => channels.filter((channel) => channel.status === ChannelStatus.Open))
  }

  public resolvePendingTransaction(eventType: IndexerEvents, tx: string): DeferType<string> {
    const deferred = {} as DeferType<string>

    deferred.promise = new Promise((resolve, reject) => {
      deferred.reject = () => {
        this.removeListener(eventType, listener)
        log('listener %s on %s timed out and thus removed', eventType, tx)
      }
      const timeoutObj = setTimeout(() => {
        deferred.reject() // remove listener but throw now error
        reject(tx)
      }, INDEXER_TIMEOUT)

      deferred.resolve = () => {
        clearTimeout(timeoutObj)
        this.removeListener(eventType, listener)
        log('listener %s on %s is removed', eventType, tx)
        resolve(tx)
      }

      const listener = (txHash: string[]) => {
        const indexed = txHash.find((emitted) => emitted === tx)
        if (indexed) deferred.resolve()
      }
      this.addListener(eventType, listener)
      log('listener %s on %s is added', eventType, tx)
    })

    return deferred
  }
}

export default Indexer
