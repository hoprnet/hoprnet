import Web3 from 'web3';
import CoreConnector from '..';
import { Hash, AccountId } from '../types';
import { HoprToken } from '../tsc/web3/HoprToken';
export declare type Account = {
    privKey: Hash;
    pubKey: Hash;
    address: AccountId;
};
/**
 * Return private key data like public key and address.
 * @param _privKey private key to derive data from
 */
export declare function getPrivKeyData(_privKey: Uint8Array): Promise<Account>;
/**
 * Fund an account.
 * @param web3 the web3 instance the our hoprToken contract is deployed to
 * @param hoprToken the hoprToken instance that will be used to mint tokens
 * @param funder object
 * @param account object
 */
export declare function fundAccount(web3: Web3, hoprToken: HoprToken, funder: Account, account: Account): Promise<void>;
/**
 * Create a random account.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export declare function createAccount(): Promise<Account>;
/**
 * Create a random account or use provided one, and then fund it.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export declare function createAccountAndFund(web3: Web3, hoprToken: HoprToken, funder: Account, account?: string | Uint8Array | Account): Promise<Account>;
/**
 * Given a private key, create a connector node.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export declare function createNode(privKey: Uint8Array, debug?: boolean): Promise<CoreConnector>;
/**
 * Disconnect web3 as if it lost connection
 * @param web3 Web3 instance
 */
export declare function disconnectWeb3(web3: Web3): Promise<void>;
