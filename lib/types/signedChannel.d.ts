import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { Signature, Channel } from '.';
import { Uint8ArrayE } from '../types/extended';
import type HoprEthereum from '..';
declare class SignedChannel extends Uint8ArrayE implements Types.SignedChannel {
    private _signature?;
    private _channel?;
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        signature: Signature;
        channel: Channel;
    });
    get signature(): Signature;
    get channel(): Channel;
    get signer(): Promise<Uint8Array>;
    verify(coreConnector: HoprEthereum): Promise<boolean>;
    static get SIZE(): number;
    static create(coreConnector: HoprEthereum, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        channel: Channel;
        signature?: Signature;
    }): Promise<SignedChannel>;
}
export default SignedChannel;
