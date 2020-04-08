/// <reference types="node" />
export * from './deserialize';
export * from './serialize';
export declare const KEYPAIR_CIPHER_ALGORITHM = "chacha20";
export declare const KEYPAIR_IV_LENGTH = 16;
export declare const KEYPAIR_CIPHER_KEY_LENGTH = 32;
export declare const KEYPAIR_SALT_LENGTH = 32;
export declare const KEYPAIR_SCRYPT_PARAMS: {
    N: number;
    r: number;
    p: number;
};
export declare const KEYPAIR_PADDING: Buffer;
export declare const KEYPAIR_MESSAGE_DIGEST_ALGORITHM = "sha256";
