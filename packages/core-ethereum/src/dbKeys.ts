import { toU8a, serializeToU8a } from '@hoprnet/hopr-utils'
import { Hash, PublicKey } from './types'

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
export function ChannelId(signatureHash: Hash): Uint8Array {
  return serializeToU8a([
    [PREFIX, PREFIX.length],
    [channelIdSubPrefix, channelIdSubPrefix.length],
    [signatureHash.serialize(), Hash.SIZE]
  ])
}

/**
 * Returns the db-key under which nonces are saved.
 * @param channelId channelId of the channel
 * @param nonce the nonce
 */
export function Nonce(channelId: Hash, nonce: Hash): Uint8Array {
  return serializeToU8a([
    [PREFIX, PREFIX.length],
    [nonceSubPrefix, nonceSubPrefix.length],
    [channelId.serialize(), Hash.SIZE],
    [SEPERATOR, SEPERATOR.length],
    [nonce.serialize(), Hash.SIZE]
  ])
}

export function OnChainSecret(): Uint8Array {
  return OnChainSecretIntermediary(0)
}

export function OnChainSecretIntermediary(iteration: number): Uint8Array {
  return serializeToU8a([
    [PREFIX, PREFIX.length],
    [onChainSecretIntermediary, onChainSecretIntermediary.length],
    [SEPERATOR, SEPERATOR.length],
    [toU8a(iteration, ON_CHAIN_SECRET_ITERATION_WIDTH), ON_CHAIN_SECRET_ITERATION_WIDTH]
  ])
}

/**
 * Returns the db-key under which the tickets are saved in the database.
 */
export function AcknowledgedTicket(counterPartyPubKey: PublicKey, challenge: Hash): Uint8Array {
  return serializeToU8a([
    [ticketSubPrefix, ticketSubPrefix.length],
    [acknowledgedSubPrefix, acknowledgedSubPrefix.length],
    [counterPartyPubKey.serialize(), PublicKey.SIZE],
    [SEPERATOR, SEPERATOR.length],
    [challenge.serialize(), Hash.SIZE]
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
