import { Types } from '@hoprnet/hopr-core-connector-interface';
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
        signature: Signature;
        channel: Channel;
    });
    get signature(): Signature;
    get channel(): Channel;
    get signer(): Promise<Uint8Array>;
    verify(publicKey: Uint8Array): Promise<boolean>;
    static get SIZE(): number;
}
export default SignedChannel;
