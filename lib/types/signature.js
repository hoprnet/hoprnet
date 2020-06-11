"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const extended_1 = require("../types/extended");
const constants_1 = require("../constants");
class Signature extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr == null) {
            super(Signature.SIZE);
        }
        else {
            super(arr.bytes, arr.offset, Signature.SIZE);
        }
        if (struct != null) {
            this.set(struct.signature, this.signatureOffset - this.byteOffset);
            this.set([struct.recovery], this.recoveryOffset - this.byteOffset);
        }
    }
    get signatureOffset() {
        return this.byteOffset;
    }
    get signature() {
        return new Uint8Array(this.buffer, this.signatureOffset, constants_1.SIGNATURE_LENGTH);
    }
    get recoveryOffset() {
        return this.byteOffset + constants_1.SIGNATURE_LENGTH;
    }
    get recovery() {
        return this[this.recoveryOffset - this.byteOffset];
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
        return Promise.resolve(new Signature(arr, struct));
    }
}
exports.default = Signature;
//# sourceMappingURL=signature.js.map