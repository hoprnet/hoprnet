import { Hash, AccountId } from './types'
import type { Types } from '@hoprnet/hopr-core-connector-interface'

const encoder = new TextEncoder()
const PREFIX = encoder.encode('payments-')
const SEPERATOR = encoder.encode('-')
const channelSubPrefix = encoder.encode('channel-')
const challengeSubPrefix = encoder.encode('challenge-')
const channelIdSubPrefix = encoder.encode('channelId-')
const nonceSubPrefix = encoder.encode('nonce-')
const ticketSubPrefix = encoder.encode('ticket-')
const onChainSecret = encoder.encode('onChainSecret')
const confirmedBlockNumber = encoder.encode('confirmedBlockNumber')

export function Channel(counterparty: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelSubPrefix.length, channelSubPrefix],
    [counterparty.length, counterparty],
  ])
}

export function ChannelKeyParse(arr: Uint8Array): Uint8Array {
  return arr.slice(PREFIX.length + channelSubPrefix.length)
}

export function Challenge(channelId: Types.Hash, challenge: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [challengeSubPrefix.length, challengeSubPrefix],
    [channelId.length, channelId],
    [SEPERATOR.length, SEPERATOR],
    [challenge.length, challenge],
  ])
}

export function ChallengeKeyParse(arr: Uint8Array): [Hash, Hash] {
  const fromStart = PREFIX.length + challengeSubPrefix.length
  const fromEnd = fromStart + Hash.SIZE
  const toStart = fromEnd + SEPERATOR.length
  const toEnd = toStart + Hash.SIZE

  return [new Hash(arr.slice(fromStart, fromEnd)), new Hash(arr.slice(toStart, toEnd))]
}

export function ChannelId(signatureHash: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelIdSubPrefix.length, channelIdSubPrefix],
    [signatureHash.length, signatureHash],
  ])
}

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
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [onChainSecret.length, onChainSecret],
  ])
}

export function Ticket(channelId: Types.Hash, challenge: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [ticketSubPrefix.length, ticketSubPrefix],
    [channelId.length, channelId],
    [SEPERATOR.length, SEPERATOR],
    [challenge.length, challenge],
  ])
}

export function ConfirmedBlockNumber(): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [confirmedBlockNumber.length, confirmedBlockNumber],
  ])
}

export function ChannelEntry(from: Types.AccountId, to: Types.AccountId): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelSubPrefix.length, channelSubPrefix],
    [from.length, from],
    [SEPERATOR.length, SEPERATOR],
    [to.length, to],
  ])
}

export function ChannelEntryParse(arr: Uint8Array): [Types.AccountId, Types.AccountId] {
  const fromStart = PREFIX.length + channelSubPrefix.length
  const fromEnd = fromStart + AccountId.SIZE
  const toStart = fromEnd + SEPERATOR.length
  const toEnd = toStart + AccountId.SIZE

  return [new AccountId(arr.slice(fromStart, fromEnd)), new AccountId(arr.slice(toStart, toEnd))]
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
