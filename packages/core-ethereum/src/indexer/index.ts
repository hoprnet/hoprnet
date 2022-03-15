import { setImmediate as setImmediatePromise } from 'timers/promises'
import BN from 'bn.js'
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
  debug,
  retryWithBackoff,
  Balance,
  ordered,
  u8aToHex,
  FIFO,
  type DeferType,
  type Ticket
} from '@hoprnet/hopr-utils'

import type { ChainWrapper } from '../ethereum'
import type { Event, EventNames, IndexerEvents, TokenEvent, TokenEventNames } from './types'
import { isConfirmedBlock, snapshotComparator, type IndexerSnapshot } from './utils'
import { Contract, errors } from 'ethers'
import { INDEXER_TIMEOUT, MAX_TRANSACTION_BACKOFF } from '../constants'
import type { TypedEvent, TypedEventFilter } from '@hoprnet/hopr-ethereum'

const log = debug('hopr-core-ethereum:indexer')
const verbose = debug('hopr-core-ethereum:verbose:indexer')

const getSyncPercentage = (start: number, current: number, end: number) =>
  (((current - start) / (end - start)) * 100).toFixed(2)
const backoffOption: Parameters<typeof retryWithBackoff>[1] = { maxDelay: MAX_TRANSACTION_BACKOFF }

/**
 * Indexes HoprChannels smart contract and stores to the DB,
 * all channels in the network.
 * Also keeps track of the latest block number.
 */
class Indexer extends EventEmitter {
  public status: 'started' | 'restarting' | 'stopped' = 'stopped'
  public latestBlock: number = 0 // latest known on-chain block number

  // Use FIFO + sliding window for many events
  private unconfirmedEvents: FIFO<TypedEvent<any, any>>

  private chain: ChainWrapper
  private genesisBlock: number
  private lastSnapshot: IndexerSnapshot | undefined

  private unsubscribeErrors: () => void
  private unsubscribeBlock: () => void

  constructor(
    private address: Address,
    private db: HoprDB,
    private maxConfirmations: number,
    private blockRange: number
  ) {
    super()

    this.unconfirmedEvents = FIFO<TypedEvent<any, any>>()
  }

  /**
   * Starts indexing.
   */
  public async start(chain: ChainWrapper, genesisBlock: number): Promise<void> {
    if (this.status === 'started') {
      return
    }
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
    fromBlock = Math.max(fromBlock, this.genesisBlock)

    log('Starting to index from block %d, sync progress 0%%', fromBlock)

    const orderedBlocks = ordered<number>()

    // Starts the asynchronous stream of indexer events
    // and feeds them to the event listener
    ;(async function (this: Indexer) {
      for await (const block of orderedBlocks.iterator()) {
        await this.onNewBlock(block.value, true, true) // exceptions are handled
      }
    }.call(this))

    // Do not process new blocks before querying old blocks has finished
    const newBlocks = ordered<number>()

    const unsubscribeBlock = this.chain.subscribeBlock(async (block: number) => {
      newBlocks.push({
        index: block,
        value: block
      })
    })

    this.unsubscribeBlock = () => {
      unsubscribeBlock()
      newBlocks.end()
      orderedBlocks.end()
    }

    this.unsubscribeErrors = this.chain.subscribeError(async (error: any) => {
      await this.onProviderError(error) // exceptions are handled
    })

    // get past events
    fromBlock = await this.processPastEvents(fromBlock, latestOnChainBlock, this.blockRange)

    // Feed new blocks to the ordered queue
    ;(async function (this: Indexer) {
      for await (const newBlock of newBlocks.iterator()) {
        orderedBlocks.push(newBlock) // exceptions are handled
      }
    }.call(this))

    log('Subscribing to events from block %d', fromBlock)

    this.status = 'started'
    this.emit('status', 'started')
    log(chalk.green('Indexer started!'))
  }

