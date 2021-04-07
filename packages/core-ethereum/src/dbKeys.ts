/*
  Helper functions which generate database keys
*/
import { toU8a } from '@hoprnet/hopr-utils'
import { Hash, PublicKey } from './types'
import type { Types } from '@hoprnet/hopr-core-connector-interface'

const encoder = new TextEncoder()
const PREFIX = encoder.encode('payments-')
const SEPERATOR = encoder.encode('-')
const challengeSubPrefix = encoder.encode('challenge-')
const channelIdSubPrefix = encoder.encode('channelId-')
const nonceSubPrefix = encoder.encode('nonce-')
const ticketSubPrefix = encoder.encode('tickets-')
const acknowledgedSubPrefix = encoder.encode('acknowledged-')
const onChainSecretIntermediary = encoder.encode('onChainSecretIntermediary-')

const ON_CHAIN_SECRET_ITERATION_WIDTH = 4 // bytes

/**
 * Returns the db-key under which the challenge is saved.
 * @param channelId channelId of the channel
 * @param challenge challenge to save
 */
export function Challenge(channelId: Types.Hash, challenge: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [challengeSubPrefix.length, challengeSubPrefix],
    [Hash.SIZE, channelId.serialize()],
    [SEPERATOR.length, SEPERATOR],
    [Hash.SIZE, challenge.serialize()]
  ])
}

/**
 * Reconstructs channelId and the specified challenge from a challenge db-key.
 * @param arr a challenge db-key
 */
export function ChallengeKeyParse(arr: Uint8Array): [Hash, Hash] {
  const channelIdStart = PREFIX.length + challengeSubPrefix.length
  const channelIdEnd = channelIdStart + Hash.SIZE
  const challengeStart = channelIdEnd + SEPERATOR.length
  const challengeEnd = challengeStart + Hash.SIZE

  return [new Hash(arr.slice(channelIdStart, channelIdEnd)), new Hash(arr.slice(challengeStart, challengeEnd))]
}

/**
 * Returns the db-key under which signatures of acknowledgements are saved.
 * @param signatureHash hash of an ackowledgement signature
 */
export function ChannelId(signatureHash: Types.Hash): Uint8Array {
  return allocationHelper([
    [PREFIX.length, PREFIX],
    [channelIdSubPrefix.length, channelIdSubPrefix],
    [Hash.SIZE, signatureHash.serialize()]
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
    [Hash.SIZE, channelId.serialize()],
    [SEPERATOR.length, SEPERATOR],
    [Hash.SIZE, nonce.serialize()]
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
    [ON_CHAIN_SECRET_ITERATION_WIDTH, toU8a(iteration, ON_CHAIN_SECRET_ITERATION_WIDTH)]
  ])
}

/**
 * Returns the db-key under which the tickets are saved in the database.
 */
export function AcknowledgedTicket(counterPartyPubKey: Types.PublicKey, challenge: Types.Hash): Uint8Array {
  return allocationHelper([
    [ticketSubPrefix.length, ticketSubPrefix],
    [acknowledgedSubPrefix.length, acknowledgedSubPrefix],
    [PublicKey.SIZE, counterPartyPubKey.serialize()],
    [SEPERATOR.length, SEPERATOR],
    [Hash.SIZE, challenge.serialize()]
  ])
}

/**
 * Reconstructs counterPartyPubKey and the specified challenge from a AcknowledgedTicket db-key.
 * @param arr a AcknowledgedTicket db-key
 * @param props additional arguments
 */
export function AcknowledgedTicketParse(arr: Uint8Array): [PublicKey, Hash] {
  const counterPartyPubKeyStart = ticketSubPrefix.length + acknowledgedSubPrefix.length
  const counterPartyPubKeyEnd = counterPartyPubKeyStart + PublicKey.SIZE
  const challengeStart = counterPartyPubKeyEnd + SEPERATOR.length
  const challengeEnd = challengeStart + Hash.SIZE

  return [
    new PublicKey(arr.slice(counterPartyPubKeyStart, counterPartyPubKeyEnd)),
    new Hash(arr.slice(challengeStart, challengeEnd))
  ]
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
