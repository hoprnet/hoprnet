"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const secp256k1_1 = __importDefault(require("secp256k1"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const _1 = require(".");
const extended_1 = require("../types/extended");
const utils_1 = require("../utils");
class SignedChannel extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, SignedChannel.SIZE);
        }
        else if (arr == null && struct != null) {
            super(hopr_utils_1.u8aConcat(struct.signature, struct.channel));
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get signature() {
        if (this._signature == null) {
            const signature = this.subarray(0, _1.Signature.SIZE);
            this._signature = new _1.Signature({
                bytes: signature.buffer,
                offset: signature.byteOffset
            });
        }
        return this._signature;
    }
    get channel() {
        if (this._channel == null) {
            const channel = this.subarray(_1.Signature.SIZE, _1.Signature.SIZE + _1.Channel.SIZE);
            this._channel = new _1.Channel({
                bytes: channel.buffer,
                offset: channel.byteOffset
            });
        }
        return this._channel;
    }
    get signer() {
        return this.channel.hash.then(channelHash => {
            return secp256k1_1.default.ecdsaRecover(this.signature.signature, this.signature.recovery, channelHash);
        });
    }
    async verify(coreConnector) {
        return await utils_1.verify(this.channel.toU8a(), this.signature, coreConnector.self.publicKey);
    }
    static get SIZE() {
        return _1.Signature.SIZE + _1.Channel.SIZE;
    }
    static async create(coreConnector, arr, struct) {
        const emptySignatureArray = new Uint8Array(_1.Signature.SIZE).fill(0x00);
        let signedChannel;
        if (typeof arr !== "undefined") {
            signedChannel = new SignedChannel(arr);
        }
        else if (typeof struct !== "undefined") {
            signedChannel = new SignedChannel(undefined, {
                channel: struct.channel,
                signature: struct.signature || new _1.Signature({
                    bytes: emptySignatureArray.buffer,
                    offset: emptySignatureArray.byteOffset
                })
            });
        }
        else {
            throw Error(`Invalid input parameters.`);
        }
        if (signedChannel.signature.eq(emptySignatureArray)) {
            signedChannel.set(await utils_1.sign(await signedChannel.channel.hash, coreConnector.self.privateKey), 0);
        }
        return signedChannel;
    }
}
exports.default = SignedChannel;
