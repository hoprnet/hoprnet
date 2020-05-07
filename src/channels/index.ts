import type HoprEthereum from '..'
import BN from 'bn.js'
import chalk from 'chalk'
import { Subscription } from 'web3-core-subscriptions'
import { BlockHeader } from 'web3-eth'
import { u8aToNumber, stringToU8a } from '@hoprnet/hopr-utils'
import * as dbKeys from '../dbKeys'
import { AccountId, ChannelEntry } from '../types'
import { getParties, Log } from '../utils'
import { MAX_CONFIRMATIONS } from '../config'
import { ContractEventEmitter, ContractEventLog } from '../tsc/web3/types'

type Channel = { partyA: AccountId; partyB: AccountId; channelEntry: ChannelEntry }
type OpenedChannelEvent = ContractEventLog<{ opener: string; counterParty: string }>
type ClosedChannelEvent = ContractEventLog<{ closer: string; counterParty: string }>

/**
 * @returns a custom event id for logging purposes.
 */
function getEventId(event: ContractEventLog<any>): string {
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
 * Barebones indexer to keep track of all open payment channels.
 * Eventually we will move to a better solution.
 */
class Channels {
  private log = Log(['channels'])
  private status: 'started' | 'stopped' = 'stopped'
  private unconfirmedEvents = new Map<string, OpenedChannelEvent | ClosedChannelEvent>()
  private starting: Promise<any>
  private stopping: Promise<any>
  private newBlockEvent: Subscription<BlockHeader>
  private openedChannelEvent: ContractEventEmitter<any> | undefined
  private closedChannelEvent: ContractEventEmitter<any> | undefined

  constructor(private connector: HoprEthereum) {}

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

  // does it exist
  public async has(partyA: AccountId, partyB: AccountId): Promise<boolean> {
    return this.connector.db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB))).then(
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

  // @TODO: improve function types
  // get stored channels using a query
  public async get(query?: { partyA?: AccountId; partyB?: AccountId }): Promise<Channel[]> {
    const { dbKeys, db } = this.connector
    const channels: Channel[] = []
    const allSmall = new Uint8Array(AccountId.SIZE).fill(0x00)
    const allBig = new Uint8Array(AccountId.SIZE).fill(0xff)
    const hasQuery = typeof query !== 'undefined'
    const hasPartyA = hasQuery && typeof query.partyA !== 'undefined'
    const hasPartyB = hasQuery && typeof query.partyB !== 'undefined'

    if (hasQuery && !hasPartyA && !hasPartyB) {
      throw Error('query is empty')
    }

    let gte: Buffer
    let lte: Buffer
    if (hasQuery) {
      gte = Buffer.from(dbKeys.ChannelEntry(hasPartyA ? query.partyA : allSmall, hasPartyB ? query.partyB : allSmall))
      lte = Buffer.from(dbKeys.ChannelEntry(hasPartyA ? query.partyA : allBig, hasPartyB ? query.partyB : allBig))
    } else {
      gte = Buffer.from(dbKeys.ChannelEntry(allSmall, allSmall))
      lte = Buffer.from(dbKeys.ChannelEntry(allBig, allBig))
    }

    return new Promise((resolve, reject) => {
      db.createReadStream({
        gte,
        lte,
      })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const [partyA, partyB] = dbKeys.ChannelEntryParse(key)
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

  // get all stored channels
  public async getAll(): Promise<Channel[]> {
    return this.get()
  }

  // store a channel
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
      return event.blockNumber + MAX_CONFIRMATIONS <= block.number
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

  // listen to all open / close events, store entries after X confirmations
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
        let fromBlock = await this.getLatestConfirmedBlockNumber()
        // go back 12 blocks in case of a re-org
        if (fromBlock - MAX_CONFIRMATIONS > 0) {
          fromBlock = fromBlock - MAX_CONFIRMATIONS
        }

        this.log(`starting to pull events from block ${fromBlock}..`)

        this.newBlockEvent = this.connector.web3.eth
          .subscribe('newBlockHeaders')
          .on('data', (block) => {
            this.onNewBlock(block)
          })
          .on('error', reject)

        this.openedChannelEvent = this.connector.hoprChannels.events
          .OpenedChannel({
            fromBlock,
          })
          .on('data', (event) => {
            this.unconfirmedEvents.set(getEventId(event), event)
          })
          .on('error', reject)

        this.closedChannelEvent = this.connector.hoprChannels.events
          .ClosedChannel({
            fromBlock,
          })
          .on('data', (event) => {
            this.unconfirmedEvents.set(getEventId(event), event)
          })
          .on('error', reject)

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

  // stop listening to events
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
          this.openedChannelEvent.removeAllListeners()
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

export default Channels
