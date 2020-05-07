import Web3 from 'web3';
import CoreConnector from '..';
import { Hash, AccountId } from '../types';
import { HoprToken } from '../tsc/web3/HoprToken';
import { Await } from '../tsc/utils';
/**
 * Return private key data like public key and address
 * @param _privKey private key to derive data from
 */
export declare function getPrivKeyData(_privKey: Uint8Array): Promise<{
    privKey: Hash;
    pubKey: Hash;
    address: AccountId;
}>;
/**
 * Given web3 instance, and hoprToken instance, generate a new user and send funds to it.
 * @param web3 the web3 instance the our hoprToken contract is deployed to
 * @param funder object
 * @param hoprToken the hoprToken instance that will be used to mint tokens
 * @returns user object
 */
export declare function generateUser(web3: Web3, funder: Await<ReturnType<typeof getPrivKeyData>>, hoprToken: HoprToken): Promise<{
    privKey: Hash;
    pubKey: Hash;
    address: AccountId;
}>;
/**
 * Given a private key, generate a connector node.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export declare function generateNode(privKey: Uint8Array): Promise<CoreConnector>;
