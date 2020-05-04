import BN from 'bn.js';
import { BNE, Uint8ArrayE } from '../types/extended';
declare class ChannelEntry extends Uint8ArrayE {
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        blockNumber: BN;
        transactionIndex: BN;
        logIndex: BN;
    });
    get blockNumber(): BNE;
    get transactionIndex(): BNE;
    get logIndex(): BNE;
    static get SIZE(): number;
}
export default ChannelEntry;
