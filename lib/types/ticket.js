"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const hash_1 = __importDefault(require("./hash"));
const ticketEpoch_1 = __importDefault(require("./ticketEpoch"));
const balance_1 = __importDefault(require("./balance"));
const extended_1 = require("../types/extended");
const utils_1 = require("../utils");
class Ticket extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr == null && struct == null) {
            throw Error(`Invalid constructor arguments.`);
        }
        if (arr == null) {
            super(Ticket.SIZE);
        }
        else {
            super(arr.bytes, arr.offset, Ticket.SIZE);
        }
        if (struct != null) {
            this.set(struct.channelId, this.channelIdOffset - this.byteOffset);
            this.set(struct.challenge, this.challengeOffset - this.byteOffset);
            this.set(struct.epoch.toU8a(), this.epochOffset - this.byteOffset);
            this.set(struct.amount.toU8a(), this.amountOffset - this.byteOffset);
            this.set(struct.winProb, this.winProbOffset - this.byteOffset);
            this.set(struct.onChainSecret, this.onChainSecretOffset - this.byteOffset);
        }
    }
    get channelIdOffset() {
        return this.byteOffset;
    }
    get channelId() {
        return new hash_1.default(new Uint8Array(this.buffer, this.channelIdOffset, hash_1.default.SIZE));
    }
    get challengeOffset() {
        return this.byteOffset + hash_1.default.SIZE;
    }
    get challenge() {
        return new hash_1.default(new Uint8Array(this.buffer, this.challengeOffset, hash_1.default.SIZE));
    }
    get epochOffset() {
        return this.byteOffset + hash_1.default.SIZE + hash_1.default.SIZE;
    }
    get epoch() {
        return new ticketEpoch_1.default(new Uint8Array(this.buffer, this.epochOffset, ticketEpoch_1.default.SIZE));
    }
    get amountOffset() {
        return this.byteOffset + hash_1.default.SIZE + hash_1.default.SIZE + ticketEpoch_1.default.SIZE;
    }
    get amount() {
        return new balance_1.default(new Uint8Array(this.buffer, this.amountOffset, balance_1.default.SIZE));
    }
    get winProbOffset() {
        return this.byteOffset + hash_1.default.SIZE + hash_1.default.SIZE + ticketEpoch_1.default.SIZE + balance_1.default.SIZE;
    }
    get winProb() {
        return new hash_1.default(new Uint8Array(this.buffer, this.winProbOffset, hash_1.default.SIZE));
    }
    get onChainSecretOffset() {
        return this.byteOffset + hash_1.default.SIZE + hash_1.default.SIZE + ticketEpoch_1.default.SIZE + balance_1.default.SIZE + hash_1.default.SIZE;
    }
    get onChainSecret() {
        return new hash_1.default(new Uint8Array(this.buffer, this.onChainSecretOffset, hash_1.default.SIZE));
    }
    get hash() {
        return utils_1.hash(this);
    }
    static get SIZE() {
        return hash_1.default.SIZE + hash_1.default.SIZE + ticketEpoch_1.default.SIZE + balance_1.default.SIZE + hash_1.default.SIZE + hash_1.default.SIZE;
    }
    getEmbeddedFunds() {
        return this.amount.mul(new bn_js_1.default(this.winProb)).div(new bn_js_1.default(new Uint8Array(hash_1.default.SIZE).fill(0xff)));
    }
}
exports.default = Ticket;
//# sourceMappingURL=ticket.js.map