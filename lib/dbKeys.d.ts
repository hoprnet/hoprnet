import { Hash } from './types';
import type { Types } from '@hoprnet/hopr-core-connector-interface';
/**
 * Returns the db-key under which the channel is saved.
 * @param counterparty counterparty of the channel
 */
export declare function Channel(counterparty: Types.Hash): Uint8Array;
/**
 * Reconstructs the channelId from a db-key.
 * @param arr a channel db-key
 */
export declare function ChannelKeyParse(arr: Uint8Array): Uint8Array;
/**
 * Returns the db-key under which the challenge is saved.
 * @param channelId channelId of the channel
 * @param challenge challenge to save
 */
export declare function Challenge(channelId: Types.Hash, challenge: Types.Hash): Uint8Array;
/**
 * Reconstructs channelId and the specified challenge from a challenge db-key.
 * @param arr a challenge db-key
 */
export declare function ChallengeKeyParse(arr: Uint8Array): [Hash, Hash];
/**
 * Returns the db-key under which signatures of acknowledgements are saved.
 * @param signatureHash hash of an ackowledgement signature
 */
export declare function ChannelId(signatureHash: Types.Hash): Uint8Array;
/**
 * Returns the db-key under which nonces are saved.
 * @param channelId channelId of the channel
 * @param nonce the nonce
 */
export declare function Nonce(channelId: Types.Hash, nonce: Types.Hash): Uint8Array;
export declare function OnChainSecret(): Uint8Array;
export declare function OnChainSecretIntermediary(iteration: number): Uint8Array;
/**
 * Returns the db-key under which the tickets are saved in the database.
 */
export declare function Ticket(channelId: Types.Hash, challenge: Types.Hash): Uint8Array;
/**
 * Returns the db-key under which the latest confirmed block number is saved in the database.
 */
export declare function ConfirmedBlockNumber(): Uint8Array;
/**
 * Returns the db-key under which channel entries are saved.
 * @param partyA the accountId of partyA
 * @param partyB the accountId of partyB
 */
export declare function ChannelEntry(partyA: Types.AccountId, partyB: Types.AccountId): Uint8Array;
/**
 * Reconstructs parties from a channel entry db-key.
 * @param arr a challenge db-key
 * @returns an array containing partyA's and partyB's accountIds
 */
export declare function ChannelEntryParse(arr: Uint8Array): [Types.AccountId, Types.AccountId];
