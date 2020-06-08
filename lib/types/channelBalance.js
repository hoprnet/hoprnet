"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const extended_1 = require("../types/extended");
const balance_1 = __importDefault(require("./balance"));
class ChannelBalance extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, ChannelBalance.SIZE);
        }
        else if (arr == null && struct != null) {
            super(hopr_utils_1.u8aConcat(new balance_1.default(struct.balance.toString()).toU8a(), new balance_1.default(struct.balance_a.toString()).toU8a()));
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get balance() {
        return new balance_1.default(this.subarray(0, balance_1.default.SIZE));
    }
    get balance_a() {
        return new balance_1.default(this.subarray(balance_1.default.SIZE, balance_1.default.SIZE + balance_1.default.SIZE));
    }
    static get SIZE() {
        return balance_1.default.SIZE + balance_1.default.SIZE;
    }
    static create(arr, struct) {
        return new ChannelBalance(arr, struct);
    }
}
exports.default = ChannelBalance;
//# sourceMappingURL=channelBalance.js.map