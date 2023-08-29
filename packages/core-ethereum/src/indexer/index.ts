import { setImmediate as setImmediatePromise } from 'timers/promises'
import BN from 'bn.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'
import { EventEmitter } from 'events'
import { Multiaddr } from '@multiformats/multiaddr'
import {
  defer,
  ChannelStatus,
  Balance,
  BalanceType,
  Address,
  ChannelEntry,
  AccountEntry,
  Snapshot,
  debug,
  retryWithBackoffThenThrow,
  ordered,
  FIFO,
  type DeferType,
  create_multi_counter,
  create_gauge,
  Database,
  Handlers,
  random_integer,
  OffchainPublicKey,
  CORE_ETHEREUM_CONSTANTS,
  U256
} from '@hoprnet/hopr-utils'

import type { ChainWrapper } from '../ethereum.js'
import { type IndexerEventEmitter, IndexerStatus, type IndexerEvents } from './types.js'
import { isConfirmedBlock, snapshotComparator, type IndexerSnapshot } from './utils.js'
import { BigNumber, errors } from 'ethers'
import { Filter, Log } from '@ethersproject/abstract-provider'

// @ts-ignore untyped library
import retimer from 'retimer'

// Exported from Rust
const constants = CORE_ETHEREUM_CONSTANTS()

const log = debug('hopr-core-ethereum:indexer')
const error = debug('hopr-core-ethereum:indexer:error')
const verbose = debug('hopr-core-ethereum:verbose:indexer')

const getSyncPercentage = (start: number, current: number, end: number) =>
  (((current - start) / (end - start)) * 100).toFixed(2)
const backoffOption: Parameters<typeof retryWithBackoffThenThrow>[1] = { maxDelay: constants.MAX_TRANSACTION_BACKOFF }

// Metrics
const metric_indexerErrors = create_multi_counter(
  'core_ethereum_mcounter_indexer_provider_errors',
  'Multicounter for provider errors in Indexer',
  ['type']
)
// const metric_unconfirmedBlocks = create_counter(
//   'core_ethereum_counter_indexer_processed_unconfirmed_blocks',
//   'Number of processed unconfirmed blocks'
// )
// const metric_numAnnouncements = create_counter(
//   'core_ethereum_counter_indexer_announcements',
//   'Number of processed announcements'
// )
const metric_blockNumber = create_gauge('core_ethereum_gauge_indexer_block_number', 'Current block number')
// const metric_channelStatus = create_multi_gauge(
//   'core_ethereum_gauge_indexer_channel_status',
//   'Status of different channels',
//   ['channel']
// )
// const metric_ticketsRedeemed = create_counter(
//   'core_ethereum_counter_indexer_tickets_redeemed',
//   'Number of redeemed tickets'
// )

/**
 * Indexes HoprChannels smart contract and stores to the DB,
 * all channels in the network.
 * Also keeps track of the latest block number.
 */
class Indexer extends (EventEmitter as new () => IndexerEventEmitter) {
  public status: IndexerStatus = IndexerStatus.STOPPED
  public latestBlock: number = 0 // latest known on-chain block number
  public startupBlock: number = 0 // blocknumber at which the indexer starts

  // Use FIFO + sliding window for many events
  private unconfirmedEvents: FIFO<Log>

  private chain: ChainWrapper
  private genesisBlock: number
  private lastSnapshot: IndexerSnapshot | undefined
  private safeAddress: Address

  private blockProcessingLock: DeferType<void> | undefined

  private unsubscribeErrors: () => void
  private unsubscribeBlock: () => void

  private handlers: Handlers

  constructor(
    private address: Address,
    private db: Database,
    private maxConfirmations: number,
    private blockRange: number
  ) {
    super()

    this.unconfirmedEvents = FIFO<Log>()
  }

