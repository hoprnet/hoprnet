"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const secp256k1_1 = __importDefault(require("secp256k1"));
const types_1 = require("../types");
const extended_1 = require("../types/extended");
const utils_1 = require("../utils");
class SignedTicket extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr == null) {
            super(SignedTicket.SIZE);
        }
        else {
            super(arr.bytes, arr.offset, SignedTicket.SIZE);
        }
        if (struct != null) {
            if (struct.signature != null) {
                this.set(struct.signature, this.signatureOffset - this.byteOffset);
            }
            if (struct.ticket != null) {
                const ticket = struct.ticket.toU8a();
                if (ticket.length == types_1.Ticket.SIZE) {
                    this.set(ticket, this.ticketOffset - this.byteOffset);
                }
                else if (ticket.length < types_1.Ticket.SIZE) {
                    this.set(hopr_utils_1.u8aConcat(ticket, new Uint8Array(types_1.Ticket.SIZE - ticket.length)), this.ticketOffset - this.byteOffset);
                }
                else {
                    throw Error(`Ticket is too big by ${ticket.length - types_1.Ticket.SIZE} elements.`);
                }
            }
        }
    }
    get ticketOffset() {
        return this.byteOffset + types_1.Signature.SIZE;
    }
    get ticket() {
        if (this._ticket == null) {
            this._ticket = new types_1.Ticket({
                bytes: this.buffer,
                offset: this.ticketOffset,
            });
        }
        return this._ticket;
    }
    get signatureOffset() {
        return this.byteOffset;
    }
    get signature() {
        if (this._signature == null) {
            this._signature = new types_1.Signature({
                bytes: this.buffer,
                offset: this.signatureOffset,
            });
        }
        return this._signature;
    }
    get signer() {
        return new Promise(async (resolve, reject) => {
            try {
                resolve(secp256k1_1.default.ecdsaRecover(this.signature.signature, this.signature.recovery, await this.ticket.hash));
            }
            catch (err) {
                reject(err);
            }
        });
    }
    async verify(pubKey) {
        return utils_1.verify(await this.ticket.hash, this.signature, pubKey);
    }
    static get SIZE() {
        return types_1.Signature.SIZE + types_1.Ticket.SIZE;
    }
    static create(arr, struct) {
        return Promise.resolve(new SignedTicket(arr, struct));
    }
}
exports.default = SignedTicket;
//# sourceMappingURL=signedTicket.js.map