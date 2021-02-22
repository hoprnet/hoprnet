import type { AccountId, Hash } from './types'

/**
 * Returns the db-key under which the channel is saved.
 * @param counterparty counterparty of the channel
 */
export function Channel(counterparty: AccountId): Uint8Array

/**
 * Reconstructs the channelId from a db-key.
 * @param arr a channel db-key
 * @param props additional arguments
 */
export function ChannelKeyParse(arr: Uint8Array, ...props: any[]): AccountId

/**
 * Returns the db-key under which the challenge is saved.
 * @param channelId channelId of the channel
 * @param challenge challenge to save
 */
export function Challenge(channelId: Hash, challenge: Hash): Uint8Array

/**
 * Reconstructs channelId and the specified challenge from a challenge db-key.
 * @param arr a challenge db-key
 * @param props additional arguments
 */
export function ChallengeKeyParse(arr: Uint8Array, ...props: any[]): [Hash, Hash]

/**
 * Returns the db-key under which signatures of acknowledgements are saved.
 * @param signatureHash hash of an ackowledgement signature
 */
export function ChannelId(signatureHash: Hash): Uint8Array

/**
 * Returns the db-key under which nonces are saved.
 * @param channelId channelId of the channel
 * @param nonce the nonce
 */
export function Nonce(channelId: Hash, nonce: Hash): Uint8Array
