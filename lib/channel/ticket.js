"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const types_1 = require("../types");
const extended_1 = require("../types/extended");
const DEFAULT_WIN_PROB = new bn_js_1.default(1);
class TicketFactory {
    constructor(channel) {
        this.channel = channel;
    }
    async create(amount, challenge, arr) {
        const account = await this.channel.coreConnector.utils.pubKeyToAccountId(this.channel.counterparty);
        const { hashedSecret } = await this.channel.coreConnector.hoprChannels.methods.accounts(hopr_utils_1.u8aToHex(account)).call();
        const winProb = new extended_1.Uint8ArrayE(new bn_js_1.default(new Uint8Array(types_1.Hash.SIZE).fill(0xff)).div(DEFAULT_WIN_PROB).toArray('le', types_1.Hash.SIZE));
        const channelId = await this.channel.channelId;
        const signedTicket = new types_1.SignedTicket(arr);
        const ticket = new types_1.Ticket({
            bytes: signedTicket.buffer,
            offset: signedTicket.ticketOffset,
        }, {
            channelId,
            challenge,
            // @TODO set this dynamically
            epoch: new types_1.TicketEpoch(0),
            amount: new types_1.Balance(amount.toString()),
            winProb,
            onChainSecret: new extended_1.Uint8ArrayE(hopr_utils_1.stringToU8a(hashedSecret)),
        });
        await ticket.sign(this.channel.coreConnector.account.keys.onChain.privKey, undefined, {
            bytes: signedTicket.buffer,
            offset: signedTicket.signatureOffset,
        });
        return signedTicket;
    }
    async verify(signedTicket) {
        // @TODO: check if this is needed
        // if ((await channel.currentBalanceOfCounterparty).add(signedTicket.ticket.amount).lt(await channel.balance)) {
        //   return false
        // }
        try {
            await this.channel.testAndSetNonce(signedTicket);
        }
        catch {
            return false;
        }
        return await signedTicket.verify(await this.channel.offChainCounterparty);
    }
    // @TODO: implement submit
    async submit(signedTicket) {
        throw Error('not implemented');
    }
}
exports.default = TicketFactory;
//# sourceMappingURL=ticket.js.map