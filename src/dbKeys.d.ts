import { TypeClasses } from './types';
export default interface DbKeys {
    /**
     * Returns the db-key under which the channel is saved.
     * @param counterparty counterparty of the channel
     */
    Channel(counterparty: TypeClasses.AccountId): Uint8Array;
    /**
     * Reconstructs the channelId from a db-key.
     * @param arr a channel db-key
     * @param props additional arguments
     */
    ChannelKeyParse(arr: Uint8Array, ...props: any[]): TypeClasses.AccountId;
    /**
     * Returns the db-key under which the challenge is saved.
     * @param channelId channelId of the channel
     * @param challenge challenge to save
     */
    Challenge(channelId: TypeClasses.Hash, challenge: TypeClasses.Hash): Uint8Array;
    /**
     * Reconstructs channelId and the specified challenge from a challenge db-key.
     * @param arr a challenge db-key
     * @param props additional arguments
     */
    ChallengeKeyParse(arr: Uint8Array, ...props: any[]): [TypeClasses.Hash, TypeClasses.Hash];
    /**
     * Returns the db-key under which signatures of acknowledgements are saved.
     * @param signatureHash hash of an ackowledgement signature
     */
    ChannelId(signatureHash: TypeClasses.Hash): Uint8Array;
    /**
     * Returns the db-key under which nonces are saved.
     * @param channelId channelId of the channel
     * @param nonce the nonce
     */
    Nonce(channelId: TypeClasses.Hash, nonce: TypeClasses.Hash): Uint8Array;
    /**
     * Returns the db-key under which the on-chain secret is saved.
     */
    OnChainSecret(): Uint8Array;
}
