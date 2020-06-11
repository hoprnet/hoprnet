/*
  Helper functions which generate database keys
*/
import { toU8a } from '@hoprnet/hopr-utils'
import { Hash, AccountId } from './types'
import type { Types } from '@hoprnet/hopr-core-connector-interface'

const encoder = new TextEncoder()
const PREFIX = encoder.encode('payments-')
const SEPERATOR = encoder.encode('-')
const channelSubPrefix = encoder.encode('channel-')
const channelEntrySubPrefix = encoder.encode('channelEntry-')
const challengeSubPrefix = encoder.encode('challenge-')
const channelIdSubPrefix = encoder.encode('channelId-')
const nonceSubPrefix = encoder.encode('nonce-')
const ticketSubPrefix = encoder.encode('ticket-')
const onChainSecretIntermediary = encoder.encode('onChainSecretIntermediary-')
const confirmedBlockNumber = encoder.encode('confirmedBlockNumber')

const ON_CHAIN_SECRET_ITERATION_WIDTH = 4 // bytes

/**
 * Returns the db-key under which the channel is saved.
 * @param counterparty counterparty of the channel
 */
export function Channel(counterparty: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelSubPrefix.length, channelSubPrefix],
    [counterparty.length, counterparty],
  ])
}

/**
 * Reconstructs the channelId from a db-key.
 * @param arr a channel db-key
 */
export function ChannelKeyParse(arr: Uint8Array): Uint8Array {
  return arr.slice(PREFIX.length + channelSubPrefix.length)
}

/**
 * Returns the db-key under which the challenge is saved.
 * @param channelId channelId of the channel
 * @param challenge challenge to save
 */
export function Challenge(channelId: Types.Hash, challenge: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [challengeSubPrefix.length, challengeSubPrefix],
    [channelId.length, channelId],
    [SEPERATOR.length, SEPERATOR],
    [challenge.length, challenge],
  ])
}

/**
 * Reconstructs channelId and the specified challenge from a challenge db-key.
 * @param arr a challenge db-key
 */
export function ChallengeKeyParse(arr: Uint8Array): [Hash, Hash] {
  const channelIdStart = PREFIX.length + challengeSubPrefix.length
  const channelIdEnd = channelIdStart + Hash.SIZE
  const challangeStart = channelIdEnd + SEPERATOR.length
  const challangeEnd = challangeStart + Hash.SIZE

  return [new Hash(arr.slice(channelIdStart, channelIdEnd)), new Hash(arr.slice(challangeStart, challangeEnd))]
}

/**
 * Returns the db-key under which signatures of acknowledgements are saved.
 * @param signatureHash hash of an ackowledgement signature
 */
export function ChannelId(signatureHash: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelIdSubPrefix.length, channelIdSubPrefix],
    [signatureHash.length, signatureHash],
  ])
}

/**
 * Returns the db-key under which nonces are saved.
 * @param channelId channelId of the channel
 * @param nonce the nonce
 */
export function Nonce(channelId: Types.Hash, nonce: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [nonceSubPrefix.length, nonceSubPrefix],
    [channelId.length, channelId],
    [SEPERATOR.length, SEPERATOR],
    [nonce.length, nonce],
  ])
}

export function OnChainSecret(): Uint8Array {
  return OnChainSecretIntermediary(0)
}

export function OnChainSecretIntermediary(iteration: number): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [onChainSecretIntermediary.length, onChainSecretIntermediary],
    [SEPERATOR.length, SEPERATOR],
    [ON_CHAIN_SECRET_ITERATION_WIDTH, toU8a(iteration, ON_CHAIN_SECRET_ITERATION_WIDTH)],
  ])
}

/**
 * Returns the db-key under which the tickets are saved in the database.
 */
export function Ticket(channelId: Types.Hash, challenge: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [ticketSubPrefix.length, ticketSubPrefix],
    [channelId.length, channelId],
    [SEPERATOR.length, SEPERATOR],
    [challenge.length, challenge],
  ])
}

/**
 * Returns the db-key under which the latest confirmed block number is saved in the database.
 */
export function ConfirmedBlockNumber(): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [confirmedBlockNumber.length, confirmedBlockNumber],
  ])
}

/**
 * Returns the db-key under which channel entries are saved.
 * @param partyA the accountId of partyA
 * @param partyB the accountId of partyB
 */
export function ChannelEntry(partyA: Types.AccountId, partyB: Types.AccountId): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelEntrySubPrefix.length, channelEntrySubPrefix],
    [partyA.length, partyA],
    [SEPERATOR.length, SEPERATOR],
    [partyB.length, partyB],
  ])
}

/**
 * Reconstructs parties from a channel entry db-key.
 * @param arr a challenge db-key
 * @returns an array containing partyA's and partyB's accountIds
 */
export function ChannelEntryParse(arr: Uint8Array): [Types.AccountId, Types.AccountId] {
  const partyAStart = PREFIX.length + channelEntrySubPrefix.length
  const partyAEnd = partyAStart + AccountId.SIZE
  const partyBStart = partyAEnd + SEPERATOR.length
  const partyBEnd = partyBStart + AccountId.SIZE

  return [new AccountId(arr.slice(partyAStart, partyAEnd)), new AccountId(arr.slice(partyBStart, partyBEnd))]
}

function allocationHelper(arr: [number, Uint8Array][]): Uint8Array {
  const totalLength = arr.reduce((acc, current) => {
    return acc + current[0]
  }, 0)

  let result = new Uint8Array(totalLength)

  let offset = 0
  for (let [size, data] of arr) {
    result.set(data, offset)
    offset += size
  }

  return result
}
