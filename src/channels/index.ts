import type HoprEthereum from '..'
import { u8aToNumber, toU8a, stringToU8a } from '@hoprnet/hopr-utils'
import * as dbKeys from '../dbKeys'
import { AccountId } from '../types'
import { getParties, Log } from '../utils'

type Channel = { partyA: AccountId; partyB: AccountId; blockNumber: number }

const log = Log(['channels'])

class Channels {
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

    let gt: Buffer
    let lt: Buffer
    if (hasQuery) {
      gt = Buffer.from(dbKeys.ChannelEntry(hasPartyA ? query.partyA : allSmall, hasPartyB ? query.partyB : allSmall))
      lt = Buffer.from(dbKeys.ChannelEntry(hasPartyA ? query.partyA : allBig, hasPartyB ? query.partyB : allBig))
    } else {
      gt = Buffer.from(dbKeys.ChannelEntry(allSmall, allSmall))
      lt = Buffer.from(dbKeys.ChannelEntry(allBig, allBig))
    }

    return new Promise((resolve, reject) => {
      db.createReadStream({
        gt,
        lt,
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
    const { dbKeys, db } = coreConnector

    const key = Buffer.from(dbKeys.ChannelEntry(partyA, partyB))

    return db.del(key)
  }

  // listen to all open / close events, store entries after X confirmations
  static async start(coreConnector: HoprEthereum): Promise<void> {
    let fromBlock = 0

    try {
      fromBlock = await coreConnector.db.get(Buffer.from(coreConnector.dbKeys.ConfirmedBlockNumber())).then((res) => {
        return u8aToNumber(res)
      })
    } catch (err) {
      if (err.notFound == null) {
        throw err
      }
    }

    log(`starting to pull events from block ${fromBlock}..`)

    coreConnector.hoprChannels.events
      .OpenedChannel({
        fromBlock,
      })
      .on('data', async (event) => {
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
      })

    coreConnector.hoprChannels.events
      .ClosedChannel({
        fromBlock,
      })
      .on('data', async (event) => {
        const opener = new AccountId(stringToU8a(event.returnValues.closer))
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
      })
  }
}

export default Channels
