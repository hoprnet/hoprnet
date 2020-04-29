"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const secp256k1_1 = __importDefault(require("secp256k1"));
const _1 = require(".");
const extended_1 = require("../types/extended");
class SignedTicket extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr == null) {
            super(SignedTicket.SIZE);
        }
        else {
            super(arr.bytes, arr.offset, SignedTicket.SIZE);
        }
        if (struct != null) {
            const ticket = struct.ticket.toU8a();
            this.set(struct.signature, this.signatureOffset - this.byteOffset);
            this._signature = struct.signature;
            if (ticket.length == _1.Ticket.SIZE) {
                this.set(struct.ticket, this.ticketOffset - this.byteOffset);
            }
            else if (ticket.length < _1.Ticket.SIZE) {
                this.set(hopr_utils_1.u8aConcat(ticket, new Uint8Array(_1.Ticket.SIZE - ticket.length)), this.ticketOffset - this.byteOffset);
            }
            else {
                throw Error(`Ticket is too big by ${ticket.length - _1.Ticket.SIZE} elements.`);
            }
            this._ticket = struct.ticket;
        }
    }
    get ticketOffset() {
        return this.byteOffset + _1.Signature.SIZE;
    }
    get ticket() {
        if (this._ticket == null) {
            this._ticket = new _1.Ticket({
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
            this._signature = new _1.Signature({
                bytes: this.buffer,
                offset: this.signatureOffset,
            });
        }
        return this._signature;
    }
    get signer() {
        return new Promise(async (resolve, reject) => {
            try {
                const signer = secp256k1_1.default.ecdsaRecover(this.signature.signature, this.signature.recovery, await this.ticket.hash);
                return resolve(signer);
            }
            catch (err) {
                return reject(err);
            }
        });
    }
    static get SIZE() {
        return _1.Signature.SIZE + _1.Ticket.SIZE;
    }
    static create(arr, struct) {
        return new SignedTicket(arr, struct);
    }
}
exports.default = SignedTicket;
