import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { ChannelBalance, Moment } from '.';
import { Uint8ArrayE } from '../types/extended';
import { Signature } from '@hoprnet/hopr-core-connector-interface/src/types';
export declare enum ChannelStatus {
    UNINITIALISED = 0,
    FUNDING = 1,
    OPEN = 2,
    PENDING = 3
}
declare class Channel extends Uint8ArrayE implements Types.Channel {
    moment?: Moment;
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        balance: ChannelBalance;
        status: ChannelStatus;
        moment?: Moment;
    });
    get balance(): ChannelBalance;
    get stateCounter(): number;
    get status(): ChannelStatus;
    get hash(): Promise<import("./hash").default>;
    sign(privKey: Uint8Array, pubKey: Uint8Array, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<Signature>;
    static get SIZE(): number;
    static createFunded(balance: ChannelBalance): Channel;
    static createActive(balance: ChannelBalance): Channel;
    static createPending(moment: Moment, balance: ChannelBalance): Channel;
}
export default Channel;
