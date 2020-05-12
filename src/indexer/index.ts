import type { Indexer as IIndexer } from '@hoprnet/hopr-core-connector-interface'
import type HoprEthereum from '..'
import BN from 'bn.js'
import chalk from 'chalk'
import { Subscription } from 'web3-core-subscriptions'
import { BlockHeader } from 'web3-eth'
import { u8aToNumber, stringToU8a } from '@hoprnet/hopr-utils'
import { AccountId, ChannelEntry } from '../types'
import { getParties, Log } from '../utils'
import { MAX_CONFIRMATIONS } from '../config'
import { ContractEventEmitter, ContractEventLog } from '../tsc/web3/types'

// we save up some memory by only caching the event data we use
type LightEvent<E extends ContractEventLog<any>> = Pick<
  E,
  'event' | 'blockNumber' | 'transactionHash' | 'transactionIndex' | 'logIndex' | 'returnValues'
>
type Channel = { partyA: AccountId; partyB: AccountId; channelEntry: ChannelEntry }
type OpenedChannelEvent = LightEvent<ContractEventLog<{ opener: string; counterParty: string }>>
type ClosedChannelEvent = LightEvent<ContractEventLog<{ closer: string; counterParty: string }>>

const SMALLEST_ACCOUNT = new Uint8Array(AccountId.SIZE).fill(0x00)
const BIGGEST_ACCOUNT = new Uint8Array(AccountId.SIZE).fill(0xff)

/**
 * @returns a custom event id for logging purposes.
 */
