import type { AccountId, Hash } from './types';

declare interface DbKeys {
    /**
     * Returns the db-key under which the channel is saved.
     * @param counterparty counterparty of the channel
     */
    Channel(counterparty: AccountId.Instance): Uint8Array;
    /**
     * Reconstructs the channelId from a db-key.
     * @param arr a channel db-key
     * @param props additional arguments
     */
    ChannelKeyParse(arr: Uint8Array, ...props: any[]): AccountId.Instance;
    /**
     * Returns the db-key under which the challenge is saved.
     * @param channelId channelId of the channel
     * @param challenge challenge to save
     */
    Challenge(channelId: Hash.Instance, challenge: Hash.Instance): Uint8Array;
    /**
     * Reconstructs channelId and the specified challenge from a challenge db-key.
     * @param arr a challenge db-key
     * @param props additional arguments
     */
    ChallengeKeyParse(arr: Uint8Array, ...props: any[]): [Hash.Instance, Hash.Instance];
    /**
     * Returns the db-key under which signatures of acknowledgements are saved.
     * @param signatureHash hash of an ackowledgement signature
     */
    ChannelId(signatureHash: Hash.Instance): Uint8Array;
    /**
     * Returns the db-key under which nonces are saved.
     * @param channelId channelId of the channel
     * @param nonce the nonce
     */
    Nonce(channelId: Hash.Instance, nonce: Hash.Instance): Uint8Array;
    /**
     * Returns the db-key under which the on-chain secret is saved.
     */
    OnChainSecret(): Uint8Array;
    /**
     * Returns the db-key under which the tickets are saved in the database.
     */
    Ticket(channelId: Hash.Instance, challenge: Hash.Instance): Uint8Array
}

export default DbKeys
