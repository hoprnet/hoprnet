"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PADDING = void 0;
const constants_1 = require("../../constants");
const header_1 = require("./header");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
exports.PADDING = new TextEncoder().encode('PADDING');
class Message extends Uint8Array {
    constructor(_encrypted, arr) {
        if (arr == null) {
            super(Message.SIZE);
        }
        else {
            super(arr.bytes, arr.offset, constants_1.PACKET_SIZE);
        }
        this.encrypted = _encrypted;
    }
    static get SIZE() {
        return constants_1.PACKET_SIZE;
    }
    subarray(begin = 0, end = Message.SIZE) {
        return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin);
    }
    getCopy() {
        const msgCopy = new Message(this.encrypted);
        msgCopy.set(this);
        return msgCopy;
    }
    get plaintext() {
        if (this.encrypted) {
            throw Error(`Cannot read encrypted data.`);
        }
        return hopr_utils_1.lengthPrefixedToU8a(this, exports.PADDING, constants_1.PACKET_SIZE);
    }
    get ciphertext() {
        if (!this.encrypted) {
            throw Error(`Message is unencrypted. Cannot read encrypted data.`);
        }
        return this;
    }
    static create(msg, arr) {
        const message = new Message(false, arr);
        message.set(hopr_utils_1.toLengthPrefixedU8a(msg, exports.PADDING, constants_1.PACKET_SIZE));
        return message;
    }
    onionEncrypt(secrets) {
        if (!Array.isArray(secrets) || secrets.length <= 0) {
            throw Error('Invald input arguments. Expected array with at least one secret key.');
        }
        this.encrypted = true;
        for (let i = secrets.length; i > 0; i--) {
            const { key, iv } = header_1.deriveCipherParameters(secrets[i - 1]);
            hopr_utils_1.PRP.createPRP(key, iv).permutate(this);
        }
        return this;
    }
    decrypt(secret) {
        const { key, iv } = header_1.deriveCipherParameters(secret);
        hopr_utils_1.PRP.createPRP(key, iv).inverse(this);
        return this;
    }
}
exports.default = Message;
//# sourceMappingURL=message.js.map