export declare const PADDING: Uint8Array;
export default class Message extends Uint8Array {
    encrypted: boolean;
    constructor(_encrypted: boolean, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    });
    static get SIZE(): number;
    subarray(begin?: number, end?: number): Uint8Array;
    getCopy(): Message;
    get plaintext(): Uint8Array;
    get ciphertext(): Uint8Array;
    static create(msg: Uint8Array, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }): Message;
    onionEncrypt(secrets: Uint8Array[]): Message;
    decrypt(secret: Uint8Array): Message;
}
