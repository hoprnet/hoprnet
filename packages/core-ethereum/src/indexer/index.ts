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
  u8aConcat,
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
import { errors } from 'ethers'
import { INDEXER_TIMEOUT, MAX_TRANSACTION_BACKOFF } from '../constants'
import type { TypedEvent } from '@hoprnet/hopr-ethereum'

const log = debug('hopr-core-ethereum:indexer')
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
        await this.onNewBlock(block.value, true) // exceptions are handled
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
   * @param withTokenTransactions [optional] if true, also query for token transfer
   * towards or from the node towards someone else
   * @returns all relevant events in the specified block range
   */
  private async getEvents(
    fromBlock: number,
    toBlock: number,
    withTokenTransactions: boolean
  ): Promise<TypedEvent<any, any>[]> {
    const queries = [
      this.chain
        .getChannels()
        .queryFilter(
          {
            topics: [
              [
                this.chain.getChannels().interface.getEventTopic('Announcement'),
                this.chain.getChannels().interface.getEventTopic('ChannelUpdated'),
                this.chain.getChannels().interface.getEventTopic('TicketRedeemed')
              ]
            ]
          },
          fromBlock,
          toBlock
        )
        .then((events: TypedEvent<any, any>[]) => {
          return events.map((event: TypedEvent<any, any>) => {
            return Object.assign(event, this.chain.getChannels().interface.parseLog(event))
          })
        })
    ]

    if (withTokenTransactions) {
      queries.push(
        ...[
          this.chain
            .getToken()
            .queryFilter(
              {
                topics: [
                  // Token transfer *towards* us
                  [this.chain.getToken().interface.getEventTopic('Transfer')],
                  [u8aToHex(Uint8Array.from([...new Uint8Array(12).fill(0), ...this.address.serialize()]))]
                ]
              },
              fromBlock,
              toBlock
            )
            .then((events: TypedEvent<any, any>[]) => {
              return events.map((event: TypedEvent<any, any>) =>
                Object.assign(event, this.chain.getToken().interface.parseLog(event))
              )
            }),
          this.chain
            .getToken()
            .queryFilter(
              {
                topics: [
                  // Token transfer *from* us towards someone else
                  [this.chain.getToken().interface.getEventTopic('Transfer')],
                  null,
                  [u8aToHex(Uint8Array.from([...new Uint8Array(12).fill(0), ...this.address.serialize()]))]
                ]
              },
              fromBlock,
              toBlock
            )
            .then((events: TypedEvent<any, any>[]) => {
              return events.map((event: TypedEvent<any, any>) =>
                Object.assign(event, this.chain.getToken().interface.parseLog(event))
              )
            })
        ]
      )
    }

    const events = await Promise.all(queries)
    const normalizedEvents = events
      .flat(1)
      .sort(snapshotComparator)
      // @TODO fix type clash
      .map((event) => {
        if (event.event == undefined) {
          return Object.assign(event, { event: event.name })
        }
        return event
      })

    return normalizedEvents
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

      let events: Event<any>[] = []

      try {
        events = await this.getEvents(fromBlock, toBlock, false)
        log(`Getting events from ${fromBlock} to ${toBlock} successful, range ${toBlock - fromBlock}`)
      } catch (error) {
        failedCount++

        if (failedCount > 5) {
          throw error
        }

        continue
      }

      this.onNewEvents(events)
      await this.onNewBlock(toBlock, false)
      failedCount = 0
      fromBlock = toBlock

      log('Sync progress %d% @ block %d', getSyncPercentage(start, fromBlock, maxToBlock), toBlock)

      if (fromBlock < maxToBlock) {
        // Give other tasks CPU time to happen
        // Wait until end of next event loop iteration before starting next I/O query
        await setImmediatePromise()
      }
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
  private async onNewBlock(blockNumber: number, fetchEvents = false): Promise<void> {
    // NOTE: This function is also used in event handlers
    // where it cannot be 'awaited', so all exceptions need to be caught.

    log('Indexer got new block %d', blockNumber)
    this.emit('block', blockNumber)

    // update latest block
    this.latestBlock = Math.max(this.latestBlock, blockNumber)

    let lastDatabaseSnapshot: Snapshot | undefined
    try {
      lastDatabaseSnapshot = await this.db.getLatestConfirmedSnapshotOrUndefined()

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
    } catch (err) {
      log(
        `error: failed to retrieve information about block ${blockNumber} with finality ${this.maxConfirmations}`,
        err
      )
    }

    if (fetchEvents) {
      // Don't fail immediately when one block is temporarily not available
      const RETRIES = 3
      let events: TypedEvent<any, any>[] = []

      for (let i = 0; i < RETRIES; i++) {
        try {
          events = await this.getEvents(blockNumber - this.maxConfirmations, blockNumber - this.maxConfirmations, true)
        } catch (err) {
          if (i < RETRIES) {
            // Give other tasks CPU time to happen
            // Push next provider query to end of next event loop iteration
            await setImmediatePromise()
            continue
          } else {
            log(
              `Cannot fetch block ${blockNumber - this.maxConfirmations} despite ${RETRIES} retries. Skipping block.`,
              err
            )
          }
        }

        break
      }

      this.onNewEvents(events)
    }

    await this.processUnconfirmedEvents(blockNumber, lastDatabaseSnapshot)
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
  async processUnconfirmedEvents(blockNumber: number, lastDatabaseSnapshot: Snapshot | undefined) {
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

      // update transaction manager
      this.chain.updateConfirmedTransaction(event.transactionHash)
      log('Event name %s and hash %s', eventName, event.transactionHash)
      try {
        switch (eventName) {
          case 'Announcement':
          case 'Announcement(address,bytes,bytes)':
            this.indexEvent('announce', [event.transactionHash])
            await this.onAnnouncement(event as Event<'Announcement'>, new BN(blockNumber.toPrecision()))
            break
          case 'ChannelUpdated':
          case 'ChannelUpdated(address,address,tuple)':
            await this.onChannelUpdated(event as Event<'ChannelUpdated'>)
            break
          case 'Transfer':
          case 'Transfer(address,address,uint256)':
            // handle HOPR token transfer
            this.indexEvent('withdraw-hopr', [event.transactionHash])
            console.log('ON TRANSFER START')
            await this.onTransfer(event as TokenEvent<'Transfer'>)
            console.log('ON TRANSFER END')
            break
          case 'TicketRedeemed':
          case 'TicketRedeemed(address,address,bytes32,uint256,uint256,bytes32,uint256,uint256,bytes)':
            // if unlock `outstandingTicketBalance`, if applicable
            await this.onTicketRedeemed(event as Event<'TicketRedeemed'>)
            break
          default:
            log(`ignoring event '${String(eventName)}'`)
        }
      } catch (err) {
        log('error processing event:', event, err)
      }

      try {
        lastDatabaseSnapshot = new Snapshot(
          new BN(event.blockNumber),
          new BN(event.transactionIndex),
          new BN(event.logIndex)
        )
        await this.db.updateLatestConfirmedSnapshot(lastDatabaseSnapshot)
      } catch (err) {
        log(
          `error: failed to update latest confirmed snapshot in the database, eventBlockNum=${event.blockNumber}, txIdx=${event.transactionIndex}`,
          err
        )
      }

      if (
        this.unconfirmedEvents.size() > 0 &&
        isConfirmedBlock(this.unconfirmedEvents.peek().blockNumber, blockNumber, this.maxConfirmations)
      ) {
        // Give other tasks CPU time to happen
        // Wait until end of next event loop iteration before starting next db write-back
        await setImmediatePromise()
      }
    }

    try {
      await this.db.updateLatestBlockNumber(new BN(blockNumber))
      this.emit('block-processed', blockNumber)
    } catch (err) {
      log(`error: failed to update database with latest block number ${blockNumber}`, err)
    }
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
    const channel = await ChannelEntry.fromSCEvent(event, this.getPublicKeyOf.bind(this))
    log(`Smart contract event`)
    log(channel.toString())

    let prevState: ChannelEntry
    try {
      prevState = await this.db.getChannel(channel.getId())
    } catch (e) {
      // Channel is new
    }

    await this.db.updateChannel(channel.getId(), channel)

    if (prevState && channel.status == ChannelStatus.Closed && prevState.status != ChannelStatus.Closed) {
      log('channel was closed')
      await this.onChannelClosed(channel)
    }

    this.emit('channel-update', channel)
    log('channel-update for channel')
    log(channel.toString())

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

  private async onTicketRedeemed(event: Event<'TicketRedeemed'>) {
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
          await this.db.resolvePending(partialTicket)
        } else {
          await this.db.resolvePending({
            ...partialTicket,
            amount: outstandingBalance
          })
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

  private async onTransfer(event: TokenEvent<'Transfer'>) {
    console.log('onTransfer start', event.args.value.toString())
    const isIncoming = Address.fromString(event.args.to).eq(this.address)
    const amount = new Balance(new BN(event.args.value.toString()))

    if (isIncoming) {
      await this.db.addHoprBalance(amount)
    } else {
      await this.db.subHoprBalance(amount)
    }

    console.log('onTransfer end', event.args.value.toString())
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

    deferred.promise = new Promise((resolve, reject) => {
      let done = false

      deferred.reject = () => {
        if (done) {
          return
        }
        done = true
        this.removeListener(eventType, listener)
        log('listener %s on %s is removed due to error', eventType, tx)
        setImmediate(resolve, tx)
      }

      setTimeout(() => {
        if (done) {
          return
        }
        done = true
        // remove listener but throw now error
        this.removeListener(eventType, listener)
        log('listener %s on %s timed out and thus removed', eventType, tx)
        setImmediate(reject, tx)
      }, INDEXER_TIMEOUT)

      deferred.resolve = () => {
        if (done) {
          return
        }
        done = true
        this.removeListener(eventType, listener)
        log('listener %s on %s is removed', eventType, tx)

        setImmediate(resolve, tx)
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