  /**
   * Stops indexing.
   */
  public stop(): void {
    if (this.status === 'stopped') {
      return
    }

    log(`Stopping indexer...`)

    this.unsubscribeBlock()
    this.unsubscribeErrors()

    this.status = 'stopped'
    this.emit('status', 'stopped')
    log(chalk.green('Indexer stopped!'))
  }

  /**
   * Restarts the indexer
   * @returns a promise that resolves once the indexer
   * has been restarted
   */
  protected async restart(): Promise<void> {
    if (this.status === 'restarting') {
      return
    }

    log('Indexer restaring')

    try {
      this.status = 'restarting'

      this.stop()
      await this.start(this.chain, this.genesisBlock)
    } catch (err) {
      this.status = 'stopped'
      this.emit('status', 'stopped')
      log(chalk.red('Failed to restart: %s', err.message))
      throw err
    }
  }

  /**
   * Gets all interesting on-chain events, such as Transfer events and payment
   * channel events
   * @param fromBlock block to start from
   * @param toBlock last block (inclusive) to consider
   * towards or from the node towards someone else
   * @returns all relevant events in the specified block range
   */
  private async getEvents(
    fromBlock: number,
    toBlock: number,
    fetchTokenTransactions = false
  ): Promise<
    | {
        success: true
        events: TypedEvent<any, any>[]
      }
    | {
        success: false
      }
  > {
    let rawEvents: TypedEvent<any, any>[] = []

    const queries: { contract: Contract; filter: TypedEventFilter<any> }[] = [
      {
        contract: this.chain.getChannels(),
        filter: {
          topics: [
            [
              // Relevant channel events
              this.chain.getChannels().interface.getEventTopic('Announcement'),
              this.chain.getChannels().interface.getEventTopic('ChannelUpdated'),
              this.chain.getChannels().interface.getEventTopic('TicketRedeemed')
            ]
          ]
        }
      }
    ]

    // Actively query for logs to prevent polling done by Ethers.js
    // that don't retry on failed attempts and thus makes the indexer
    // handle errors produced by internal Ethers.js provider calls
    if (fetchTokenTransactions) {
      queries.push({
        contract: this.chain.getToken(),
        filter: {
          topics: [
            // Token transfer *from* us
            [this.chain.getToken().interface.getEventTopic('Transfer')],
            [u8aToHex(this.address.toBytes32())]
          ]
        }
      })
      queries.push({
        contract: this.chain.getToken(),
        filter: {
          topics: [
            // Token transfer *towards* us
            [this.chain.getToken().interface.getEventTopic('Transfer')],
            null,
            [u8aToHex(this.address.toBytes32())]
          ]
        }
      })
    }

    for (const query of queries) {
      let tmpEvents: TypedEvent<any, any>[]
      try {
        tmpEvents = (await query.contract.queryFilter(query.filter, fromBlock, toBlock)) as any
      } catch {
        return {
          success: false
        }
      }

      for (const event of tmpEvents) {
        Object.assign(event, query.contract.interface.parseLog(event))

        if (event.event == undefined) {
          Object.assign(event, { event: (event as any).name })
        }
        rawEvents.push(event)
      }
    }

    // sort in-place
    rawEvents.sort(snapshotComparator)

    return {
      success: true,
      events: rawEvents
    }
  }

