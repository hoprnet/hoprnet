import HoprEthereum from '.';
import { Hash } from './types';
export declare const GIANT_STEP_WIDTH = 10000;
export declare const TOTAL_ITERATIONS = 100000;
declare class HashedSecret {
    private coreConnector;
    constructor(coreConnector: HoprEthereum);
    create(): Promise<Hash>;
    getPreimage(hash: Hash): Promise<{
        preImage: Uint8Array;
        index: number;
    }>;
}
export default HashedSecret;