function getEventId(event: LightEvent<any>): string {
  return `${event.event}-${event.transactionHash}-${event.transactionIndex}-${event.logIndex}`
}

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
  private log = Log(['channels'])
  private status: 'started' | 'stopped' = 'stopped'
  private unconfirmedEvents = new Map<string, OpenedChannelEvent | ClosedChannelEvent>()
  private starting: Promise<any>
  private stopping: Promise<any>
  private newBlockEvent: Subscription<BlockHeader>
  private openedChannelEvent: ContractEventEmitter<any> | undefined
  private closedChannelEvent: ContractEventEmitter<any> | undefined

  constructor(private connector: HoprEthereum) {}

  /**
   * Returns the latest confirmed block number.
   *
   * @returns promive that resolves to a number
   */
  private async getLatestConfirmedBlockNumber(): Promise<number> {
    try {
      const blockNumber = await this.connector.db
        .get(Buffer.from(this.connector.dbKeys.ConfirmedBlockNumber()))
        .then((res) => {
          return u8aToNumber(res)
        })

      return blockNumber
    } catch (err) {
      if (err.notFound == null) {
        throw err
      }

      return 0
    }
  }

  /**
   * Check if channel entry exists.
   *
   * @returns promise that resolves to true or false
   */
  public async has(partyA: AccountId, partyB: AccountId): Promise<boolean> {
    const { dbKeys, db } = this.connector

    return db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB))).then(
      () => true,
      (err) => {
        if (err.notFound) {
          return false
        } else {
          throw err
        }
      }
    )
  }

  /**
   * Get all stored channel entries, if party is provided,
   * it will return the open channels of the given party.
   *
   * @returns promise that resolves to a list of channel entries
   */
  private async getAll(party?: AccountId): Promise<Channel[]> {
    const { dbKeys, db } = this.connector
    const channels: Channel[] = []

    return new Promise((resolve, reject) => {
      db.createReadStream({
        gte: Buffer.from(dbKeys.ChannelEntry(SMALLEST_ACCOUNT, SMALLEST_ACCOUNT)),
        lte: Buffer.from(dbKeys.ChannelEntry(BIGGEST_ACCOUNT, BIGGEST_ACCOUNT)),
      })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const [partyA, partyB] = dbKeys.ChannelEntryParse(key)
          if (party && !party.eq(partyA) && !party.eq(partyB)) {
            return
          }

          const channelEntry = new ChannelEntry({
            bytes: value,
            offset: value.byteOffset,
          })

          channels.push({
            partyA: new AccountId(partyA),
            partyB: new AccountId(partyB),
            channelEntry,
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
  private async getSingle(partyA: AccountId, partyB: AccountId): Promise<Channel | undefined> {
    const { dbKeys, db } = this.connector

    return db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB))).then(
      (value) => {
        const channelEntry = new ChannelEntry({
          bytes: value,
          offset: value.byteOffset,
        })

        return {
          partyA,
          partyB,
          channelEntry,
        }
      },
      (err) => {
        if (err.notFound) {
          return undefined
        } else {
          throw err
        }
      }
    )
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
  public async get(query?: { partyA?: AccountId; partyB?: AccountId }): Promise<Channel[]> {
    const hasQuery = typeof query !== 'undefined'
    const hasPartyA = hasQuery && typeof query.partyA !== 'undefined'
    const hasPartyB = hasQuery && typeof query.partyB !== 'undefined'

    if (!hasQuery) {
      // query not provided, get all channels
      return this.getAll()
    } else if (hasPartyA && hasPartyB) {
      // both parties provided, get channel
      const entry = await this.getSingle(query.partyA, query.partyB)

      if (typeof entry === 'undefined') {
        return []
      } else {
        return [entry]
      }
    } else {
      // only one of the parties provided, get all open channels of party
      return this.getAll(hasPartyA ? query.partyA : query.partyB)
    }
  }

  private async store(partyA: AccountId, partyB: AccountId, channelEntry: ChannelEntry): Promise<void> {
    const { dbKeys, db } = this.connector
    const { blockNumber, logIndex, transactionIndex } = channelEntry
    this.log(
      `storing channel ${partyA.toHex()}-${partyB.toHex()}:${blockNumber.toString()}-${transactionIndex.toString()}-${logIndex.toString()}`
    )

    return db.batch([
      {
        type: 'put',
        key: Buffer.from(dbKeys.ChannelEntry(partyA, partyB)),
        value: Buffer.from(channelEntry),
      },
      {
        type: 'put',
        key: Buffer.from(dbKeys.ConfirmedBlockNumber()),
        value: Buffer.from(blockNumber.toU8a()),
      },
    ])
  }

  // delete a channel
  private async delete(partyA: AccountId, partyB: AccountId): Promise<void> {
    this.log(`deleting channel ${partyA.toHex()}-${partyB.toHex()}`)

    const { dbKeys, db } = this.connector

    const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB))

    return db.del(key)
  }

  private async onNewBlock(block: BlockHeader) {
    const confirmedEvents = Array.from(this.unconfirmedEvents.values()).filter((event) => {
      return isConfirmedBlock(event.blockNumber, block.number)
    })

    for (const event of confirmedEvents) {
      const id = getEventId(event)
      this.unconfirmedEvents.delete(id)

      if (event.event === 'OpenedChannel') {
        this.onOpenedChannel(event as OpenedChannelEvent)
      } else {
        this.onClosedChannel(event as ClosedChannelEvent)
      }
    }
  }

  private async onOpenedChannel(event: OpenedChannelEvent): Promise<void> {
    const opener = new AccountId(stringToU8a(event.returnValues.opener))
    const counterParty = new AccountId(stringToU8a(event.returnValues.counterParty))
    const [partyA, partyB] = getParties(opener, counterParty)

    const newChannelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(event.blockNumber),
      transactionIndex: new BN(event.transactionIndex),
      logIndex: new BN(event.logIndex),
    })

    const channels = await this.get({
      partyA,
      partyB,
    })

    if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
      return
    }

    this.store(partyA, partyB, newChannelEntry)
  }

  private async onClosedChannel(event: ClosedChannelEvent): Promise<void> {
    const closer = new AccountId(stringToU8a(event.returnValues.closer))
    const counterParty = new AccountId(stringToU8a(event.returnValues.counterParty))
    const [partyA, partyB] = getParties(closer, counterParty)
    const newChannelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(event.blockNumber),
      transactionIndex: new BN(event.transactionIndex),
      logIndex: new BN(event.logIndex),
    })

    const channels = await this.get({
      partyA,
      partyB,
    })

    if (channels.length === 0) {
      return
    } else if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
      return
    }

    this.delete(partyA, partyB)
  }

  /**
   * Start indexing,
   * listen to all open / close events,
   * store entries after X confirmations.
   *
   * @returns true if start was succesful
   */
  public async start(): Promise<boolean> {
    this.log(`Starting indexer...`)

    if (typeof this.starting !== 'undefined') {
      return this.starting
    } else if (typeof this.stopping !== 'undefined') {
      throw Error('cannot start while stopping')
    } else if (this.status === 'started') {
      return true
    }

    this.starting = new Promise(async (resolve, reject) => {
      try {
        const onChainBlockNumber = await this.connector.web3.eth.getBlockNumber()
        let fromBlock = await this.getLatestConfirmedBlockNumber()

        // go back 8 blocks in case of a re-org at time of stopping
        if (fromBlock - MAX_CONFIRMATIONS > 0) {
          fromBlock = fromBlock - MAX_CONFIRMATIONS
        }

        this.log(`starting to pull events from block ${fromBlock}..`)

        this.newBlockEvent = this.connector.web3.eth
          .subscribe('newBlockHeaders')
          .on('error', (err) => reject(err))
          .on('data', (block) => {
            this.onNewBlock(block)
          })

        this.openedChannelEvent = this.connector.hoprChannels.events
          .OpenedChannel({
            fromBlock,
          })
          .on('error', (err) => reject(err))
          .on('data', (_event) => {
            const event: LightEvent<typeof _event> = {
              event: _event.event,
              blockNumber: _event.blockNumber,
              transactionHash: _event.transactionHash,
              transactionIndex: _event.transactionIndex,
              logIndex: _event.logIndex,
              returnValues: _event.returnValues,
            }

            if (isConfirmedBlock(event.blockNumber, onChainBlockNumber)) {
              this.onOpenedChannel(event)
            } else {
              this.unconfirmedEvents.set(getEventId(event), event)
            }
          })

        this.closedChannelEvent = this.connector.hoprChannels.events
          .ClosedChannel({
            fromBlock,
          })
          .on('error', (err) => reject(err))
          .on('data', (_event) => {
            const event: LightEvent<typeof _event> = {
              event: _event.event,
              blockNumber: _event.blockNumber,
              transactionHash: _event.transactionHash,
              transactionIndex: _event.transactionIndex,
              logIndex: _event.logIndex,
              returnValues: _event.returnValues,
            }

            if (isConfirmedBlock(event.blockNumber, onChainBlockNumber)) {
              this.onClosedChannel(event)
            } else {
              this.unconfirmedEvents.set(getEventId(event), event)
            }
          })

        this.status = 'started'
        this.log(chalk.green('Indexer started!'))
        return resolve(true)
      } catch (err) {
        this.log(err.message)

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
    this.log(`Stopping indexer...`)

    if (typeof this.starting !== 'undefined') {
      throw Error('cannot stop while starting')
    } else if (typeof this.stopping !== 'undefined') {
      return this.stopping
    } else if (this.status === 'stopped') {
      return true
    }

    this.stopping = new Promise((resolve) => {
      try {
        if (typeof this.newBlockEvent !== 'undefined') {
          this.newBlockEvent.unsubscribe()
        }
        if (typeof this.openedChannelEvent !== 'undefined') {
          this.openedChannelEvent.removeAllListeners()
        }
        if (typeof this.closedChannelEvent !== 'undefined') {
          this.closedChannelEvent.removeAllListeners()
        }

        this.unconfirmedEvents.clear()

        this.status = 'stopped'
        this.log(chalk.green('Indexer stopped!'))
        return resolve(true)
      } catch (err) {
        this.log(err.message)

        return resolve(false)
      }
    }).finally(() => {
      this.stopping = undefined
    })

    return this.stopping
  }
}

export default Indexer
