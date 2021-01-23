import type { Subscription } from 'web3-core-subscriptions'
import type { BlockHeader } from 'web3-eth'
import type { Log } from 'web3-core'
import type PeerId from 'peer-id'
import type { Indexer as IIndexer, RoutingChannel, ChannelUpdate } from '@hoprnet/hopr-core-connector-interface'
import type HoprEthereum from '..'
import EventEmitter from 'events'
import chalk from 'chalk'
import BN from 'bn.js'
import Heap from 'heap-js'
import { pubKeyToPeerId, randomChoice } from '@hoprnet/hopr-utils'
import { ChannelEntry, Public, Balance } from '../types'
import { isPartyA, getId, Log as DebugLog } from '../utils'
import { MAX_CONFIRMATIONS } from '../config'
import {
  isConfirmedBlock,
  getLatestBlockNumber,
  getChannelEntry,
  updateChannelEntry,
  getChannelEntries,
  updateLatestBlockNumber,
  eventComparator
} from './utils'
import * as topics from './topics'
import type { Event, EventData } from './topics'

const log = DebugLog(['indexer'])

type Subscriptions = 'NewBlock' | keyof EventData

/**
 * Simple indexer to keep track of all open payment channels.
 */
class Indexer extends EventEmitter implements IIndexer {
  private status: 'started' | 'stopped' = 'stopped'
  private unconfirmedEvents = new Heap<Event<any>>(eventComparator)
  private subscriptions: {
    [K in Subscriptions]?: Subscription<any>
  } = {}

  // latest known on-chain block number
  public latestBlock: number = 0

  constructor(private connector: HoprEthereum) {
    super()
  }