  /**
   * Starts indexing.
   */
  public async start(chain: ChainWrapper, genesisBlock: number, safeAddress: Address): Promise<void> {
    if (this.status === IndexerStatus.STARTED) {
      return
    }
    this.status = IndexerStatus.STARTING
    const contractAddresses = chain.getInfo()
    log(`[DEBUG]contractAddresses...${JSON.stringify(contractAddresses, null, 2)}`)

    this.handlers = Handlers.init(
      safeAddress.to_string(),
      chain.getPublicKey().to_address().to_string(),
      {
        channels: contractAddresses.hoprChannelsAddress,
        token: contractAddresses.hoprTokenAddress,
        network_registry: contractAddresses.hoprNetworkRegistryAddress,
        announcements: contractAddresses.hoprAnnouncementsAddress,
        node_safe_registry: contractAddresses.hoprNodeSafeRegistryAddress,
        node_management_module: contractAddresses.moduleAddress
      },
      {
        newAnnouncement: this.onAnnouncementUpdate.bind(this),
        ownChannelUpdated: this.onOwnChannelUpdated.bind(this),
        notAllowedToAccessNetwork: this.onNotAllowedToAccessNetwork.bind(this)
      }
    )

    log(`Starting indexer...`)
    this.chain = chain
    this.genesisBlock = genesisBlock
    this.safeAddress = safeAddress

    const [latestSavedBlock, latestOnChainBlock] = await Promise.all([
      new BN(await this.db.get_latest_block_number()).toNumber(),
      this.chain.getLatestBlockNumber()
    ])

    this.latestBlock = latestOnChainBlock
    this.startupBlock = latestOnChainBlock

    log('Latest saved block %d', latestSavedBlock)
    log('Latest on-chain block %d', latestOnChainBlock)
    log('Genesis block %d', genesisBlock)

    // go back 'MAX_CONFIRMATIONS' blocks in case of a re-org at time of stopping
    let fromBlock = latestSavedBlock
    if (fromBlock - this.maxConfirmations > 0) {
      fromBlock = fromBlock - this.maxConfirmations
    }
    // no need to query before HoprChannels or HoprNetworkRegistry existed
    fromBlock = Math.max(fromBlock, this.genesisBlock)

    // update the base valuse of balance and allowance of token for safe
    if (!this.lastSnapshot) {
      // update safe's HOPR token balance
      log(`get safe ${this.safeAddress.to_string()} HOPR balance at block ${fromBlock}`)
      const hoprBalance = await this.chain.getBalanceAtBlock(this.safeAddress, fromBlock)
      await this.db.set_hopr_balance(Balance.deserialize(hoprBalance.serialize_value(), BalanceType.HOPR))
      log(`set safe HOPR balance to ${hoprBalance.to_formatted_string()}`)

      // update safe's HORP token allowance granted to Channels contract
      log(`get safe ${this.safeAddress.to_string()} HOPR allowance at block ${fromBlock}`)
      const safeAllowance = await this.chain.getTokenAllowanceGrantedToChannelsAt(this.safeAddress, fromBlock)
      await this.db.set_staking_safe_allowance(
        Balance.deserialize(safeAllowance.serialize_value(), BalanceType.HOPR),
        new Snapshot(new U256('0'), new U256('0'), new U256('0')) // dummy snapshot
      )
      log(`set safe allowance to ${safeAllowance.to_formatted_string()}`)
    }

    log('Starting to index from block %d, sync progress 0%%', fromBlock)

    const orderedBlocks = ordered<number>()

    // Starts the asynchronous stream of indexer events
    // and feeds them to the event listener
    ;(async function (this: Indexer) {
      for await (const block of orderedBlocks.iterator()) {
        await this.onNewBlock(block.value, true, true) // exceptions are handled (for real)
      }
    }).call(this)

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
    }).call(this)

    log('Subscribing to events from block %d', fromBlock)

    this.status = IndexerStatus.STARTED
    this.emit('status', IndexerStatus.STARTED)
    log('Indexer started!')
  }

  /**
   * Stops indexing.
   */
  public async stop(): Promise<void> {
    if (this.status === IndexerStatus.STOPPED) {
      return
    }

    log(`Stopping indexer...`)

    this.unsubscribeBlock()
    this.unsubscribeErrors()

    this.blockProcessingLock && (await this.blockProcessingLock.promise)

    this.status = IndexerStatus.STOPPED
    this.emit('status', IndexerStatus.STOPPED)
    log('Indexer stopped!')
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
      this.status = IndexerStatus.RESTARTING

      this.stop()
      await this.start(this.chain, this.genesisBlock, this.safeAddress)
    } catch (err) {
      this.status = IndexerStatus.STOPPED
      this.emit('status', IndexerStatus.STOPPED)
      log('Failed to restart: %s', err.message)
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
        events: Log[]
      }
    | {
        success: false
      }
  > {
    let rawEvents: Log[] = []

    const provider = this.chain.getProvider()
    const contractAddresses = this.chain.getInfo()

    let queries: Filter[] = [
      {
        address: contractAddresses.hoprAnnouncementsAddress,
        topics: [this.handlers.get_announcement_topics()],
        fromBlock,
        toBlock
      },
      {
        address: contractAddresses.hoprChannelsAddress,
        topics: [this.handlers.get_channel_topics()],
        fromBlock,
        toBlock
      },
      {
        address: contractAddresses.hoprNodeSafeRegistryAddress,
        topics: [this.handlers.get_node_safe_registry_topics()],
        fromBlock,
        toBlock
      },
      {
        address: contractAddresses.hoprNetworkRegistryAddress,
        topics: [this.handlers.get_network_registry_topics()],
        fromBlock,
        toBlock
      },
      {
        address: contractAddresses.hoprTicketPriceOracleAddress,
        topics: [this.handlers.get_ticket_price_oracle_topics()],
        fromBlock,
        toBlock
      }
    ]

    // Token events
    // Actively query for logs to prevent polling done by Ethers.js
    // that don't retry on failed attempts and thus makes the indexer
    // handle errors produced by internal Ethers.js provider calls
    if (fetchTokenTransactions) {
      queries.push({
        address: contractAddresses.hoprTokenAddress,
        topics: [this.handlers.get_token_topics()],
        fromBlock,
        toBlock
      })
    }

    for (const query of queries) {
      try {
        rawEvents.push(...(await provider.getLogs(query)))
      } catch {
        return {
          success: false
        }
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

      let res = await this.getEvents(fromBlock, toBlock, true)

      if (res.success) {
        this.onNewEvents(res.events)
        await this.onNewBlock(toBlock, false, false)
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

    log(`etherjs error: ${error}`)

    try {
      const errorType = [errors.SERVER_ERROR, errors.TIMEOUT, 'ECONNRESET', 'ECONNREFUSED'].filter((err) =>
        [error?.code, String(error)].includes(err)
      )

      // if provider connection issue
      if (errorType.length != 0) {
        metric_indexerErrors.increment([errorType[0]])

        log('code error falls here', this.chain.getAllQueuingTransactionRequests().length)
        // allow the indexer to restart even there is no transaction in queue
        await retryWithBackoffThenThrow(
          () =>
            Promise.allSettled([
              ...this.chain.getAllQueuingTransactionRequests().map((request) => {
                // convert TransactionRequest to signed transaction and send out
                return this.chain.sendTransaction(
                  true,
                  { to: request.to, value: request.value as BigNumber, data: request.data as string },
                  // the transaction is resolved without the specific tag for its action, but rather as a result of provider retry
                  (txHash: string) => this.resolvePendingTransaction(`on-provider-error-${txHash}`, txHash)
                )
              }),
              this.restart()
            ]),
          backoffOption
        )
      } else {
        metric_indexerErrors.increment(['unknown'])
        await retryWithBackoffThenThrow(() => this.restart(), backoffOption)
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
  private async onNewBlock(blockNumber: number, fetchEvents = false, fetchNativeTxs = false): Promise<void> {
    // NOTE: This function is also used in event handlers
    // where it cannot be 'awaited', so all exceptions need to be caught.

    // Don't process any block if indexer was stopped.
    if (![IndexerStatus.STARTING, IndexerStatus.STARTED].includes(this.status)) {
      return
    }

    // Set a lock during block processing to make sure database does not get closed
    if (this.blockProcessingLock) {
      this.blockProcessingLock.resolve()
    }
    this.blockProcessingLock = defer<void>()

    const currentBlock = blockNumber - this.maxConfirmations

    if (currentBlock < 0) {
      return
    }

    log('Indexer got new block %d, handling block %d', blockNumber, blockNumber - this.maxConfirmations)
    this.emit('block', blockNumber)

    // update latest block
    this.latestBlock = Math.max(this.latestBlock, blockNumber)
    metric_blockNumber.set(this.latestBlock)

    let lastDatabaseSnapshot = await this.db.get_latest_confirmed_snapshot()

    // settle transactions before processing events
    if (fetchNativeTxs) {
      // get the number of unconfirmed (pending and mined) transactions tracked by the transaction manager
      const unconfirmedTxListeners = this.chain.getAllUnconfirmedHash()
      // only request transactions in block when transaction manager is tracking
      log('Indexer fetches native txs for %d unconfirmed tx listeners', unconfirmedTxListeners.length)
      if (unconfirmedTxListeners.length > 0) {
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
            } else if (this.listeners(`node-safe-registered-${txHash}`).length > 0) {
              this.indexEvent(`node-safe-registered-${txHash}`)
            } else if (this.listeners(`token-approved-${txHash}`).length > 0) {
              this.indexEvent(`token-approved-${txHash}`)
            }

            // update transaction manager
            this.chain.updateConfirmedTransaction(txHash)
          }
        }
      }
    }

    if (fetchEvents) {
      // Don't fail immediately when one block is temporarily not available
      const RETRIES = 3
      let res: Awaited<ReturnType<Indexer['getEvents']>>

      for (let i = 0; i < RETRIES; i++) {
        log(
          `fetchEvents at currentBlock ${currentBlock} startupBlock ${this.startupBlock} maxConfirmations ${
            this.maxConfirmations
          }. ${currentBlock > this.startupBlock + this.maxConfirmations}`
        )
        if (currentBlock > this.startupBlock) {
          // between starting block "Latest on-chain block" and finality + 1 to prevent double processing of events in blocks ["Latest on-chain block" - maxConfirmations, "Latest on-chain block"]
          res = await this.getEvents(currentBlock, currentBlock, true)
        } else {
          res = await this.getEvents(currentBlock, currentBlock, false)
        }

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

    try {
      await this.processUnconfirmedEvents(blockNumber, lastDatabaseSnapshot)
    } catch (err) {
      log(`error while processing unconfirmed events`, err)
    }

    // resend queuing transactions, when there are transactions (in queue) that haven't been accepted by the RPC
    // and resend transactions if the current balance is sufficient.

    const allQueuingTxs = this.chain.getAllQueuingTransactionRequests()
    if (allQueuingTxs.length > 0) {
      const minimumBalanceForQueuingTxs = allQueuingTxs.reduce(
        (acc, queuingTx) =>
          // to get the minimum balance required to resend a queuing transaction,
          // use the gasLimit (that shouldn't change, unless the contract state is different)
          // multiplies the maxFeePerGas of the queuing transaction
          acc.add(new BN(queuingTx.gasLimit.toString()).mul(new BN(queuingTx.maxFeePerGas.toString()))),
        new BN(0)
      )
      const currentBalance = await this.chain.getNativeBalance(this.address)
      if (
        // compare the current balance with the minimum balance required at the time of transaction being queued.
        // NB: Both gasLimit and maxFeePerGas requirement may be different due to "drastic" changes in contract state and network condition
        new BN(currentBalance.amount().to_string()).gte(minimumBalanceForQueuingTxs)
      ) {
        try {
          await Promise.all(
            allQueuingTxs.map((request) => {
              // convert TransactionRequest to signed transaction and send out
              return this.chain.sendTransaction(
                true,
                { to: request.to, value: request.value as BigNumber, data: request.data as string },
                (txHash: string) => this.resolvePendingTransaction(`on-new-block-${txHash}`, txHash)
              )
            })
          )
        } catch (err) {
          log(`error: failed to send queuing transaction on new block`, err)
        }
      }
    }

    try {
      await this.db.update_latest_block_number(blockNumber)
    } catch (err) {
      log(`error: failed to update database with latest block number ${blockNumber}`, err)
    }

    this.blockProcessingLock.resolve()

    this.emit('block-processed', currentBlock)
  }

  /**
   * Adds new events to the queue of unprocessed events
   * @dev ignores events that have been processed before.
   * @param events new unprocessed events
   */
  private onNewEvents(events: Log[] | undefined): void {
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
        'Processing event at blockNumber=%s maxConfirmations=%s',
        // @TODO: fix type clash
        blockNumber,
        this.maxConfirmations
      )

      // if we find a previous snapshot, compare event's snapshot with last processed
      if (lastDatabaseSnapshot) {
        const lastSnapshotComparison = snapshotComparator(event, {
          blockNumber: lastDatabaseSnapshot.block_number.as_u32(),
          logIndex: lastDatabaseSnapshot.log_index.as_u32(),
          transactionIndex: lastDatabaseSnapshot.transaction_index.as_u32()
        })

        // check if this is a duplicate or older than last snapshot
        // ideally we would have detected if this snapshot was indeed processed,
        // at the moment we don't keep all events stored as we intend to keep
        // this indexer very simple
        if (lastSnapshotComparison <= 0) {
          log(`Skipping event, lastSnapshotComparison=${lastSnapshotComparison}`)
          continue
        }
      }

      // @TODO: fix type clash
      lastDatabaseSnapshot = new Snapshot(
        new U256(event.blockNumber.toString()),
        new U256(event.transactionIndex.toString()),
        new U256(event.logIndex.toString())
      )

      log('indexer on_event callback for transaction hash: ', event.transactionHash)

      try {
        await this.handlers.on_event(
          this.db,
          event.address.replace('0x', ''),
          event.topics.map((t) => t.replace('0x', '')),
          event.data.replace('0x', ''),
          blockNumber.toString(),
          lastDatabaseSnapshot
        )
      } catch (err) {
        error('Error during indexer on_event callback: ', err, event)
      }
    }
  }

  onAnnouncementUpdate(account: AccountEntry) {
    this.emit('peer', {
      id: peerIdFromString(account.public_key.to_peerid_str()),
      multiaddrs: [new Multiaddr(account.get_multiaddr_str())]
    })
  }

  onOwnChannelUpdated(channel: ChannelEntry) {
    this.emit('own-channel-updated', channel)
  }

  onNotAllowedToAccessNetwork(address: Address) {
    this.emit('network-registry-eligibility-changed', address, false)
  }

  /**
   * TODO: event update
   */
  // private async onTicketRedeemed(event: Event<'TicketRedeemed'>, lastSnapshot: Snapshot) {
  //   if (Address.from_string(event.args.source).eq(this.address)) {
  //     // the node used to lock outstandingTicketBalance
  //     // rebuild part of the Ticket
  //     const partialTicket: Partial<Ticket> = {
  //       counterparty: Address.from_string(event.args.destination),
  //       amount: new Balance(event.args.amount.toString(), BalanceType.HOPR)
  //     }
  //     const outstandingBalance = Balance.deserialize(
  //       (
  //         await this.db.get_pending_balance_to(Address.deserialize(partialTicket.counterparty.serialize()))
  //       ).serialize_value(),
  //       BalanceType.HOPR
  //     )

  //     assert(lastSnapshot !== undefined)
  //     try {
  //       // Negative case:
  //       // It falls into this case when db of sender gets erased while having tickets pending.
  //       // TODO: handle this may allow sender to send arbitrary amount of tickets through open
  //       // channels with positive balance, before the counterparty initiates closure.
  //       const balance = outstandingBalance.lte(Balance.zero(BalanceType.HOPR))
  //         ? Balance.zero(BalanceType.HOPR)
  //         : outstandingBalance
  //       await this.db.resolve_pending(
  //         Address.deserialize(partialTicket.counterparty.serialize()),
  //         Balance.deserialize(balance.serialize_value(), BalanceType.HOPR),
  //         Snapshot.deserialize(lastSnapshot.serialize())
  //       )
  //       metric_ticketsRedeemed.increment()
  //     } catch (error) {
  //       log(`error in onTicketRedeemed ${error}`)
  //       throw new Error(`error in onTicketRedeemed ${error}`)
  //     }
  //   }
  // }

  private indexEvent(indexerEvent: IndexerEvents) {
    log(`Indexer indexEvent ${indexerEvent}`)
    this.emit(indexerEvent)
  }

  public async getAccount(address: Address): Promise<AccountEntry | undefined> {
    let account = await this.db.get_account(address)
    if (account !== undefined) {
      return AccountEntry.deserialize(account.serialize())
    }

    return account
  }

  public async getChainKeyOf(address: Address): Promise<Address> {
    const account = await this.getAccount(address)
    if (account !== undefined) {
      return account.chain_addr
    }
    throw new Error('Could not find chain key for address - have they announced? -' + address.to_hex())
  }

  public async getPacketKeyOf(address: Address): Promise<OffchainPublicKey> {
    const pk = await this.db.get_packet_key(address)
    if (pk !== undefined) {
      return pk
    }
    throw new Error('Could not find packet key for address - have they announced? -' + address.to_hex())
  }

  public async *getAddressesAnnouncedOnChain() {
    let announced = await this.db.get_accounts()
    while (announced.len() > 0) {
      yield new Multiaddr(announced.next().get_multiaddr_str())
    }
  }

  public async getPublicNodes(): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }[]> {
    const result: { id: PeerId; multiaddrs: Multiaddr[] }[] = []
    let out = `Known public nodes:\n`

    let publicAccounts = await this.db.get_public_node_accounts()

    while (publicAccounts.len() > 0) {
      let account = publicAccounts.next()
      if (account) {
        let packetKey = await this.db.get_packet_key(account.chain_addr)
        if (packetKey) {
          out += `  - ${packetKey.to_peerid_str()} (on-chain ${account.chain_addr.to_string()}) ${account.get_multiaddr_str()}\n`
          result.push({
            id: peerIdFromString(packetKey.to_peerid_str()),
            multiaddrs: [new Multiaddr(account.get_multiaddr_str())]
          })
        } else {
          log(`could not retrieve packet key for address ${account.chain_addr.to_string()}`)
        }
      }
    }

    // Remove last `\n`
    log(out.substring(0, out.length - 1))

    return result
  }

  /**
   * Returns a random open channel.
   * NOTE: channels with status 'PENDING_TO_CLOSE' are not included
   * @returns an open channel
   */
  public async getRandomOpenChannel(): Promise<ChannelEntry | undefined> {
    const channels = await this.db.get_channels_open()

    if (channels.len() == 0) {
      log('no open channels exist in indexer')
      return undefined
    }

    return ChannelEntry.deserialize(channels.at(random_integer(0, channels.len())).serialize())
  }

  /**
   * Returns peer's open channels.
   * NOTE: channels with status 'PENDING_TO_CLOSE' are not included
   * @param source peer
   * @returns peer's open channels
   */
  public async getOpenChannelsFrom(source: Address): Promise<ChannelEntry[]> {
    let allChannels = await this.db.get_channels_from(source)
    let channels: ChannelEntry[] = []
    while (allChannels.len() > 0) {
      channels.push(ChannelEntry.deserialize(allChannels.next().serialize()))
    }
    return channels.filter((channel) => channel.status === ChannelStatus.Open)
  }

  public resolvePendingTransaction(eventType: IndexerEvents, tx: string): DeferType<string> {
    const deferred = {} as DeferType<string>

    deferred.promise = new Promise<string>((resolve, reject) => {
      let done = false
      let timer: any

      deferred.reject = () => {
        if (done) {
          return
        }
        timer?.clear()
        done = true

        this.removeListener(eventType, deferred.resolve)
        log('listener %s on %s is removed due to error', eventType, tx)
        setImmediate(resolve, tx)
      }

      timer = retimer(
        () => {
          if (done) {
            return
          }
          timer?.clear()
          done = true
          // remove listener but throw now error
          this.removeListener(eventType, deferred.resolve)
          log('listener %s on %s timed out and thus removed', eventType, tx)
          setImmediate(reject, tx)
        },
        constants.INDEXER_TIMEOUT,
        `Timeout while indexer waiting for confirming transaction ${tx}`
      )

      deferred.resolve = () => {
        if (done) {
          return
        }
        timer?.clear()
        done = true

        this.removeListener(eventType, deferred.resolve)
        log('listener %s on %s is resolved and thus removed', eventType, tx)

        setImmediate(resolve, tx)
      }

      this.addListener(eventType, deferred.resolve)
      log('listener %s on %s is added', eventType, tx)
    })

    return deferred
  }
}

export default Indexer
