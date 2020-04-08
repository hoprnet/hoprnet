"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const _1 = require(".");
const extended_1 = require("../types/extended");
const utils_1 = require("../utils");
const u8a_1 = require("../core/u8a");
const WIN_PROB = new bn_js_1.default(1);
class Ticket extends extended_1.Uint8ArrayE {
    constructor(arr, struct) {
        if (arr != null && struct == null) {
            super(arr.bytes, arr.offset, Ticket.SIZE);
        }
        else if (arr == null && struct != null) {
            super(u8a_1.u8aConcat(new _1.Hash(struct.channelId).toU8a(), new _1.Hash(struct.challenge).toU8a(), new _1.TicketEpoch(struct.epoch).toU8a(), new _1.Balance(struct.amount).toU8a(), new _1.Hash(struct.winProb).toU8a(), new _1.Hash(struct.onChainSecret).toU8a()));
        }
        else {
            throw Error(`Invalid constructor arguments.`);
        }
    }
    get channelId() {
        return new _1.Hash(this.subarray(0, _1.Hash.SIZE));
    }
    get challenge() {
        return new _1.Hash(this.subarray(_1.Hash.SIZE, _1.Hash.SIZE + _1.Hash.SIZE));
    }
    get epoch() {
        const start = _1.Hash.SIZE + _1.Hash.SIZE;
        return new _1.TicketEpoch(this.subarray(start, start + _1.TicketEpoch.SIZE));
    }
    get amount() {
        const start = _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE;
        return new _1.Balance(this.subarray(start, start + _1.Balance.SIZE));
    }
    get winProb() {
        const start = _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE + _1.Balance.SIZE;
        return new _1.Hash(this.subarray(start, start + _1.Hash.SIZE));
    }
    get onChainSecret() {
        const start = _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE + _1.Balance.SIZE + _1.Hash.SIZE;
        return new _1.Hash(this.subarray(start, start + _1.Hash.SIZE));
    }
    getEmbeddedFunds() {
        return this.amount.mul(new bn_js_1.default(this.winProb)).div(new bn_js_1.default(new Uint8Array(_1.Hash.SIZE).fill(0xff)));
    }
    get hash() {
        return utils_1.hash(u8a_1.u8aConcat(this.challenge, this.onChainSecret, this.epoch.toU8a(), new Uint8Array(this.amount.toNumber()), this.winProb));
    }
    static get SIZE() {
        return _1.Hash.SIZE + _1.Hash.SIZE + _1.TicketEpoch.SIZE + _1.Balance.SIZE + _1.Hash.SIZE + _1.Hash.SIZE;
    }
    static async create(channel, amount, challenge) {
        const account = await channel.coreConnector.utils.pubKeyToAccountId(channel.counterparty);
        const { hashedSecret } = await channel.coreConnector.hoprChannels.methods
            .accounts(u8a_1.u8aToHex(account))
            .call();
        const winProb = new extended_1.Uint8ArrayE(new bn_js_1.default(new Uint8Array(_1.Hash.SIZE).fill(0xff)).div(WIN_PROB).toArray('le', _1.Hash.SIZE));
        const channelId = await channel.channelId;
        const ticket = new Ticket(undefined, {
            channelId: channelId,
            challenge: challenge,
            epoch: new _1.TicketEpoch(0),
            amount: new _1.Balance(amount.toString()),
            winProb: winProb,
            onChainSecret: new extended_1.Uint8ArrayE(u8a_1.stringToU8a(hashedSecret)),
        });
        const signature = await utils_1.sign(await ticket.hash, channel.coreConnector.self.privateKey).then(res => new _1.Signature({
            bytes: res.buffer,
            offset: res.byteOffset
        }));
        return new _1.SignedTicket(undefined, {
            signature,
            ticket
        });
    }
    static async verify(channel, signedTicket) {
        // @TODO: check if this is needed
        // if ((await channel.currentBalanceOfCounterparty).add(signedTicket.ticket.amount).lt(await channel.balance)) {
        //   return false
        // }
        try {
            await channel.testAndSetNonce(signedTicket);
        }
        catch {
            return false;
        }
        return utils_1.verify(await signedTicket.ticket.hash, signedTicket.signature, await channel.offChainCounterparty);
    }
    // @TODO: implement submit
    static async submit(channel, signedTicket) {
        throw Error('not implemented');
    }
}
exports.default = Ticket;
//# sourceMappingURL=ticket.js.map