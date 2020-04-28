/// <reference types="node" />
import Hopr from '../../..';
import PeerId from 'peer-id';
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
export declare type CipherParameters = {
    key: Uint8Array;
    iv: Uint8Array;
};
export declare type PRGParameters = {
    key: Uint8Array;
    iv: Uint8Array;
};
export declare class Header<Chain extends HoprCoreConnector> extends Uint8Array {
    tmpData?: Uint8Array;
    derivedSecretLastNode?: Uint8Array;
    constructor(arr: {
        bytes: ArrayBuffer;
        offset: number;
    });
    subarray(begin?: number, end?: number): Uint8Array;
    get alpha(): Uint8Array;
    get beta(): Uint8Array;
    get gamma(): Uint8Array;
    get address(): this['tmpData'];
    get identifier(): this['tmpData'];
    get hashedKeyHalf(): this['tmpData'];
    get encryptionKey(): this['tmpData'];
    get derivedSecret(): this['tmpData'];
    deriveSecret(secretKey: Uint8Array, lastNode?: boolean): void;
    verify(): boolean;
    extractHeaderInformation(lastNode?: boolean): void;
    transformForNextNode(): void;
    toString(): string;
    static get SIZE(): number;
    static create<Chain extends HoprCoreConnector>(node: Hopr<Chain>, peerIds: PeerId[], arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Promise<{
        header: Header<Chain>;
        secrets: Uint8Array[];
        identifier: Uint8Array;
    }>;
}
export declare const BETA_LENGTH: number;
export declare function deriveTagParameters(secret: Uint8Array): Uint8Array;
export declare function deriveCipherParameters(secret: Uint8Array): CipherParameters;
export declare function derivePRGParameters(secret: Uint8Array): PRGParameters;
export declare function deriveBlinding(alpha: Uint8Array, secret: Uint8Array): Uint8Array;
export declare function deriveTicketKey(secret: Uint8Array): Uint8Array;
export declare function deriveTicketKeyBlinding(secret: Uint8Array): Uint8Array;
export declare function deriveTicketLastKey(secret: Uint8Array): Uint8Array;
export declare function deriveTicketLastKeyBlinding(secret: Uint8Array): Buffer;
export declare function createMAC(secret: Uint8Array, msg: Uint8Array): Uint8Array;
