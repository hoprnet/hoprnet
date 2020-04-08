import { Challenge } from '../packet/challenge';
import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface';
import PeerId from 'peer-id';
/**
 * This class encapsulates the message that is sent back to the relayer
 * and allows that party to compute the key that is necessary to redeem
 * the previously received transaction.
 */
declare class Acknowledgement<Chain extends HoprCoreConnector> extends Uint8Array {
    private _responseSigningParty?;
    private _hashedKey?;
    private paymentChannels;
    constructor(paymentChannels: Chain, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        key: Uint8Array;
        challenge: Challenge<Chain>;
        signature?: Types.Signature;
    });
    subarray(begin?: number, end?: number): Uint8Array;
    get key(): Uint8Array;
    get hashedKey(): Promise<Uint8Array>;
    get challenge(): Challenge<Chain>;
    get hash(): Promise<Uint8Array>;
    get challengeSignatureHash(): Promise<Uint8Array>;
    get challengeSigningParty(): Promise<Uint8Array>;
    get responseSignature(): Types.Signature;
    get responseSigningParty(): Promise<Uint8Array>;
    sign(peerId: PeerId): Promise<Acknowledgement<Chain>>;
    verify(peerId: PeerId): Promise<boolean>;
    /**
     * Takes a challenge from a relayer and returns an acknowledgement that includes a
     * signature over the requested key half.
     *
     * @param challenge the signed challenge of the relayer
     * @param derivedSecret the secret that is used to create the second key half
     * @param signer contains private key
     */
    static create<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain, challenge: Challenge<Chain>, derivedSecret: Uint8Array, signer: PeerId): Promise<Acknowledgement<Chain>>;
    static SIZE<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain): number;
}
export { Acknowledgement };
