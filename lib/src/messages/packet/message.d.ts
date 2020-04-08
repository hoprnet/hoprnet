export declare const PADDING: Uint8Array;
export default class Message extends Uint8Array {
    encrypted: boolean;
    constructor(arr: {
        bytes: ArrayBuffer;
        offset: number;
    }, encrypted: boolean);
    static get SIZE(): number;
    subarray(begin?: number, end?: number): Uint8Array;
    get plaintext(): Uint8Array;
    get ciphertext(): Uint8Array;
    static createEncrypted(msg: Uint8Array): Message;
    static createPlain(msg: Uint8Array | string): Message;
    onionEncrypt(secrets: Uint8Array[]): Message;
    decrypt(secret: Uint8Array): Message;
}