  /**
   * Query past events, this will loop until it gets all blocks from `toBlock` to `fromBlock`.
   * If we exceed response pull limit, we switch into quering smaller chunks.
   * TODO: optimize DB and fetch requests
   * @param fromBlock
   * @param maxToBlock
   * @param maxBlockRange
   * @return past events and last queried block
   */
  private async processPastEvents(fromBlock: number, maxToBlock: number, maxBlockRange: number): Promise<number> {
    const start = fromBlock
    let failedCount = 0

    while (fromBlock < maxToBlock) {
      const blockRange = failedCount > 0 ? Math.floor(maxBlockRange / 4 ** failedCount) : maxBlockRange
      // should never be above maxToBlock
      let toBlock = Math.min(fromBlock + blockRange, maxToBlock)

      // log(
      //   `${failedCount > 0 ? 'Re-quering' : 'Quering'} past events from %d to %d: %d`,
      //   fromBlock,
      //   toBlock,
      //   toBlock - fromBlock
      // )

      let res = await this.getEvents(fromBlock, toBlock)

      if (res.success) {
        this.onNewEvents(res.events)
        await this.onNewBlock(toBlock, false, false, true)
      } else {
        failedCount++

        if (failedCount > 5) {
          throw Error(`Could not fetch logs from block ${fromBlock} to ${toBlock}. Giving up`)
        }

        await setImmediatePromise()

        continue
      }

      failedCount = 0
      fromBlock = toBlock

      log('Sync progress %d% @ block %d', getSyncPercentage(start, fromBlock, maxToBlock), toBlock)
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
    if (String(error).match(/eth_blockNumber/)) {
      verbose(`Ignoring failed "eth_blockNumber" provider call from Ethers.js`)
      return
    }

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
          await retryWithBackoff(
            () =>
              Promise.allSettled([
                ...this.chain
                  .getAllQueuingTransactionRequests()
                  .map((request) => this.chain.sendTransaction(request as string)),
                this.restart()
              ]),
            backoffOption
          )
        }
      } else {
        await retryWithBackoff(() => this.restart(), backoffOption)
      }
    } catch (err) {
      log(`error: exception while processing another provider error ${error}`, err)
    }
  }

  /**
   * Called whenever a new block found.
   * This will update `this.latestBlock`,
   * and processes events which are within
   * confirmed blocks.
   * @param blockNumber latest on-chain block number
   * @param fetchEvents [optional] if true, query provider for events in block
   */
  private async onNewBlock(
    blockNumber: number,
    fetchEvents = false,
    fetchNativeTxs = false,
    blocking = false
  ): Promise<void> {
    // NOTE: This function is also used in event handlers
    // where it cannot be 'awaited', so all exceptions need to be caught.

    const currentBlock = blockNumber - this.maxConfirmations

    if (currentBlock < 0) {
      return
    }

    log('Indexer got new block %d, handling block %d', blockNumber, blockNumber - this.maxConfirmations)
    this.emit('block', blockNumber)

    // update latest block
    this.latestBlock = Math.max(this.latestBlock, blockNumber)

    let lastDatabaseSnapshot = await this.db.getLatestConfirmedSnapshotOrUndefined()

    if (fetchEvents) {
      // Don't fail immediately when one block is temporarily not available
      const RETRIES = 3
      let res: Awaited<ReturnType<Indexer['getEvents']>>

      for (let i = 0; i < RETRIES; i++) {
        res = await this.getEvents(currentBlock, currentBlock, true)

        if (res.success) {
          this.onNewEvents(res.events)
          break
        } else if (i + 1 < RETRIES) {
          await setImmediatePromise()
        } else {
          log(`Cannot fetch block ${currentBlock} despite ${RETRIES} retries. Skipping block.`)
        }
      }
    }

    await this.processUnconfirmedEvents(blockNumber, lastDatabaseSnapshot, blocking)

    if (fetchNativeTxs) {
      let nativeTxHashes: string[] | undefined
      try {
        // This new block marks a previous block
        // (blockNumber - this.maxConfirmations) is final.
        // Confirm native token transactions in that previous block.
        nativeTxHashes = await this.chain.getTransactionsInBlock(currentBlock)
      } catch (err) {
        log(
          `error: failed to retrieve information about block ${currentBlock} with finality ${this.maxConfirmations}`,
          err
        )
      }

      // update transaction manager after updating db
      if (nativeTxHashes && nativeTxHashes.length > 0) {
        // @TODO replace this with some efficient set intersection algorithm
        for (const txHash of nativeTxHashes) {
          if (this.listeners(`withdraw-native-${txHash}`).length > 0) {
            this.indexEvent(`withdraw-native-${txHash}`)
          } else if (this.listeners(`withdraw-hopr-${txHash}`).length > 0) {
            this.indexEvent(`withdraw-hopr-${txHash}`)
          } else if (this.listeners(`announce-${txHash}`).length > 0) {
            this.indexEvent(`announce-${txHash}`)
          } else if (this.listeners(`channel-updated-${txHash}`).length > 0) {
            this.indexEvent(`channel-updated-${txHash}`)
          }

          this.chain.updateConfirmedTransaction(txHash)
        }
      }
    }

    try {
      await this.db.updateLatestBlockNumber(new BN(blockNumber))
    } catch (err) {
      log(`error: failed to update database with latest block number ${blockNumber}`, err)
    }

    this.emit('block-processed', currentBlock)
  }

  /**
   * Adds new events to the queue of unprocessed events
   * @dev ignores events that have been processed before.
   * @param events new unprocessed events
   */
  private onNewEvents(events: Event<any>[] | TokenEvent<any>[] | undefined): void {
    if (events == undefined || events.length == 0) {
      // Nothing to do
      return
    }

    let offset = 0

    // lastSnapshot ~= watermark of previously process events
    //
    // lastSnapshot == undefined means there is no watermark, hence
    // all events can be considered new
    if (this.lastSnapshot != undefined) {
      let currentSnapshot: IndexerSnapshot = {
        blockNumber: events[offset].blockNumber,
        logIndex: events[offset].logIndex,
        transactionIndex: events[offset].transactionIndex
      }

      // As long events are older than the current watermark,
      // increase the offset to ignore them
      while (snapshotComparator(this.lastSnapshot, currentSnapshot) >= 0) {
        offset++
        if (offset < events.length) {
          currentSnapshot = {
            blockNumber: events[offset].blockNumber,
            logIndex: events[offset].logIndex,
            transactionIndex: events[offset].transactionIndex
          }
        } else {
          break
        }
      }
    }

    // Once the offset is known upon which we have
    // received new events, add them to `unconfirmedEvents` to
    // be processed once the next+confirmationTime block
    // has been mined
    for (; offset < events.length; offset++) {
      this.unconfirmedEvents.push(events[offset])
    }

    // Update watermark for next iteration
    this.lastSnapshot = {
      blockNumber: events[events.length - 1].blockNumber,
      logIndex: events[events.length - 1].logIndex,
      transactionIndex: events[events.length - 1].transactionIndex
    }
  }

  /**
   * Process all stored but not yet processed events up to latest
   * confirmed block (latestBlock - confirmationTime)
   * @param blockNumber latest on-chain block number
   * @param lastDatabaseSnapshot latest snapshot in database
   */
  async processUnconfirmedEvents(blockNumber: number, lastDatabaseSnapshot: Snapshot | undefined, blocking: boolean) {
    log(
      'At the new block %d, there are %i unconfirmed events and ready to process %s, because the event was mined at %i (with finality %i)',
      blockNumber,
      this.unconfirmedEvents.size(),
      this.unconfirmedEvents.size() > 0
        ? isConfirmedBlock(this.unconfirmedEvents.peek().blockNumber, blockNumber, this.maxConfirmations)
        : null,
      this.unconfirmedEvents.size() > 0 ? this.unconfirmedEvents.peek().blockNumber : 0,
      this.maxConfirmations
    )

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    while (
      this.unconfirmedEvents.size() > 0 &&
      isConfirmedBlock(this.unconfirmedEvents.peek().blockNumber, blockNumber, this.maxConfirmations)
    ) {
      const event = this.unconfirmedEvents.shift()
      log(
        'Processing event %s blockNumber=%s maxConfirmations=%s',
        // @TODO: fix type clash
        event.event,
        blockNumber,
        this.maxConfirmations
      )

      // if we find a previous snapshot, compare event's snapshot with last processed
      if (lastDatabaseSnapshot) {
        const lastSnapshotComparison = snapshotComparator(event, {
          blockNumber: lastDatabaseSnapshot.blockNumber.toNumber(),
          logIndex: lastDatabaseSnapshot.logIndex.toNumber(),
          transactionIndex: lastDatabaseSnapshot.transactionIndex.toNumber()
        })

        // check if this is a duplicate or older than last snapshot
        // ideally we would have detected if this snapshot was indeed processed,
        // at the moment we don't keep all events stored as we intend to keep
        // this indexer very simple
        if (lastSnapshotComparison == 0 || lastSnapshotComparison < 0) {
          continue
        }
      }

      // @TODO: fix type clash
      const eventName = event.event as EventNames | TokenEventNames

      lastDatabaseSnapshot = new Snapshot(
        new BN(event.blockNumber),
        new BN(event.transactionIndex),
        new BN(event.logIndex)
      )

      // update transaction manager
      this.chain.updateConfirmedTransaction(event.transactionHash)
      log('Event name %s and hash %s', eventName, event.transactionHash)

      switch (eventName) {
        case 'Announcement':
        case 'Announcement(address,bytes,bytes)':
          await this.onAnnouncement(
            event as Event<'Announcement'>,
            new BN(blockNumber.toPrecision()),
            lastDatabaseSnapshot
          )
          break
        case 'ChannelUpdated':
        case 'ChannelUpdated(address,address,tuple)':
          await this.onChannelUpdated(event as Event<'ChannelUpdated'>, lastDatabaseSnapshot)
          break
        case 'Transfer':
        case 'Transfer(address,address,uint256)':
          // handle HOPR token transfer
          await this.onTransfer(event as TokenEvent<'Transfer'>, lastDatabaseSnapshot)
          break
        case 'TicketRedeemed':
        case 'TicketRedeemed(address,address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)':
          // if unlock `outstandingTicketBalance`, if applicable
          await this.onTicketRedeemed(event as Event<'TicketRedeemed'>, lastDatabaseSnapshot)
          break
        default:
          log(`ignoring event '${String(eventName)}'`)
      }

      if (
        !blocking &&
        this.unconfirmedEvents.size() > 0 &&
        isConfirmedBlock(this.unconfirmedEvents.peek().blockNumber, blockNumber, this.maxConfirmations)
      ) {
        // Give other tasks CPU time to happen
        // Wait until end of next event loop iteration before starting next db write-back
        await setImmediatePromise()
      }
    }
  }

  private async onAnnouncement(event: Event<'Announcement'>, blockNumber: BN, lastSnapshot: Snapshot): Promise<void> {
    // publicKey given by the SC is verified
    const publicKey = PublicKey.fromUncompressedPubKey(
      // add uncompressed key identifier
      Uint8Array.from([4, ...stringToU8a(event.args.publicKey)])
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

    await this.db.updateAccountAndSnapshot(account, lastSnapshot)

    this.emit('peer', {
      id: account.getPeerId(),
      multiaddrs: [account.multiAddr]
    })
  }

  private async onChannelUpdated(event: Event<'ChannelUpdated'>, lastSnapshot: Snapshot): Promise<void> {
    log('channel-updated for hash %s', event.transactionHash)
    const channel = await ChannelEntry.fromSCEvent(event, this.getPublicKeyOf.bind(this))

    let prevState: ChannelEntry
    try {
      prevState = await this.db.getChannel(channel.getId())
    } catch (e) {
      // Channel is new
    }

    await this.db.updateChannelAndSnapshot(channel.getId(), channel, lastSnapshot)

    if (prevState && channel.status == ChannelStatus.Closed && prevState.status != ChannelStatus.Closed) {
      log('channel was closed')
      await this.onChannelClosed(channel)
    }

    this.emit('channel-update', channel)
    verbose('channel-update for channel')
    verbose(channel.toString())

    if (channel.source.toAddress().eq(this.address) || channel.destination.toAddress().eq(this.address)) {
      this.emit('own-channel-updated', channel)

      if (channel.destination.toAddress().eq(this.address)) {
        // Channel _to_ us
        if (channel.status === ChannelStatus.WaitingForCommitment) {
          log('channel to us waiting for commitment')
          log(channel.toString())
          this.emit('channel-waiting-for-commitment', channel)
        }
      }
    }
  }

  private async onTicketRedeemed(event: Event<'TicketRedeemed'>, lastSnapshot: Snapshot) {
    if (Address.fromString(event.args.source).eq(this.address)) {
      // the node used to lock outstandingTicketBalance
      // rebuild part of the Ticket
      const partialTicket: Partial<Ticket> = {
        counterparty: Address.fromString(event.args.destination),
        amount: new Balance(new BN(event.args.amount.toString()))
      }
      const outstandingBalance = await this.db.getPendingBalanceTo(partialTicket.counterparty)

      try {
        if (!outstandingBalance.toBN().gte(new BN('0'))) {
          await this.db.resolvePending(partialTicket, lastSnapshot)
        } else {
          await this.db.resolvePending(
            {
              ...partialTicket,
              amount: outstandingBalance
            },
            lastSnapshot
          )
          // It falls into this case when db of sender gets erased while having tickets pending.
          // TODO: handle this may allow sender to send arbitrary amount of tickets through open
          // channels with positive balance, before the counterparty initiates closure.
        }
      } catch (error) {
        log(`error in onTicketRedeemed ${error}`)
        throw new Error(`error in onTicketRedeemed ${error}`)
      }
    }
  }

  private async onChannelClosed(channel: ChannelEntry) {
    await this.db.deleteAcknowledgedTicketsFromChannel(channel)
    this.emit('channel-closed', channel)
  }

  private async onTransfer(event: TokenEvent<'Transfer'>, lastSnapshot: Snapshot) {
    const isIncoming = Address.fromString(event.args.to).eq(this.address)
    const amount = new Balance(new BN(event.args.value.toString()))

    if (isIncoming) {
      await this.db.addHoprBalance(amount, lastSnapshot)
    } else {
      await this.db.subHoprBalance(amount, lastSnapshot)
    }
  }

  private indexEvent(indexerEvent: IndexerEvents) {
    this.emit(indexerEvent)
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
    const accounts = await this.db.getAccounts((account: AccountEntry) => account.containsRouting())

    const result: { id: PeerId; multiaddrs: Multiaddr[] }[] = Array.from({ length: accounts.length })

    log(`Known public nodes:`)
    for (const [index, account] of accounts.entries()) {
      result[index] = {
        id: account.getPeerId(),
        multiaddrs: [account.multiAddr]
      }
      log(`\t${account.getPeerId().toB58String()} ${account.multiAddr.toString()}`)
    }

    return result
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
      .then((channels: ChannelEntry[]) => channels.filter((channel) => channel.status === ChannelStatus.Open))
  }

  public resolvePendingTransaction(eventType: IndexerEvents, tx: string): DeferType<string> {
    const deferred = {} as DeferType<string>

    deferred.promise = new Promise<string>((resolve, reject) => {
      let done = false

      deferred.reject = () => {
        if (done) {
          return
        }
        done = true
        this.removeListener(eventType, deferred.resolve)
        log('listener %s on %s is removed due to error', eventType, tx)
        setImmediate(resolve, tx)
      }

      setTimeout(() => {
        if (done) {
          return
        }
        done = true
        // remove listener but throw now error
        this.removeListener(eventType, deferred.resolve)
        log('listener %s on %s timed out and thus removed', eventType, tx)
        setImmediate(reject, tx)
      }, INDEXER_TIMEOUT)

      deferred.resolve = () => {
        if (done) {
          return
        }
        done = true
        this.removeListener(eventType, deferred.resolve)
        log('listener %s on %s is removed', eventType, tx)

        setImmediate(resolve, tx)
      }

      this.addListener(eventType, deferred.resolve)
      log('listener %s on %s is added', eventType, tx)
    })

    return deferred
  }
}

export default Indexer