  /**
   * Starts indexing.
   *
   * @returns true if start was succesful
   */
  public async start(): Promise<void> {
    if (this.status === 'started') return
    log(`Starting indexer...`)

    const { web3, hoprChannels } = this.connector

    // go back 'MAX_CONFIRMATIONS' blocks in case of a re-org at time of stopping
    let fromBlock = await getLatestBlockNumber(this.connector)
    if (fromBlock - MAX_CONFIRMATIONS > 0) {
      fromBlock = fromBlock - MAX_CONFIRMATIONS
    }

    log('Starting to index from block %d', fromBlock)

    // subscribe to events
    // @TODO: when we refactor this needs to be more generic

    this.subscriptions['NewBlock'] = web3.eth.subscribe('newBlockHeaders').on('data', (block) => this.onNewBlock(block))

    this.subscriptions['FundedChannel'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: topics.generateTopics(topics.EventSignatures.FundedChannel, undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onFundedChannel(topics.decodeFundedChannel(log))
      })

    this.subscriptions['OpenedChannel'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: topics.generateTopics(topics.EventSignatures.OpenedChannel, undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onOpenedChannel(topics.decodeOpenedChannel(log))
      })

    this.subscriptions['RedeemedTicket'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: topics.generateTopics(topics.EventSignatures.RedeemedTicket, undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onRedeemedTicket(topics.decodeRedeemedTicket(log))
      })

    this.subscriptions['InitiatedChannelClosure'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: topics.generateTopics(topics.EventSignatures.InitiatedChannelClosure, undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onInitiatedChannelClosure(topics.decodeInitiatedChannelClosure(log))
      })

    this.subscriptions['ClosedChannel'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: topics.generateTopics(topics.EventSignatures.ClosedChannel, undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onClosedChannel(topics.decodeClosedChannel(log))
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

    for (const subscription of Object.values(this.subscriptions)) {
      subscription.unsubscribe()
    }

    this.status = 'stopped'
    log(chalk.green('Indexer stopped!'))
  }

  public async getChannelEntry(partyA: Public, partyB: Public): Promise<ChannelEntry | undefined> {
    return getChannelEntry(this.connector, partyA, partyB)
  }

  public async getChannelEntries(party: Public): Promise<ChannelUpdate[]> {
    return getChannelEntries(this.connector, party)
  }

  private async onNewBlock(block: BlockHeader) {
    log('New block %d', block.number)

    // update latest block
    if (this.latestBlock < block.number) {
      this.latestBlock = block.number
    }

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    while (
      this.unconfirmedEvents.length > 0 &&
      isConfirmedBlock(this.unconfirmedEvents.top(1)[0].blockNumber.toNumber(), block.number)
    ) {
      const event = this.unconfirmedEvents.pop()

      if (event.name === 'FundedChannel') {
        this.onFundedChannel(event as Event<'FundedChannel'>)
      } else if (event.name === 'OpenedChannel') {
        this.onOpenedChannel(event as Event<'OpenedChannel'>)
      } else if (event.name === 'RedeemedTicket') {
        this.onRedeemedTicket(event as Event<'RedeemedTicket'>)
      } else if (event.name === 'InitiatedChannelClosure') {
        this.onInitiatedChannelClosure(event as Event<'InitiatedChannelClosure'>)
      } else if (event.name === 'ClosedChannel') {
        this.onClosedChannel(event as Event<'ClosedChannel'>)
      }
    }

    await updateLatestBlockNumber(this.connector, new BN(block.number))
  }

  private ignoreAndStoreUnconfirmed(event: Event<any>): boolean {
    if (!isConfirmedBlock(event.blockNumber.toNumber(), this.latestBlock)) {
      this.unconfirmedEvents.push(event)
      return true
    }

    return false
  }

  // private ignoreAlreadyProcessed(eventA: Event<any>, eventB: Event<any>): boolean {
  //   return true
  // }

  private async onFundedChannel(event: Event<'FundedChannel'>): Promise<void> {
    if (this.ignoreAndStoreUnconfirmed(event)) return

    const { isPartyA } = this.connector.utils
    const storedChannel = await getChannelEntry(this.connector, event.data.recipient, event.data.counterparty)
    const recipientAccountId = await event.data.recipient.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isRecipientPartyA = isPartyA(recipientAccountId, counterpartyAccountId)

    let channelEntry: ChannelEntry

    if (storedChannel) {
      channelEntry = new ChannelEntry(undefined, {
        blockNumber: event.blockNumber,
        transactionIndex: event.transactionIndex,
        logIndex: event.logIndex,
        deposit: storedChannel.deposit.add(event.data.recipientAmount.add(event.data.counterpartyAmount)),
        partyABalance: storedChannel.partyABalance.add(
          isRecipientPartyA ? event.data.recipientAmount : event.data.counterpartyAmount
        ),
        closureTime: new BN(0),
        stateCounter: storedChannel.stateCounter.addn(1),
        closureByPartyA: false
      })
    } else {
      channelEntry = new ChannelEntry(undefined, {
        blockNumber: event.blockNumber,
        transactionIndex: event.transactionIndex,
        logIndex: event.logIndex,
        deposit: event.data.recipientAmount.add(event.data.counterpartyAmount),
        partyABalance: isRecipientPartyA ? event.data.recipientAmount : event.data.counterpartyAmount,
        closureTime: new BN(0),
        stateCounter: storedChannel.stateCounter.addn(1),
        closureByPartyA: false
      })
    }

    await updateChannelEntry(
      this.connector,
      isRecipientPartyA ? event.data.recipient : event.data.counterparty,
      isRecipientPartyA ? event.data.counterparty : event.data.recipient,
      channelEntry
    )

    log(
      'Channel %s got funded by %s',
      chalk.green(await getId(recipientAccountId, counterpartyAccountId)),
      chalk.green(event.data.funder)
    )
  }

  private async onOpenedChannel(event: Event<'OpenedChannel'>): Promise<void> {
    if (this.ignoreAndStoreUnconfirmed(event)) return

    const storedChannel = await getChannelEntry(this.connector, event.data.opener, event.data.counterparty)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel!'))
      return
    }

    const openerAccountId = await event.data.opener.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isOpenerPartyA = isPartyA(openerAccountId, counterpartyAccountId)
    const partyA = isOpenerPartyA ? event.data.opener : event.data.counterparty
    const partyB = isOpenerPartyA ? event.data.counterparty : event.data.opener

    const channelEntry = new ChannelEntry(undefined, {
      blockNumber: event.blockNumber,
      transactionIndex: event.transactionIndex,
      logIndex: event.logIndex,
      deposit: storedChannel.deposit,
      partyABalance: storedChannel.partyABalance,
      closureTime: storedChannel.closureTime,
      stateCounter: storedChannel.stateCounter.addn(1),
      closureByPartyA: false
    })

    await updateChannelEntry(this.connector, partyA, partyB, channelEntry)

    this.emit('channelOpened', {
      partyA,
      partyB,
      channelEntry
    })

    log(
      'Channel %s got opened by %s',
      chalk.green(await getId(openerAccountId, counterpartyAccountId)),
      chalk.green(openerAccountId.toHex())
    )
  }

  private async onRedeemedTicket(event: Event<'RedeemedTicket'>): Promise<void> {
    if (this.ignoreAndStoreUnconfirmed(event)) return

    const storedChannel = await getChannelEntry(this.connector, event.data.redeemer, event.data.counterparty)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel!'))
      return
    }

    const redeemerAccountId = await event.data.redeemer.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isRedeemerPartyA = isPartyA(redeemerAccountId, counterpartyAccountId)

    const channelEntry = new ChannelEntry(undefined, {
      blockNumber: event.blockNumber,
      transactionIndex: event.transactionIndex,
      logIndex: event.logIndex,
      deposit: storedChannel.deposit,
      partyABalance: isRedeemerPartyA
        ? storedChannel.partyABalance.add(event.data.amount)
        : storedChannel.partyABalance.sub(event.data.amount),
      closureTime: storedChannel.closureTime,
      stateCounter: storedChannel.stateCounter,
      closureByPartyA: false
    })

    await updateChannelEntry(
      this.connector,
      isRedeemerPartyA ? event.data.redeemer : event.data.counterparty,
      isRedeemerPartyA ? event.data.counterparty : event.data.redeemer,
      channelEntry
    )

    log(
      'Ticket redeemd in channel %s by %s',
      chalk.green(await getId(redeemerAccountId, counterpartyAccountId)),
      chalk.green(redeemerAccountId.toHex())
    )
  }

