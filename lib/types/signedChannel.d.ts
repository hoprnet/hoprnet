import type { Types } from '@hoprnet/hopr-core-connector-interface';
import Signature from './signature';
import Channel from './channel';
import { Uint8ArrayE } from '../types/extended';
declare class SignedChannel extends Uint8ArrayE implements Types.SignedChannel {
    private _signature?;
    private _channel?;
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature?: Signature;
        channel?: Channel;
    });
    get signatureOffset(): number;
    get signature(): Signature;
    get channelOffset(): number;
    get channel(): Channel;
    get signer(): Promise<Uint8Array>;
    verify(publicKey: Uint8Array): Promise<boolean>;
    static get SIZE(): number;
    static create(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature?: Signature;
        channel?: Channel;
    }): Promise<SignedChannel>;
}
export default SignedChannel;
