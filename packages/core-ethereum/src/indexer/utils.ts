import type { Log } from 'web3-core'
import type HoprEthereum from '..'
import type { Event } from './types'
import { u8aConcat, u8aToNumber, stringToU8a } from '@hoprnet/hopr-utils'
import { Public } from '../types'
import { MAX_CONFIRMATIONS } from '../config'
import { ConfirmedBlockNumber } from '../dbKeys'

export const SMALLEST_PUBLIC_KEY = Buffer.from(
  new Public(u8aConcat(new Uint8Array([0x02]), new Uint8Array(32).fill(0x00)))
)
export const BIGGEST_PUBLIC_KEY = Buffer.from(
  new Public(u8aConcat(new Uint8Array([0x03]), new Uint8Array(32).fill(0xff)))
)
export const CONFIRMED_BLOCK_KEY = Buffer.from(ConfirmedBlockNumber())

/**
 * Compares the two events provided and returns `true` if the first event
 * is the most recent.
 * @param event
 * @param oldEvent
 * @returns boolean
 */
export const isMoreRecentEvent = (event: Event<any>, oldEvent: Event<any>): boolean => {
  const okBlockNumber = oldEvent.blockNumber <= event.blockNumber
  const okTransactionIndex = okBlockNumber && oldEvent.transactionIndex <= event.transactionIndex
  const okLogIndex = okTransactionIndex && oldEvent.logIndex <= event.logIndex

  return okBlockNumber && okTransactionIndex && okLogIndex
}

/**
 * Compares blockNumber and onChainBlockNumber and returns `true`
 * if blockNumber is considered confirmed.
 * @returns boolean
 */
export const isConfirmedBlock = (blockNumber: number, onChainBlockNumber: number): boolean => {
  return blockNumber + MAX_CONFIRMATIONS <= onChainBlockNumber
}

/**
 * Assumed the first indexed event parameters are the public keys,
 * it then reconstructs them by looking into topic 0.
 * @TODO: requires documentantion
 * @param topics
 */
export const decodePublicKeysFromTopics = (topics: Log['topics']): [Public, Public] => {
  return [
    new Public(
      u8aConcat(new Uint8Array([2 + ((parseInt(topics[0].slice(64, 66), 16) >> 1) % 2)]), stringToU8a(topics[1]))
    ),
    new Public(u8aConcat(new Uint8Array([2 + (parseInt(topics[0].slice(64, 66), 16) % 2)]), stringToU8a(topics[2])))
  ]
}

/**
 * Queries the database to find the latest confirmed block number.
 * @returns promise that resolves to a number
 */
export const getLatestConfirmedBlockNumber = async (connector: HoprEthereum): Promise<number> => {
  const { db } = connector

  try {
    return u8aToNumber(await db.get(CONFIRMED_BLOCK_KEY)) as number
  } catch (err) {
    if (err.notFound) {
      return 0
    }

    throw err
  }
}
