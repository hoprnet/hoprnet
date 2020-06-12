"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const types_1 = require("../types");
const DEFAULT_WIN_PROB = new bn_js_1.default(1);
class TicketFactory {
    constructor(channel) {
        this.channel = channel;
    }
    async create(amount, challenge, arr) {
        const winProb = new types_1.Hash(new bn_js_1.default(new Uint8Array(types_1.Hash.SIZE).fill(0xff)).div(DEFAULT_WIN_PROB).toArray('le', types_1.Hash.SIZE));
        const channelId = await this.channel.channelId;
        const signedTicket = new types_1.SignedTicket(arr);
        const ticket = new types_1.Ticket({
            bytes: signedTicket.buffer,
            offset: signedTicket.ticketOffset,
        }, {
            channelId,
            challenge,
            epoch: await this.channel.coreConnector.account.ticketEpoch,
            amount: new types_1.Balance(amount.toString()),
            winProb,
            onChainSecret: new types_1.Hash(),
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
    async submit(signedTicket) {
        const { hoprChannels, signTransaction, account, utils } = this.channel.coreConnector;
        const { ticket, signature } = signedTicket;
        const { r, s, v } = utils.getSignatureParameters(signature);
        const counterPartySecret = hopr_utils_1.u8aXOR(false, ticket.challenge, ticket.onChainSecret);
        const transaction = await signTransaction(hoprChannels.methods.redeemTicket(hopr_utils_1.u8aToHex(ticket.challenge), hopr_utils_1.u8aToHex(ticket.onChainSecret), hopr_utils_1.u8aToHex(counterPartySecret), ticket.amount.toString(), hopr_utils_1.u8aToHex(ticket.winProb), hopr_utils_1.u8aToHex(r), hopr_utils_1.u8aToHex(s), v), {
            from: (await account.address).toHex(),
            to: hoprChannels.options.address,
            nonce: (await account.nonce).valueOf(),
        });
        const receipt = await transaction.send();
        console.log(receipt);
    }
}
exports.default = TicketFactory;
//# sourceMappingURL=ticket.js.map