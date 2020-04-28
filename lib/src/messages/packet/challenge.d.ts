import PeerId from 'peer-id';
import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface';
import BN from 'bn.js';
/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an
 * inappropriate acknowledgement.
 */
export declare class Challenge<Chain extends HoprCoreConnector> extends Uint8Array {
    private paymentChannels;
    private _hashedKey;
    private _fee;
    private _counterparty;
    constructor(paymentChannels: Chain, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature: Types.Signature;
    });
    get challengeSignature(): Types.Signature;
    set challengeSignature(signature: Types.Signature);
    get signatureHash(): Promise<Types.Hash>;
    static SIZE<Chain extends HoprCoreConnector>(paymentChannels: Chain): number;
    get hash(): Types.Hash;
    subarray(begin?: number, end?: number): Uint8Array;
    getCopy(): Challenge<Chain>;
    /**
     * Uses the derived secret and the signature to recover the public
     * key of the signer.
     */
    get counterparty(): Promise<Uint8Array>;
    /**
     * Signs the challenge and includes the transferred amount of money as
     * well as the ethereum address of the signer into the signature.
     *
     * @param peerId that contains private key and public key of the node
     */
    sign(peerId: PeerId): Promise<Challenge<Chain>>;
    /**
     * Creates a challenge object.
     *
     * @param hashedKey that is used to generate the key half
     * @param fee
     */
    static create<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain, hashedKey: Uint8Array, fee: BN, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Challenge<Chain>;
    /**
     * Verifies the challenge by checking whether the given public matches the
     * one restored from the signature.
     *
     * @param peerId PeerId instance that contains the public key of
     * the signer
     * @param secret the secret that was used to derive the key half
     */
    verify(peerId: PeerId): Promise<boolean>;
}
