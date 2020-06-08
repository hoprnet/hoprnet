"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const secp256k1_1 = __importDefault(require("secp256k1"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const signature_1 = __importDefault(require("./signature"));
const channel_1 = __importDefault(require("./channel"));
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
            const signature = this.subarray(0, signature_1.default.SIZE);
            this._signature = new signature_1.default({
                bytes: signature.buffer,
                offset: signature.byteOffset,
            });
        }
        return this._signature;
    }
    get channel() {
        if (this._channel == null) {
            const channel = this.subarray(signature_1.default.SIZE, signature_1.default.SIZE + channel_1.default.SIZE);
            this._channel = new channel_1.default({
                bytes: channel.buffer,
                offset: channel.byteOffset,
            });
        }
        return this._channel;
    }
    get signer() {
        return this.channel.hash.then((channelHash) => {
            return secp256k1_1.default.ecdsaRecover(this.signature.signature, this.signature.recovery, channelHash);
        });
    }
    async verify(publicKey) {
        return await utils_1.verify(this.channel.toU8a(), this.signature, publicKey);
    }
    static get SIZE() {
        return signature_1.default.SIZE + channel_1.default.SIZE;
    }
}
exports.default = SignedChannel;
//# sourceMappingURL=signedChannel.js.map