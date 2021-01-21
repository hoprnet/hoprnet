import type { Indexer as IIndexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import type HoprEthereum from '..'
import BN from 'bn.js'
import chalk from 'chalk'
import { Subscription } from 'web3-core-subscriptions'
import { BlockHeader } from 'web3-eth'
import { u8aToNumber, u8aConcat, u8aToHex, pubKeyToPeerId, randomChoice } from '@hoprnet/hopr-utils'
import { ChannelEntry, Public, Balance } from '../types'
import { isPartyA, events, getId, pubKeyToAccountId } from '../utils'
import { MAX_CONFIRMATIONS } from '../config'
import { ContractEventLog } from '../tsc/web3/types'
import { Log as OnChainLog } from 'web3-core'
import PeerId from 'peer-id'
import Heap from 'heap-js'
import debug from 'debug'

const log = debug('hopr:indexer')

// we save up some memory by only caching the event data we use
type LightEvent<E extends ContractEventLog<any>> = Pick<
  E,
  'event' | 'blockNumber' | 'transactionHash' | 'transactionIndex' | 'logIndex' | 'returnValues'
>
type Channel = { partyA: Public; partyB: Public; channelEntry: ChannelEntry }
export type OpenedChannelEvent = LightEvent<ContractEventLog<{ opener: Public; counterparty: Public }>>
export type ClosedChannelEvent = LightEvent<
  ContractEventLog<{ closer: Public; counterparty: Public; partyAAmount?: BN; partyBAmount?: BN }>
>

const SMALLEST_PUBLIC_KEY = new Public(u8aConcat(new Uint8Array([0x02]), new Uint8Array(32).fill(0x00)))
const BIGGEST_PUBLIC_KEY = new Public(u8aConcat(new Uint8Array([0x03]), new Uint8Array(32).fill(0xff)))

/**
 * Returns true if 'newChannelEntry' is more recent.
 *
 * @param oldChannelEntry
 * @param newChannelEntry
 * @returns true if 'newChannelEntry' is more recent than 'oldChannelEntry'
 */
function isMoreRecent(oldChannelEntry: ChannelEntry, newChannelEntry: ChannelEntry): boolean {
  const okBlockNumber = oldChannelEntry.blockNumber.lte(newChannelEntry.blockNumber)
  const okTransactionIndex = okBlockNumber && oldChannelEntry.transactionIndex.lte(newChannelEntry.transactionIndex)
  const okLogIndex = okTransactionIndex && oldChannelEntry.logIndex.lt(newChannelEntry.logIndex)

  return okBlockNumber && okTransactionIndex && okLogIndex
}

/**
 * @returns true if blockNumber has passed max confirmations
 */
function isConfirmedBlock(blockNumber: number, onChainBlockNumber: number): boolean {
  return blockNumber + MAX_CONFIRMATIONS <= onChainBlockNumber
}

/**
 * Simple indexer to keep track of all open payment channels.
 */
class Indexer implements IIndexer {
  private status: 'started' | 'stopped' = 'stopped'
  private unconfirmedEvents: (OpenedChannelEvent | ClosedChannelEvent)[] = []
  private starting?: Promise<boolean>
  private stopping?: Promise<boolean>
  private newBlockEvent?: Subscription<BlockHeader>
  private openedChannelEvent?: Subscription<OnChainLog>
  private closedChannelEvent?: Subscription<OnChainLog>
  private newChannelHandler: (newChannels: IndexerChannel[]) => void

  // latest known on-chain block number
  public latestBlock: number = 0

  constructor(private connector: HoprEthereum) {
    this.newChannelHandler = () => {}
  }

  /**
   * Returns the latest confirmed block number.
   *
   * @returns promive that resolves to a number
   */
  private async getLatestConfirmedBlockNumber(): Promise<number> {
    try {
      return u8aToNumber(
        (await this.connector.db.get(Buffer.from(this.connector.dbKeys.ConfirmedBlockNumber()))) as Uint8Array
      ) as number
    } catch (err) {
      if (err.notFound == null) {
        throw err
      }

      return 0
    }
  }

  private async toIndexerChannel(source: PeerId, channel: Channel): Promise<IndexerChannel> {
    const sourcePubKey = new Public(source.pubKey.marshal())
    const channelId = await getId(await pubKeyToAccountId(channel.partyA), await pubKeyToAccountId(channel.partyB))
    const state = await this.connector.hoprChannels.methods.channels(channelId.toHex()).call()
    if (sourcePubKey.eq(channel.partyA)) {
      return [source, await pubKeyToPeerId(channel.partyB), new Balance(state.partyABalance)]
    } else {
      const partyBBalance = new Balance(state.deposit).sub(new Balance(state.partyABalance))
      return [source, await pubKeyToPeerId(channel.partyA), partyBBalance]
    }
  }

  public onNewChannels(handler: (newChannels: IndexerChannel[]) => void): void {
    this.newChannelHandler = handler
  }

  public async getRandomChannel(): Promise<IndexerChannel | undefined> {
    //const HACK = 3970950 // Arbitrarily chosen block for our testnet. Total hack.
    const all = await this.getAll(undefined)
    const filtered = all //.filter((x) => x.channelEntry.blockNumber.gtn(HACK))

    if (filtered.length === 0) {
      log('no channels exist in indexer > hack')
      return undefined
    }
    log('picking random from ', filtered.length, ' channels')
    const random = randomChoice(filtered)
    return this.toIndexerChannel(await pubKeyToPeerId(random.partyA), random)
  }

  public async getChannelsFromPeer(source: PeerId): Promise<IndexerChannel[]> {
    const sourcePubKey = new Public(source.pubKey.marshal())
    const channels = await this.getAll(sourcePubKey)
    let cout: IndexerChannel[] = []
    for (let channel of channels) {
      let directed = await this.toIndexerChannel(source, channel)
      if (directed[2].gtn(0)) {
        cout.push(directed)
      }
    }

    return cout
  }

  /**
   * Check if channel entry exists.
   *
   * @returns promise that resolves to true or false
  private async has(partyA: Public, partyB: Public): Promise<boolean> {
    const { dbKeys, db } = this.connector

    try {
      await db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB)))
    } catch (err) {
      if (err.notFound) {
        return false
      }
    }

    return true
  }
   */

  /**
   * Get all stored channel entries, if party is provided,
   * it will return the open channels of the given party.
   *
   * @returns promise that resolves to a list of channel entries
   */
  private async getAll(party: Public | undefined, filter?: (node: Public) => boolean): Promise<Channel[]> {
    const { dbKeys, db } = this.connector
    const channels: Channel[] = []

    return await new Promise<Channel[]>((resolve, reject) => {
      db.createReadStream({
        gte: Buffer.from(dbKeys.ChannelEntry(SMALLEST_PUBLIC_KEY, SMALLEST_PUBLIC_KEY)),
        lte: Buffer.from(dbKeys.ChannelEntry(BIGGEST_PUBLIC_KEY, BIGGEST_PUBLIC_KEY))
      })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const [partyA, partyB] = dbKeys.ChannelEntryParse(key)

          if (
            (party != null && !(party.eq(partyA) || party.eq(partyB))) ||
            (filter != null && !(filter(partyA) && filter(partyB)))
          ) {
            return
          }

          const channelEntry = new ChannelEntry({
            bytes: value,
            offset: value.byteOffset
          })

          channels.push({
            partyA,
            partyB,
            channelEntry
          })
        })
        .on('end', () => resolve(channels))
    })
  }

  /**
   * Get stored channel of the given parties.
   *
   * @returns promise that resolves to a channel entry or undefined if not found
   */
  private async getSingle(partyA: Public, partyB: Public): Promise<Channel | undefined> {
    const { dbKeys, db } = this.connector

    let _entry: Uint8Array | undefined
    try {
      _entry = (await db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB)))) as Uint8Array
    } catch (err) {
      if (err.notFound) {
        return
      }
    }

    if (_entry == null || _entry.length == 0) {
      return
    }

    const channelEntry = new ChannelEntry({
      bytes: _entry,
      offset: _entry.byteOffset
    })

    return {
      partyA,
      partyB,
      channelEntry
    }
  }

  /**
   * Get stored channels entries.
   *
   * If query is left empty, it will return all channels.
   *
   * If only one party is provided, it will return all channels of the given party.
   *
   * If both parties are provided, it will return the channel of the given party.
   *
   * @param query
   * @returns promise that resolves to a list of channel entries
   */
  public async get(
    query?: { partyA?: Public; partyB?: Public },
    filter?: (node: Public) => boolean
  ): Promise<Channel[]> {
    if (query == null) {
      // query not provided, get all channels
      return this.getAll(undefined, filter)
    } else if (query.partyA != null && query.partyB != null) {
      if (filter != null) {
        throw Error(`Applying a filter on precise queries is not supported.`)
      }
      // both parties provided, get channel
      const channel = await this.getSingle(query.partyA, query.partyB)

      return channel != null ? [channel] : []
    } else {
      // only one of the parties provided, get all open channels of party
      return this.getAll(query.partyA != null ? query.partyA : query.partyB, filter)
    }
  }

  private async store(partyA: Public, partyB: Public, channelEntry: ChannelEntry): Promise<void> {
    const { dbKeys, db } = this.connector
    const { blockNumber, logIndex, transactionIndex } = channelEntry

    log(
      `storing channel ${partyA.toHex()}-${partyB.toHex()}:${blockNumber.toString()}-${transactionIndex.toString()}-${logIndex.toString()}`
    )

    return await db.batch([
      {
        type: 'put',
        key: Buffer.from(dbKeys.ChannelEntry(partyA, partyB)),
        value: Buffer.from(channelEntry)
      },
      {
        type: 'put',
        key: Buffer.from(dbKeys.ConfirmedBlockNumber()),
        value: Buffer.from(blockNumber.toU8a())
      }
    ])
  }

  // delete a channel
  private async delete(partyA: Public, partyB: Public): Promise<void> {
    log(`deleting channel ${u8aToHex(partyA)}-${u8aToHex(partyB)}`)

    const { dbKeys, db } = this.connector

    const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB))

    return db.del(key)
  }

  private compareUnconfirmedEvents(
    a: OpenedChannelEvent | ClosedChannelEvent,
    b: OpenedChannelEvent | ClosedChannelEvent
  ): number {
    return a.blockNumber - b.blockNumber
  }

  private async onNewBlock(block: BlockHeader) {
    if (this.latestBlock < block.number) {
      this.latestBlock = block.number
    }

    while (
      this.unconfirmedEvents.length > 0 &&
      isConfirmedBlock(
        Heap.heaptop(this.unconfirmedEvents, 1, this.compareUnconfirmedEvents)[0].blockNumber,
        block.number
      )
    ) {
      const event = Heap.heappop(this.unconfirmedEvents, this.compareUnconfirmedEvents) as
        | OpenedChannelEvent
        | ClosedChannelEvent

      if (event.event === 'OpenedChannel') {
        this.onOpenedChannel(event as OpenedChannelEvent)
      } else {
        this.onClosedChannel(event as ClosedChannelEvent)
      }
    }
  }

  private async onOpenedChannel(event: OpenedChannelEvent): Promise<void> {
    let partyA: Public, partyB: Public

    if (isPartyA(await event.returnValues.opener.toAccountId(), await event.returnValues.counterparty.toAccountId())) {
      partyA = event.returnValues.opener
      partyB = event.returnValues.counterparty
    } else {
      partyA = event.returnValues.counterparty
      partyB = event.returnValues.opener
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

  private async onClosedChannel(event: ClosedChannelEvent): Promise<void> {
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

  /**
   * Start indexing,
   * listen to all open / close events,
   * store entries after X confirmations.
   *
   * @returns true if start was succesful
   */
  public async start(): Promise<boolean> {
    log(`Starting indexer...`)

    if (typeof this.starting !== 'undefined') {
      return this.starting
    } else if (typeof this.stopping !== 'undefined') {
      throw Error('cannot start while stopping')
    } else if (this.status === 'started') {
      return true
    }

    this.starting = new Promise<boolean>(async (resolve, reject) => {
      let rejected = false
      try {
        const onChainBlockNumber = await this.connector.web3.eth.getBlockNumber()
        let fromBlock = await this.getLatestConfirmedBlockNumber()

        // go back 8 blocks in case of a re-org at time of stopping
        if (fromBlock - MAX_CONFIRMATIONS > 0) {
          fromBlock = fromBlock - MAX_CONFIRMATIONS
        }

        log(`starting to pull events from block ${fromBlock}..`)

        this.newBlockEvent = this.connector.web3.eth
          .subscribe('newBlockHeaders')
          .on('error', (err) => {
            if (!rejected) {
              rejected = true
              reject(err)
            }
          })
          .on('data', (block) => this.onNewBlock(block))

        this.openedChannelEvent = this.connector.web3.eth
          .subscribe('logs', {
            address: this.connector.hoprChannels.options.address,
            fromBlock,
            topics: events.OpenedChannelTopics(undefined, undefined)
          })
          .on('error', (err) => {
            if (!rejected) {
              rejected = true
              reject(err)
            }
          })
          .on('data', (_event: OnChainLog) => this.processOpenedChannelEvent(_event, onChainBlockNumber))

        this.closedChannelEvent = this.connector.web3.eth
          .subscribe('logs', {
            address: this.connector.hoprChannels.options.address,
            fromBlock,
            topics: events.ClosedChannelTopics(undefined, undefined)
          })
          .on('error', (err) => {
            if (!rejected) {
              rejected = true
              reject(err)
            }
          })
          .on('data', (_event) => this.processClosedChannelEvent(_event, onChainBlockNumber))

        this.status = 'started'
        log(chalk.green('Indexer started!'))
        return resolve(true)
      } catch (err) {
        log(err.message)

        return this.stop()
      }
    }).finally(() => {
      this.starting = undefined
    })

    return this.starting
  }

  /**
   * Stop indexing.
   *
   * @returns true if stop was succesful
   */
  public async stop(): Promise<boolean> {
    log(`Stopping indexer...`)

    if (this.starting != null) {
      throw Error('cannot stop while starting')
    } else if (this.stopping != undefined) {
      return this.stopping
    } else if (this.status === 'stopped') {
      return true
    }

    this.stopping = new Promise<boolean>((resolve) => {
      try {
        this.newBlockEvent?.unsubscribe()
        this.openedChannelEvent?.unsubscribe()
        this.closedChannelEvent?.unsubscribe()

        this.status = 'stopped'
        log(chalk.green('Indexer stopped!'))
        return resolve(true)
      } catch (err) {
        log(err.message)

        return resolve(false)
      }
    }).finally(() => {
      this.stopping = undefined
    })

    return this.stopping
  }

  private processOpenedChannelEvent(_event: OnChainLog, onChainBlockNumber: number) {
    const event: OpenedChannelEvent = events.decodeOpenedChannelEvent(_event)

    if (isConfirmedBlock(event.blockNumber, onChainBlockNumber)) {
      this.onOpenedChannel(event)
    } else {
      // @TODO add membership with bloom filter to check before adding event to heap
      Heap.heappush(this.unconfirmedEvents, event, this.compareUnconfirmedEvents)
    }
  }

  private processClosedChannelEvent(_event: OnChainLog, onChainBlockNumber: number) {
    const event: ClosedChannelEvent = events.decodeClosedChannelEvent(_event)

    if (isConfirmedBlock(event.blockNumber, onChainBlockNumber)) {
      this.onClosedChannel(event)
    } else {
      // @TODO add membership with bloom filter to check before adding event to heap
      Heap.heappush(this.unconfirmedEvents, event, this.compareUnconfirmedEvents)
    }
  }
}

export default Indexer
