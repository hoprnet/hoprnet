import BN from 'bn.js';
import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { Uint8ArrayE } from '../types/extended';
import Balance from './balance';
declare class ChannelBalance extends Uint8ArrayE implements Types.ChannelBalance {
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        balance: BN | Balance;
        balance_a: BN | Balance;
    });
    get balanceOffset(): number;
    get balance(): Balance;
    get balanceAOffset(): number;
    get balance_a(): Balance;
    static get SIZE(): number;
    static create(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        balance: Balance;
        balance_a: Balance;
    }): ChannelBalance;
}
export default ChannelBalance;
