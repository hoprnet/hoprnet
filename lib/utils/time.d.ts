import BN from 'bn.js';
import Web3 from 'web3';
export declare function advanceBlock(web3: Web3): any;
export declare function advanceBlockTo(web3: Web3, target: BN | number): Promise<void>;
export declare function latest(web3: Web3): Promise<BN>;
export declare function latestBlock(web3: Web3): Promise<BN>;
export declare function increase(web3: Web3, duration: BN | number): Promise<void>;
/**
 * Beware that due to the need of calling two separate ganache methods and rpc calls overhead
 * it's hard to increase time precisely to a target point so design your test to tolerate
 * small fluctuations from time to time.
 *
 * @param target time in seconds
 */
export declare function increaseTo(web3: Web3, target: BN | number): Promise<void>;
