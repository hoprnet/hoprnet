import { Networks } from './tsc/types';
export declare const DEFAULT_URI = "ws://127.0.0.1:9545/";
export declare const TOKEN_ADDRESSES: {
    [key in Networks]: string;
};
export declare const CHANNELS_ADDRESSES: {
    [key in Networks]: string;
};
export declare const FUND_ACCOUNT_PRIVATE_KEY: string;
export declare const DEMO_ACCOUNTS: string[];
export declare const MAX_CONFIRMATIONS = 8;
