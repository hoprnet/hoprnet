import Web3 from 'web3';
import { LevelUp } from 'levelup';
import HoprCoreConnector, { Types as ITypes, Channel as IChannel, Constants as IConstants } from '@hoprnet/hopr-core-connector-interface';
import Ticket from './ticket';
import * as dbkeys from './dbKeys';
import * as types from './types';
import * as utils from './utils';
import * as constants from './constants';
import { Networks } from './tsc/types';
import { HoprChannels } from './tsc/web3/HoprChannels';
import { HoprToken } from './tsc/web3/HoprToken';
export default class HoprEthereum implements HoprCoreConnector {
    db: LevelUp;
    self: {
        privateKey: Uint8Array;
        publicKey: Uint8Array;
        onChainKeyPair: {
            privateKey?: Uint8Array;
            publicKey?: Uint8Array;
        };
    };
    account: types.AccountId;
    web3: Web3;
    network: Networks;
    hoprChannels: HoprChannels;
    hoprToken: HoprToken;
    private _status;
    private _initializing;
    private _starting;
    private _stopping;
    private _nonce?;
    signTransaction: ReturnType<typeof utils.TransactionSigner>;
    log: ReturnType<typeof utils['Log']>;
    constructor(db: LevelUp, self: {
        privateKey: Uint8Array;
        publicKey: Uint8Array;
        onChainKeyPair: {
            privateKey?: Uint8Array;
            publicKey?: Uint8Array;
        };
    }, account: types.AccountId, web3: Web3, network: Networks, hoprChannels: HoprChannels, hoprToken: HoprToken);
    readonly dbKeys: typeof dbkeys;
    readonly utils: typeof utils;
    readonly types: typeof ITypes;
    readonly constants: typeof constants;
    readonly channel: typeof IChannel;
    readonly ticket: typeof Ticket;
    readonly CHAIN_NAME = "HOPR on Ethereum";
    get nonce(): Promise<number>;
    get accountBalance(): Promise<types.Balance>;
    get accountNativeBalance(): Promise<types.NativeBalance>;
    start(): Promise<void>;
    stop(): Promise<void>;
    get started(): boolean;
    initOnchainValues(nonce?: number): Promise<void>;
    initialize(): Promise<void>;
    initializeAccountSecret(): Promise<boolean>;
    checkAccountSecret(): Promise<boolean>;
    setAccountSecret(nonce?: number): Promise<void>;
    checkWeb3(): Promise<boolean>;
    static readonly constants: typeof IConstants;
    static create(db: LevelUp, seed?: Uint8Array, options?: {
        id?: number;
        provider?: string;
    }): Promise<HoprEthereum>;
}
export declare const Types: typeof types;
export declare const Utils: typeof utils;
