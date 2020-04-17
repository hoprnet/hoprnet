"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bignumber_js_1 = __importDefault(require("bignumber.js"));
const bn_js_1 = __importDefault(require("bn.js"));
const chalk_1 = __importDefault(require("chalk"));
const utils_1 = require("../utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const utils_2 = require("../../src/utils");
class OpenChannel {
    constructor(node) {
        this.node = node;
    }
    /**
     * Encapsulates the functionality that is executed once the user decides to open a payment channel
     * with another party.
     * @param query peerId string to send message to
     */
    async execute(rl, query) {
        if (query == null || query == '') {
            console.log(chalk_1.default.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`));
            return;
        }
        let counterparty;
        try {
            counterparty = await utils_1.checkPeerIdInput(query);
        }
        catch (err) {
            console.log(err.message);
            return;
        }
        const channelId = await this.node.paymentChannels.utils.getId(
        /* prettier-ignore */
        await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()), await this.node.paymentChannels.utils.pubKeyToAccountId(counterparty.pubKey.marshal()));
        const tokens = new bignumber_js_1.default((await this.node.paymentChannels.accountBalance).toString()).div(new bignumber_js_1.default(10).pow(this.node.paymentChannels.types.Balance.DECIMALS));
        let funds, tmpFunds;
        const tokenQuestion = `How many ${this.node.paymentChannels.types.Balance.SYMBOL} (${chalk_1.default.magenta(`${tokens.toString()} ${this.node.paymentChannels.types.Balance.SYMBOL}`)} available) shall get staked? : `;
        const exitQuestion = `Do you want to cancel (${chalk_1.default.green('Y')} / ${chalk_1.default.red('n')}) : `;
        do {
            tmpFunds = await new Promise(resolve => rl.question(tokenQuestion, resolve));
            try {
                funds = new bignumber_js_1.default(tmpFunds);
            }
            catch { }
            hopr_utils_1.clearString(tokenQuestion + tmpFunds, rl);
            if (tmpFunds.length == 0) {
                let decision = await new Promise(resolve => rl.question(exitQuestion, resolve));
                if (decision.length == 0 || decision.match(/^y(es)?$/i)) {
                    hopr_utils_1.clearString(exitQuestion + decision, rl);
                    return;
                }
                hopr_utils_1.clearString(exitQuestion + decision, rl);
            }
        } while (funds == null || funds.lte(0) || funds.gt(tokens) || funds.isNaN());
        const channelFunding = new bn_js_1.default(funds.times(new bignumber_js_1.default(10).pow(this.node.paymentChannels.types.Balance.DECIMALS)).toString());
        const isPartyA = this.node.paymentChannels.utils.isPartyA(await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()), await this.node.paymentChannels.utils.pubKeyToAccountId(counterparty.pubKey.marshal()));
        const channelBalance = this.node.paymentChannels.types.ChannelBalance.create(undefined, isPartyA
            ? {
                balance: channelFunding,
                balance_a: channelFunding,
            }
            : {
                balance: channelFunding,
                balance_a: new bn_js_1.default(0),
            });
        const unsubscribe = hopr_utils_1.startDelayedInterval(`Submitted transaction. Waiting for confirmation`);
        try {
            await this.node.paymentChannels.channel.create(this.node.paymentChannels, counterparty.pubKey.marshal(), async () => this.node.paymentChannels.utils.pubKeyToAccountId(await this.node.interactions.payments.onChainKey.interact(counterparty)), channelBalance, (balance) => this.node.interactions.payments.open.interact(counterparty, balance));
            console.log(`${chalk_1.default.green(`Successfully opened channel`)} ${chalk_1.default.yellow(hopr_utils_1.u8aToHex(channelId))}`);
        }
        catch (err) {
            console.log(chalk_1.default.red(err.message));
        }
        unsubscribe();
    }
    complete(line, cb, query) {
        this.node.paymentChannels.channel.getAll(this.node.paymentChannels, async (channel) => (await utils_2.pubKeyToPeerId(await channel.offChainCounterparty)).toB58String(), async (channelIds) => {
            let peerIdStringSet;
            try {
                peerIdStringSet = new Set(await Promise.all(channelIds));
            }
            catch (err) {
                console.log(chalk_1.default.red(err.message));
                return cb(undefined, [[''], line]);
            }
            const peers = [];
            for (const peerInfo of this.node.peerStore.peers.values()) {
                if (utils_1.isBootstrapNode(this.node, peerInfo.id)) {
                    continue;
                }
                if (!peerIdStringSet.has(peerInfo.id.toB58String())) {
                    peers.push(peerInfo.id.toB58String());
                }
            }
            if (peers.length < 1) {
                console.log(chalk_1.default.red(`\nDoesn't know any node to open a payment channel with.`));
                return cb(undefined, [[''], line]);
            }
            const hits = query ? peers.filter((peerId) => peerId.startsWith(query)) : peers;
            return cb(undefined, [hits.length ? hits.map((str) => `open ${str}`) : ['open'], line]);
        });
    }
}
exports.default = OpenChannel;
