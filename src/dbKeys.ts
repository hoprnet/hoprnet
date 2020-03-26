import type { Types } from '@hoprnet/hopr-core-connector-interface'
import * as constants from './constants'

const encoder = new TextEncoder()
const PREFIX = encoder.encode('payments-')
const SEPERATOR = encoder.encode('-')
const channelSubPrefix = encoder.encode('channel-')
const challengeSubPrefix = encoder.encode('challenge-')

export function Channel(counterparty: Types.AccountId): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelSubPrefix.length, channelSubPrefix],
    [counterparty.length, counterparty]
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
    [challenge.length, challenge]
  ])
}

export function ChallengeKeyParse(arr: Uint8Array): [Types.Hash, Types.Hash] {
  return [
    arr.slice(
      PREFIX.length + channelSubPrefix.length,
      PREFIX.length + channelSubPrefix.length + constants.HASH_LENGTH
    ),
    arr.slice(
      PREFIX.length + channelSubPrefix.length + constants.HASH_LENGTH + SEPERATOR.length,
      PREFIX.length + channelSubPrefix.length + constants.HASH_LENGTH + SEPERATOR.length + constants.HASH_LENGTH
    )
  ]
}

export function ChannelId(signatureHash: Types.Hash): Uint8Array {
  const subPrefix = encoder.encode('channelId-')

  return allocationHelper([
    [PREFIX.length, PREFIX],
    [subPrefix.length, subPrefix],
    [signatureHash.length, signatureHash]
  ])
}

export function Nonce(channelId: Types.Hash, nonce: Types.Hash): Uint8Array {
  const subPrefix = encoder.encode('nonce-')

  return allocationHelper([
    [PREFIX.length, PREFIX],
    [subPrefix.length, subPrefix],
    [channelId.length, channelId],
    [SEPERATOR.length, SEPERATOR],
    [nonce.length, nonce]
  ])
}

export function OnChainSecret(): Uint8Array {
  const subPrefix = encoder.encode('onChainSecret')

  return allocationHelper([
    [PREFIX.length, PREFIX],
    [subPrefix.length, subPrefix]
  ])
}

export function Ticket(channelId: Types.Hash, challenge: Types.Hash): Uint8Array {
  const subPrefix = encoder.encode('ticket-')

  return allocationHelper([
    [PREFIX.length, PREFIX],
    [subPrefix.length, subPrefix],
    [channelId.length, channelId],
    [SEPERATOR.length, SEPERATOR],
    [challenge.length, challenge]
  ])
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