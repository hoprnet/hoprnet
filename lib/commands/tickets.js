"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const chalk_1 = __importDefault(require("chalk"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
class Tickets {
    constructor(node) {
        this.node = node;
    }
    /**
     * @param query channelId string to send message to
     */
    async execute(query) {
        if (!query) {
            console.log(chalk_1.default.red('This command takes a channel ID as a parameter'));
            return;
        }
        const { Balance } = this.node.paymentChannels.types;
        const signedTickets = await this.node.paymentChannels.tickets.get(hopr_utils_1.stringToU8a(query));
        if (signedTickets.size === 0) {
            console.log(chalk_1.default.yellow(`\nNo tickets found.`));
            return;
        }
        const result = Array.from(signedTickets.values()).reduce((result, signedTicket) => {
            result.tickets.push({
                'amount (HOPR)': hopr_utils_1.moveDecimalPoint(signedTicket.ticket.amount.toString(), Balance.DECIMALS * -1).toString(),
            });
            result.total = result.total.add(signedTicket.ticket.amount);
            return result;
        }, {
            tickets: [],
            total: new bn_js_1.default(0),
        });
        console.table(result.tickets);
        console.log('Found', result.tickets.length, 'unredeemed tickets in channel ID', chalk_1.default.blue(query));
        console.log('You will receive', chalk_1.default.yellow(hopr_utils_1.moveDecimalPoint(result.total.toString(), Balance.DECIMALS * -1).toString()), 'HOPR', 'once you redeem them.');
    }
    complete(line, cb, query) {
        this.node.paymentChannels.channel.getAll(async (channel) => hopr_utils_1.u8aToHex(await channel.channelId), async (channelIdsPromise) => {
            let channelIds = [];
            try {
                channelIds = await Promise.all(channelIdsPromise);
            }
            catch (err) {
                console.log(chalk_1.default.red(err.message));
                return cb(undefined, [[''], line]);
            }
            if (channelIds.length < 1) {
                console.log(chalk_1.default.red(`\nNo open channels found.`));
                return cb(undefined, [[''], line]);
            }
            const hits = query ? channelIds.filter((channelId) => channelId.startsWith(query)) : channelIds;
            return cb(undefined, [hits.length ? hits.map((str) => `tickets ${str}`) : ['tickets'], line]);
        });
    }
}
exports.default = Tickets;
//# sourceMappingURL=tickets.js.map