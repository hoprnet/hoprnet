"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const extended_1 = require("../types/extended");
const balance_1 = __importDefault(require("./balance"));
class ChannelBalance extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null) {
            super(arr.bytes, arr.offset, ChannelBalance.SIZE);
        }
        else {
            super(ChannelBalance.SIZE);
        }
        if (struct != null) {
            if (struct.balance != null) {
                this.set(new balance_1.default(struct.balance.toString()).toU8a(), this.balanceOffset - this.byteOffset);
            }
            if (struct.balance_a != null) {
                this.set(new balance_1.default(struct.balance_a.toString()).toU8a(), this.balanceAOffset - this.byteOffset);
            }
        }
    }
    get balanceOffset() {
        return this.byteOffset;
    }
    get balance() {
        return new balance_1.default(new Uint8Array(this.buffer, this.balanceOffset, balance_1.default.SIZE));
    }
    get balanceAOffset() {
        return this.byteOffset + balance_1.default.SIZE;
    }
    get balance_a() {
        return new balance_1.default(new Uint8Array(this.buffer, this.balanceAOffset, balance_1.default.SIZE));
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