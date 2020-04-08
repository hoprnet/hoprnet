import Web3 from 'web3';
import CoreConnector from '..';
import { Hash, AccountId } from '../types';
import { HoprToken } from '../tsc/web3/HoprToken';
import { Await } from '../tsc/utils';
export declare function getPrivKeyData(_privKey: Uint8Array): Promise<{
    privKey: Hash;
    pubKey: Hash;
    address: AccountId;
}>;
export declare function generateUser(web3: Web3, funder: Await<ReturnType<typeof getPrivKeyData>>, hoprToken: HoprToken): Promise<{
    privKey: Hash;
    pubKey: Hash;
    address: AccountId;
}>;
export declare function generateNode(privKey: Uint8Array): Promise<CoreConnector>;
