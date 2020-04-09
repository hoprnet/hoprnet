"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const u8a_1 = require("../core/u8a");
const extended_1 = require("../types/extended");
const constants_1 = require("../constants");
class Signature extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, Signature.SIZE);
        }
        else if (arr == null && struct != null) {
            super(u8a_1.u8aConcat(struct.signature, new Uint8Array([struct.recovery])));
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get signature() {
        return this.subarray(0, constants_1.SIGNATURE_LENGTH);
    }
    get recovery() {
        return u8a_1.u8aToNumber(this.subarray(constants_1.SIGNATURE_LENGTH, constants_1.SIGNATURE_LENGTH + constants_1.SIGNATURE_RECOVERY_LENGTH));
    }
    get msgPrefix() {
        return new Uint8Array();
    }
    get onChainSignature() {
        return this.signature;
    }
    static get SIZE() {
        return constants_1.SIGNATURE_LENGTH + constants_1.SIGNATURE_RECOVERY_LENGTH;
    }
    static create(arr, struct) {
        return new Signature(arr, struct);
    }
}
exports.default = Signature;
