import HoprEthereum from '.';
import { Hash } from './types';
export declare const GIANT_STEP_WIDTH = 10000;
export declare const TOTAL_ITERATIONS = 100000;
declare class HashedSecret {
    private coreConnector;
    _onChainValuesInitialized: boolean;
    constructor(coreConnector: HoprEthereum);
    /**
     * generate and set account secret
     */
    submit(nonce?: number): Promise<void>;
    /**
     * Checks whether node has an account secret set onchain and offchain
     * @returns a promise resolved true if secret is set correctly
     */
    check(): Promise<void>;
    /**
     * Returns a deterministic secret that is used in debug mode.
     */
    private getDebugAccountSecret;
    /**
     * Creates the on-chain secret and stores the intermediate values
     * into the database.
     */
    create(): Promise<Hash>;
    /**
     * Tries to find a pre-image for the given hash by using the intermediate
     * values from the database.
     * @param hash the hash to find a preImage for
     */
    getPreimage(hash: Hash): Promise<{
        preImage: Hash;
        index: number;
    }>;
}
export default HashedSecret;
