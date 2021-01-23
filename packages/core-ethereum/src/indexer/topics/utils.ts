import type { Log } from 'web3-core'
import type { EventData } from './types'
import createKeccakHash from 'keccak'
import { u8aConcat, stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'
import { Public } from '../../types'

/**
 * known event signatures that we will subscribe and reduce
 * data from, ideally this should be taken from
 * the web3 types genereted but at this time we can't
 * since we use non-standard events that typechain doesn't
 * recognise
 */
export const EventSignatures: {
  [K in keyof EventData]: Buffer
} = {
  FundedChannel: createKeccakHash('keccak256').update('FundedChannel(address,uint,uint,uint,uint)').digest(),
  OpenedChannel: createKeccakHash('keccak256').update('OpenedChannel(uint,uint)').digest(),
  RedeemedTicket: createKeccakHash('keccak256').update('RedeemedTicket(uint,uint,uint)').digest(),
  InitiatedChannelClosure: createKeccakHash('keccak256').update('InitiatedChannelClosure(uint,uint,uint)').digest(),
  ClosedChannel: createKeccakHash('keccak256').update('ClosedChannel(uint,uint,uint,uint)').digest()
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
 * @TODO: requires documentantion
 * @param rawTopic
 * @param first
 * @param second
 */
export const getTopic0 = (rawTopic: Uint8Array, first: number, second: number): string => {
  return u8aToHex(
    u8aConcat(rawTopic.slice(0, 31), new Uint8Array([((rawTopic[31] >> 2) << 2) | (first % 2 << 1) | second % 2]))
  )
}
