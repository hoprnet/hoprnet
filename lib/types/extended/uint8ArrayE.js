"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
class Uint8ArrayE extends Uint8Array {
    subarray(begin = 0, end) {
        return new Uint8Array(this.buffer, begin + this.byteOffset, end != null ? end - begin : undefined);
    }
    toU8a() {
        return new Uint8Array(this);
    }
    toHex() {
        return hopr_utils_1.u8aToHex(this);
    }
    eq(b) {
        return hopr_utils_1.u8aEquals(this, b);
    }
}
exports.default = Uint8ArrayE;
