"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const _1 = require(".");
const extended_1 = require("../types/extended");
const utils_1 = require("../utils");
//
const web3_1 = __importDefault(require("web3"));
const web3 = new web3_1.default();
/**
 * Given a message, prefix it with "\x19Ethereum Signed Message:\n" and return it's hash
 * @param msg the message to hash
 * @returns a hash
 */
function toEthSignedMessageHash(msg) {
    return new _1.Hash(hopr_utils_1.stringToU8a(web3.eth.accounts.hashMessage(msg)));
}
function encode(items) {
    const { types, values } = items.reduce((result, item) => {
        result.types.push(item.type);
        result.values.push(item.value);
        return result;
    }, {
        types: [],
        values: [],
    });
    return web3.eth.abi.encodeParameters(types, values);
}
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
        return new _1.Hash(new Uint8Array(this.buffer, this.channelIdOffset, _1.Hash.SIZE));
    }
    get challengeOffset() {
        return this.byteOffset + _1.Hash.SIZE;
    }
    get challenge() {
        return new _1.Hash(new Uint8Array(this.buffer, this.challengeOffset, _1.Hash.SIZE));
    }
    get epochOffset() {
        return this.byteOffset + _1.Hash.SIZE + _1.Hash.SIZE;
    }
    get epoch() {
        return new _1.TicketEpoch(new Uint8Array(this.buffer, this.epochOffset, _1.TicketEpoch.SIZE));
    }
    get amountOffset() {
        return this.byteOffset + _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE;
    }
    get amount() {
        return new _1.Balance(new Uint8Array(this.buffer, this.amountOffset, _1.Balance.SIZE));
    }
    get winProbOffset() {
        return this.byteOffset + _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE + _1.Balance.SIZE;
    }
    get winProb() {
        return new _1.Hash(new Uint8Array(this.buffer, this.winProbOffset, _1.Hash.SIZE));
    }
    get onChainSecretOffset() {
        return this.byteOffset + _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE + _1.Balance.SIZE + _1.Hash.SIZE;
    }
    get onChainSecret() {
        return new _1.Hash(new Uint8Array(this.buffer, this.onChainSecretOffset, _1.Hash.SIZE));
    }
    get hash() {
        const encodedTicket = encode([
            { type: 'bytes32', value: hopr_utils_1.u8aToHex(this.channelId) },
            { type: 'bytes32', value: hopr_utils_1.u8aToHex(this.challenge) },
            { type: 'bytes32', value: hopr_utils_1.u8aToHex(this.onChainSecret) },
            { type: 'uint256', value: this.epoch.toString() },
            { type: 'uint256', value: this.amount.toString() },
            { type: 'bytes32', value: hopr_utils_1.u8aToHex(this.winProb) },
        ]);
        return toEthSignedMessageHash(encodedTicket);
    }
    static get SIZE() {
        return _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE + _1.Balance.SIZE + _1.Hash.SIZE + _1.Hash.SIZE;
    }
    getEmbeddedFunds() {
        return this.amount.mul(new bn_js_1.default(this.winProb)).div(new bn_js_1.default(new Uint8Array(_1.Hash.SIZE).fill(0xff)));
    }
    async sign(privKey, pubKey, arr) {
        return utils_1.sign(this.hash, privKey, undefined, arr);
    }
    static create(arr, struct) {
        return new Ticket(arr, struct);
    }
}
exports.default = Ticket;
//# sourceMappingURL=ticket.js.map