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

const log = Log(['channels'])
const unconfirmedEvents = new Map<string, OpenedChannelEvent | ClosedChannelEvent>()
let status: 'started' | 'stopped' = 'stopped'
let starting: Promise<any>
let stopping: Promise<any>
let newBlockEvent: Subscription<BlockHeader>
let openedChannelEvent: ContractEventEmitter<any> | undefined
let closedChannelEvent: ContractEventEmitter<any> | undefined

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
  static async getLatestConfirmedBlockNumber(connector: HoprEthereum): Promise<number> {
    try {
      const blockNumber = await connector.db.get(Buffer.from(connector.dbKeys.ConfirmedBlockNumber())).then((res) => {
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
  static async has(connector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<boolean> {
    return connector.db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB))).then(
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
  static async get(
    connector: HoprEthereum,
    query?: {
      partyA?: AccountId
      partyB?: AccountId
    }
  ): Promise<Channel[]> {
    const { dbKeys, db } = connector
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
  static async getAll(connector: HoprEthereum): Promise<Channel[]> {
    return Channels.get(connector)
  }

  // store a channel
  static async store(
    connector: HoprEthereum,
    partyA: AccountId,
    partyB: AccountId,
    channelEntry: ChannelEntry
  ): Promise<void> {
    const { dbKeys, db } = connector
    const { blockNumber, logIndex, transactionIndex } = channelEntry
    log(
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
  static async delete(connector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<void> {
    log(`deleting channel ${partyA.toHex()}-${partyB.toHex()}`)

    const { dbKeys, db } = connector

    const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB))

    return db.del(key)
  }

  static async onNewBlock(connector: HoprEthereum, block: BlockHeader) {
    const confirmedEvents = Array.from(unconfirmedEvents.values()).filter((event) => {
      return event.blockNumber + MAX_CONFIRMATIONS <= block.number
    })

    for (const event of confirmedEvents) {
      const id = getEventId(event)
      unconfirmedEvents.delete(id)

      if (event.event === 'OpenedChannel') {
        Channels.onOpenedChannel(connector, event as OpenedChannelEvent)
      } else {
        Channels.onClosedChannel(connector, event as ClosedChannelEvent)
      }
    }
  }

  static async onOpenedChannel(connector: HoprEthereum, event: OpenedChannelEvent): Promise<void> {
    const opener = new AccountId(stringToU8a(event.returnValues.opener))
    const counterParty = new AccountId(stringToU8a(event.returnValues.counterParty))
    const [partyA, partyB] = getParties(opener, counterParty)

    const newChannelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(event.blockNumber),
      transactionIndex: new BN(event.transactionIndex),
      logIndex: new BN(event.logIndex),
    })

    const channels = await Channels.get(connector, {
      partyA,
      partyB,
    })

    if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
      return
    }

    Channels.store(connector, partyA, partyB, newChannelEntry)
  }

  static async onClosedChannel(connector: HoprEthereum, event: ClosedChannelEvent): Promise<void> {
    const closer = new AccountId(stringToU8a(event.returnValues.closer))
    const counterParty = new AccountId(stringToU8a(event.returnValues.counterParty))
    const [partyA, partyB] = getParties(closer, counterParty)
    const newChannelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(event.blockNumber),
      transactionIndex: new BN(event.transactionIndex),
      logIndex: new BN(event.logIndex),
    })

    const channels = await Channels.get(connector, {
      partyA,
      partyB,
    })

    if (channels.length === 0) {
      return
    } else if (channels.length > 0 && !isMoreRecent(channels[0].channelEntry, newChannelEntry)) {
      return
    }

    Channels.delete(connector, partyA, partyB)
  }

  // listen to all open / close events, store entries after X confirmations
  static async start(connector: HoprEthereum): Promise<boolean> {
    log(`Starting indexer...`)

    if (typeof starting !== 'undefined') {
      return starting
    } else if (typeof stopping !== 'undefined') {
      throw Error('cannot start while stopping')
    } else if (status === 'started') {
      return true
    }

    starting = new Promise(async (resolve, reject) => {
      try {
        let fromBlock = await Channels.getLatestConfirmedBlockNumber(connector)
        // go back 12 blocks in case of a re-org
        if (fromBlock - MAX_CONFIRMATIONS > 0) {
          fromBlock = fromBlock - MAX_CONFIRMATIONS
        }

        log(`starting to pull events from block ${fromBlock}..`)

        newBlockEvent = connector.web3.eth
          .subscribe('newBlockHeaders')
          .on('data', (block) => {
            Channels.onNewBlock(connector, block)
          })
          .on('error', reject)

        openedChannelEvent = connector.hoprChannels.events
          .OpenedChannel({
            fromBlock,
          })
          .on('data', (event) => {
            unconfirmedEvents.set(getEventId(event), event)
          })
          .on('error', reject)

        closedChannelEvent = connector.hoprChannels.events
          .ClosedChannel({
            fromBlock,
          })
          .on('data', (event) => {
            unconfirmedEvents.set(getEventId(event), event)
          })
          .on('error', reject)

        status = 'started'
        log(chalk.green('Indexer started!'))
        return resolve(true)
      } catch (err) {
        log(err.message)

        return Channels.stop()
      }
    }).finally(() => {
      starting = undefined
    })

    return starting
  }

  // stop listening to events
  static async stop(): Promise<boolean> {
    log(`Stopping indexer...`)

    if (typeof starting !== 'undefined') {
      throw Error('cannot stop while starting')
    } else if (typeof stopping !== 'undefined') {
      return stopping
    } else if (status === 'stopped') {
      return true
    }

    stopping = new Promise((resolve) => {
      try {
        if (typeof newBlockEvent !== 'undefined') {
          newBlockEvent.unsubscribe()
        }
        if (typeof openedChannelEvent !== 'undefined') {
          openedChannelEvent.removeAllListeners()
        }
        if (typeof closedChannelEvent !== 'undefined') {
          openedChannelEvent.removeAllListeners()
        }

        unconfirmedEvents.clear()

        status = 'stopped'
        log(chalk.green('Indexer stopped!'))
        return resolve(true)
      } catch (err) {
        log(err.message)

        return resolve(false)
      }
    }).finally(() => {
      stopping = undefined
    })

    return stopping
  }
}

export default Channels