  private async onInitiatedChannelClosure(event: Event<'InitiatedChannelClosure'>): Promise<void> {
    if (this.ignoreAndStoreUnconfirmed(event)) return

    const storedChannel = await getChannelEntry(this.connector, event.data.initiator, event.data.counterparty)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel!'))
      return
    }

    const initiatorAccountId = await event.data.initiator.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isInitiatorPartyA = isPartyA(initiatorAccountId, counterpartyAccountId)

    const channelEntry = new ChannelEntry(undefined, {
      blockNumber: event.blockNumber,
      transactionIndex: event.transactionIndex,
      logIndex: event.logIndex,
      deposit: storedChannel.deposit,
      partyABalance: storedChannel.partyABalance,
      closureTime: event.data.closureTime,
      stateCounter: storedChannel.stateCounter.addn(1),
      closureByPartyA: isInitiatorPartyA
    })

    await updateChannelEntry(
      this.connector,
      isInitiatorPartyA ? event.data.initiator : event.data.counterparty,
      isInitiatorPartyA ? event.data.counterparty : event.data.initiator,
      channelEntry
    )

    log(
      'Channel closure initiated for %s by %s',
      chalk.green(await getId(initiatorAccountId, counterpartyAccountId)),
      chalk.green(initiatorAccountId.toHex())
    )
  }

  private async onClosedChannel(event: Event<'ClosedChannel'>): Promise<void> {
    if (this.ignoreAndStoreUnconfirmed(event)) return

    const storedChannel = await getChannelEntry(this.connector, event.data.closer, event.data.counterparty)
    if (!storedChannel) {
      log(chalk.red('Could not find stored channel!'))
      return
    }

    const closerAccountId = await event.data.closer.toAccountId()
    const counterpartyAccountId = await event.data.counterparty.toAccountId()
    const isCloserPartyA = isPartyA(closerAccountId, counterpartyAccountId)
    const partyA = isCloserPartyA ? event.data.closer : event.data.counterparty
    const partyB = isCloserPartyA ? event.data.counterparty : event.data.closer

    const channelEntry = new ChannelEntry(undefined, {
      blockNumber: event.blockNumber,
      transactionIndex: event.transactionIndex,
      logIndex: event.logIndex,
      deposit: new BN(0),
      partyABalance: new BN(0),
      closureTime: new BN(0),
      stateCounter: storedChannel.stateCounter.addn(1),
      closureByPartyA: false
    })

    await updateChannelEntry(this.connector, partyA, partyB, channelEntry)

    this.emit('channelClosed', {
      partyA,
      partyB,
      channelEntry
    })

    log(
      'Channel %s got closed by %s',
      chalk.green(await getId(closerAccountId, counterpartyAccountId)),
      chalk.green(closerAccountId.toHex())
    )
  }

  // routing related
  // @TODO: refactor
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
    const results = await getChannelEntries(this.connector)
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
    const channels = await getChannelEntries(this.connector, sourcePubKey)
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
