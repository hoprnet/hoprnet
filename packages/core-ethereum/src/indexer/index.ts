// import type { Indexer as IIndexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import type { Subscription } from 'web3-core-subscriptions'
import type { BlockHeader } from 'web3-eth'
import type { Log } from 'web3-core'
import type HoprEthereum from '..'
// import type { ContractEventLog } from '../tsc/web3/types'
import chalk from 'chalk'
import BN from 'bn.js'
import EventEmitter from 'events'
// import { u8aToNumber, u8aConcat, u8aToHex, pubKeyToPeerId, randomChoice } from '@hoprnet/hopr-utils'
import { ChannelEntry, Public, Balance } from '../types'
import { isPartyA, events, getId, pubKeyToAccountId, Log as DebugLog } from '../utils'
import { MAX_CONFIRMATIONS } from '../config'
// import PeerId from 'peer-id'
// import Heap from 'heap-js'
import { getLatestConfirmedBlockNumber, isConfirmedBlock, decodePublicKeysFromTopics } from './utils'
// import { HoprChannels } from '../tsc/web3/HoprChannels'
import type { Event, EventData } from './types'
import {
  decodeFundedChannel,
  decodeOpenedChannel,
  decodeRedeemedTicket,
  decodeInitiatedChannelClosure,
  decodeClosedChannel
} from './decodeLogs'

const log = DebugLog(['indexer'])

type Subscriptions = 'NewBlock' | keyof EventData

/**
 * Simple indexer to keep track of all open payment channels.
 */
class Indexer {
  private status: 'started' | 'stopped' = 'stopped'
  private eventEmitter = new EventEmitter()
  private unconfirmedEvents = new Map<string, Event<keyof EventData>>()
  private subscriptions: {
    [K in Subscriptions]?: Subscription<any>
  } = {}

  // latest known on-chain block number
  public latestBlock: number = 0

  constructor(private connector: HoprEthereum) {}

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
    let fromBlock = await getLatestConfirmedBlockNumber(this.connector)
    if (fromBlock - MAX_CONFIRMATIONS > 0) {
      fromBlock = fromBlock - MAX_CONFIRMATIONS
    }

    // subscribe to events
    // @TODO: when we refactor this needs to be more generic

    this.subscriptions['NewBlock'] = web3.eth.subscribe('newBlockHeaders').on('data', (block) => this.onNewBlock(block))

    this.subscriptions['FundedChannel'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: events.FundedChannelTopics(undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onFundedChannel(decodeFundedChannel(log))
      })

    this.subscriptions['OpenedChannel'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: events.OpenedChannelTopics(undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onOpenedChannel(decodeOpenedChannel(log))
      })

    this.subscriptions['RedeemedTicket'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: events.RedeemedTicketTopics(undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onRedeemedTicket(decodeRedeemedTicket(log))
      })

    this.subscriptions['InitiatedChannelClosure'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: events.InitiatedChannelClosure(undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onInitiatedChannelClosure(decodeInitiatedChannelClosure(log))
      })

    this.subscriptions['ClosedChannel'] = web3.eth
      .subscribe('logs', {
        address: hoprChannels.options.address,
        fromBlock,
        topics: events.ClosedChannelTopics(undefined, undefined)
      })
      .on('data', (log: Log) => {
        this.onClosedChannel(decodeClosedChannel(log))
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

  private async onNewBlock(block: BlockHeader) {
    // update latest block
    if (this.latestBlock < block.number) {
      this.latestBlock = block.number
    }

    // check unconfirmed events and process them if found
    // to be within a confirmed block
    // TODO: optimize
    for (const event of this.unconfirmedEvents.values()) {
      const isConfirmed = isConfirmedBlock(event.blockNumber, block.number)
      if (!isConfirmed) continue

      this.unconfirmedEvents.delete(event.transactionHash)

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
      } else {
        console.log('unsupported event')
      }
    }
  }

  private async onFundedChannel(event: Event<'FundedChannel'>): Promise<void> {
    if (!isConfirmedBlock(event.blockNumber, this.latestBlock)) {
      this.unconfirmedEvents.set(event.transactionHash, event)
      return
    }
  }

  private async onOpenedChannel(event: Event<'OpenedChannel'>): Promise<void> {
    if (!isConfirmedBlock(event.blockNumber, this.latestBlock)) {
      this.unconfirmedEvents.set(event.transactionHash, event)
      return
    }

    let partyA: Public, partyB: Public

    if (isPartyA(await event.data.opener.toAccountId(), await event.data.counterparty.toAccountId())) {
      partyA = event.data.opener
      partyB = event.data.counterparty
    } else {
      partyA = event.data.counterparty
      partyB = event.data.opener
    }

    const newChannelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(event.blockNumber),
      transactionIndex: new BN(event.transactionIndex),
      logIndex: new BN(event.logIndex)
    })

    const channels = await this.get({
      partyA,
      partyB
    })

    if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
      return
    }

    this.store(partyA, partyB, newChannelEntry)
    this.newChannelHandler([]) // TODO - pass new channels
  }

  private async onRedeemedTicket(event: Event<'RedeemedTicket'>): Promise<void> {
    if (!isConfirmedBlock(event.blockNumber, this.latestBlock)) {
      this.unconfirmedEvents.set(event.transactionHash, event)
      return
    }
  }

  private async onInitiatedChannelClosure(event: Event<'InitiatedChannelClosure'>): Promise<void> {
    if (!isConfirmedBlock(event.blockNumber, this.latestBlock)) {
      this.unconfirmedEvents.set(event.transactionHash, event)
      return
    }
  }

  private async onClosedChannel(event: Event<'ClosedChannel'>): Promise<void> {
    if (!isConfirmedBlock(event.blockNumber, this.latestBlock)) {
      this.unconfirmedEvents.set(event.transactionHash, event)
      return
    }

    let partyA: Public, partyB: Public

    if (isPartyA(await event.returnValues.closer.toAccountId(), await event.returnValues.counterparty.toAccountId())) {
      partyA = event.returnValues.closer
      partyB = event.returnValues.counterparty
    } else {
      partyA = event.returnValues.counterparty
      partyB = event.returnValues.closer
    }

    const newChannelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(event.blockNumber),
      transactionIndex: new BN(event.transactionIndex),
      logIndex: new BN(event.logIndex)
    })

    const channels = await this.get({
      partyA,
      partyB
    })

    if (channels.length === 0) {
      return
    } else if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
      return
    }

    await this.delete(partyA, partyB)
  }
}

export default Indexer
