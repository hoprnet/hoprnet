export declare type Networks = 'mainnet' | 'morden' | 'ropsten' | 'rinkeby' | 'goerli' | 'kovan' | 'private';
export declare const HOPR_TOKEN: {
    [key in Networks]: string;
};
export declare const HOPR_CHANNELS: {
    [key in Networks]: string;
};
export declare const HOPR_MINTER: {
    [key in Networks]: string;
};
