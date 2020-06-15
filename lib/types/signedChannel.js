"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const secp256k1_1 = __importDefault(require("secp256k1"));
const signature_1 = __importDefault(require("./signature"));
const channel_1 = __importDefault(require("./channel"));
const extended_1 = require("../types/extended");
const utils_1 = require("../utils");
class SignedChannel extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr == null) {
            super(SignedChannel.SIZE);
        }
        else {
            super(arr.bytes, arr.offset, SignedChannel.SIZE);
        }
        if (struct != null) {
            if (struct.channel != null) {
                this.set(struct.channel.toU8a(), this.channelOffset - this.byteOffset);
            }
            if (struct.signature) {
                this.set(struct.signature, this.signatureOffset - this.byteOffset);
            }
        }
    }
    get signatureOffset() {
        return this.byteOffset;
    }
    get signature() {
        if (this._signature == null) {
            this._signature = new signature_1.default({
                bytes: this.buffer,
                offset: this.signatureOffset,
            });
        }
        return this._signature;
    }
    get channelOffset() {
        return this.byteOffset + signature_1.default.SIZE;
    }
    get channel() {
        if (this._channel == null) {
            this._channel = new channel_1.default({
                bytes: this.buffer,
                offset: this.channelOffset,
            });
        }
        return this._channel;
    }
    get signer() {
        return new Promise(async (resolve, reject) => {
            try {
                resolve(secp256k1_1.default.ecdsaRecover(this.signature.signature, this.signature.recovery, await this.channel.hash));
            }
            catch (err) {
                reject(err);
            }
        });
    }
    async verify(publicKey) {
        return await utils_1.verify(await this.channel.hash, this.signature, publicKey);
    }
    static get SIZE() {
        return signature_1.default.SIZE + channel_1.default.SIZE;
    }
    static create(arr, struct) {
        return Promise.resolve(new SignedChannel(arr, struct));
    }
}
exports.default = SignedChannel;
//# sourceMappingURL=signedChannel.js.map