import type HoprEthereum from '..'
import { Subscription } from 'web3-core-subscriptions'
import { BlockHeader } from 'web3-eth'
import { u8aToNumber, toU8a, stringToU8a } from '@hoprnet/hopr-utils'
import * as dbKeys from '../dbKeys'
import { AccountId } from '../types'
import { getParties, Log, getEventId } from '../utils'
import { MAX_CONFIRMATIONS } from '../config'
import { ContractEventEmitter, ContractEventLog } from '../tsc/web3/types'

type Channel = { partyA: AccountId; partyB: AccountId; blockNumber: number }
type OpenedChannelEvent = ContractEventLog<{ opener: string; counterParty: string }>
type ClosedChannelEvent = ContractEventLog<{ closer: string; counterParty: string }>

const log = Log(['channels'])
const unconfirmedEvents = new Map<string, OpenedChannelEvent | ClosedChannelEvent>()
let isStarted = false
let newBlockEvent: Subscription<BlockHeader>
let openedChannelEvent: ContractEventEmitter<any> | undefined
let closedChannelEvent: ContractEventEmitter<any> | undefined

class Channels {
  static async getLatestConfirmedBlockNumber(coreConnector: HoprEthereum): Promise<number> {
    try {
      const blockNumber = await coreConnector.db
        .get(Buffer.from(coreConnector.dbKeys.ConfirmedBlockNumber()))
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
  static async has(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<boolean> {
    return coreConnector.db.get(Buffer.from(dbKeys.ChannelEntry(partyA, partyB))).then(
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
    coreConnector: HoprEthereum,
    query?: {
      partyA?: AccountId
      partyB?: AccountId
    }
  ): Promise<Channel[]> {
    const { dbKeys, db } = coreConnector
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
          channels.push({
            partyA: new AccountId(partyA),
            partyB: new AccountId(partyB),
            blockNumber: u8aToNumber(value),
          })
        })
        .on('end', () => resolve(channels))
    })
  }

  // get all stored channels
  static async getAll(coreConnector: HoprEthereum): Promise<Channel[]> {
    return Channels.get(coreConnector)
  }

  // store a channel
  static async store(
    coreConnector: HoprEthereum,
    partyA: AccountId,
    partyB: AccountId,
    blockNumber: number
  ): Promise<[void, void]> {
    log(`storing channel ${partyA.toHex()}-${partyB.toHex()}:${blockNumber}`)

    const { dbKeys, db } = coreConnector

    return Promise.all([
      db.put(Buffer.from(dbKeys.ChannelEntry(partyA, partyB)), Buffer.from(toU8a(blockNumber))),
      db.put(Buffer.from(dbKeys.ConfirmedBlockNumber()), Buffer.from(toU8a(blockNumber))),
    ])
  }

  // delete a channel
  static async delete(coreConnector: HoprEthereum, partyA: AccountId, partyB: AccountId): Promise<void> {
    log(`deleting channel ${partyA.toHex()}-${partyB.toHex()}`)

    const { dbKeys, db } = coreConnector

    const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB))

    return db.del(key)
  }

  static async onNewBlock(coreConnector: HoprEthereum, block: BlockHeader) {
    const confirmedEvents = Array.from(unconfirmedEvents.values()).filter((event) => {
      return event.blockNumber + MAX_CONFIRMATIONS <= block.number
    })

    for (const event of confirmedEvents) {
      const id = getEventId(event)
      unconfirmedEvents.delete(id)

      if (event.event === 'OpenedChannel') {
        Channels.onOpenedChannel(coreConnector, event as OpenedChannelEvent)
      } else {
        Channels.onClosedChannel(coreConnector, event as ClosedChannelEvent)
      }
    }
  }

  static async onOpenedChannel(coreConnector: HoprEthereum, event: OpenedChannelEvent): Promise<void> {
    const opener = new AccountId(stringToU8a(event.returnValues.opener))
    const counterParty = new AccountId(stringToU8a(event.returnValues.counterParty))
    const [partyA, partyB] = getParties(opener, counterParty)

    const channels = await Channels.get(coreConnector, {
      partyA,
      partyB,
    })

    if (channels.length > 0 && channels[0].blockNumber > event.blockNumber) {
      return
    }

    Channels.store(coreConnector, partyA, partyB, event.blockNumber)
  }

  static async onClosedChannel(coreConnector: HoprEthereum, event: ClosedChannelEvent): Promise<void> {
    const closer = new AccountId(stringToU8a(event.returnValues.closer))
    const counterParty = new AccountId(stringToU8a(event.returnValues.counterParty))
    const [partyA, partyB] = getParties(closer, counterParty)

    const channels = await Channels.get(coreConnector, {
      partyA,
      partyB,
    })

    if (channels.length === 0) {
      return
    } else if (channels.length > 0 && channels[0].blockNumber > event.blockNumber) {
      return
    }

    Channels.delete(coreConnector, partyA, partyB)
  }

  // listen to all open / close events, store entries after X confirmations
  static async start(coreConnector: HoprEthereum): Promise<boolean> {
    try {
      if (isStarted) {
        log(`already started..`)
        return true
      }

      let fromBlock = await Channels.getLatestConfirmedBlockNumber(coreConnector)
      // go back 12 blocks in case of a re-org
      if (fromBlock - MAX_CONFIRMATIONS > 0) {
        fromBlock = fromBlock - MAX_CONFIRMATIONS
      }

      log(`starting to pull events from block ${fromBlock}..`)

      newBlockEvent = coreConnector.web3.eth.subscribe('newBlockHeaders').on('data', (block) => {
        Channels.onNewBlock(coreConnector, block)
      })

      openedChannelEvent = coreConnector.hoprChannels.events
        .OpenedChannel({
          fromBlock,
        })
        .on('data', (event) => {
          unconfirmedEvents.set(getEventId(event), event)
        })

      closedChannelEvent = coreConnector.hoprChannels.events
        .ClosedChannel({
          fromBlock,
        })
        .on('data', (event) => {
          unconfirmedEvents.set(getEventId(event), event)
        })

      isStarted = true
      return true
    } catch (err) {
      log(err.message)
      return isStarted
    }
  }

  // stop listening to events
  static async stop(): Promise<boolean> {
    try {
      if (!isStarted) return true

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

      isStarted = false
      return true
    } catch (err) {
      log(err.message)
      return isStarted
    }
  }
}

export default Channels
