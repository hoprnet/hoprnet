"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const secp256k1_1 = __importDefault(require("secp256k1"));
const _1 = require(".");
const extended_1 = require("../types/extended");
const u8a_1 = require("../core/u8a");
class SignedTicket extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, SignedTicket.SIZE);
        }
        else if (arr == null && struct != null) {
            const ticket = struct.ticket.toU8a();
            if (ticket.length == _1.Ticket.SIZE) {
                super(u8a_1.u8aConcat(struct.signature, ticket));
            }
            else if (ticket.length < _1.Ticket.SIZE) {
                super(u8a_1.u8aConcat(struct.signature, ticket, new Uint8Array(_1.Ticket.SIZE - ticket.length)));
            }
            else {
                throw Error(`Ticket is too big by ${ticket.length - _1.Ticket.SIZE} elements.`);
            }
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get ticket() {
        if (this._ticket == null) {
            const ticket = this.subarray(_1.Signature.SIZE, _1.Signature.SIZE + _1.Ticket.SIZE);
            this._ticket = new _1.Ticket({
                bytes: ticket.buffer,
                offset: ticket.byteOffset
            });
        }
        return this._ticket;
    }
    get signature() {
        if (this._signature == null) {
            this._signature = new _1.Signature({
                bytes: this.buffer,
                offset: this.byteOffset
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
