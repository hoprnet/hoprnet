"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const bn_js_1 = __importDefault(require("bn.js"));
const chalk_1 = __importDefault(require("chalk"));
const utils_1 = require("../utils");
const utils_2 = require("../../src/utils");
class CloseChannel {
    constructor(node) {
        this.node = node;
    }
    async execute(query) {
        if (query == null) {
            console.log(chalk_1.default.red(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`));
            return;
        }
        let peerId;
        try {
            peerId = await utils_1.checkPeerIdInput(query);
        }
        catch (err) {
            console.log(err.message);
            return;
        }
        const unsubscribe = utils_1.startDelayedInterval(`Initiated settlement. Waiting for finalisation`);
        let channel;
        try {
            channel = await this.node.paymentChannels.channel.create(this.node.paymentChannels, peerId.pubKey.marshal(), async (counterparty) => this.node.interactions.payments.onChainKey.interact(await utils_2.pubKeyToPeerId(counterparty)));
            await channel.initiateSettlement();
            console.log(`${chalk_1.default.green(`Successfully closed channel`)} ${chalk_1.default.yellow(utils_2.u8aToHex(await channel.channelId))}. Received ${chalk_1.default.magenta(new bn_js_1.default(0).toString())} ${this.node.paymentChannels.types.Balance.SYMBOL}.`);
        }
        catch (err) {
            console.log(chalk_1.default.red(err.message));
        }
        await new Promise(resolve => setTimeout(() => {
            unsubscribe();
            process.stdout.write('\n');
            resolve();
        }));
    }
    complete(line, cb, query) {
        this.node.paymentChannels.channel.getAll(this.node.paymentChannels, async (channel) => (await utils_2.pubKeyToPeerId(await channel.offChainCounterparty)).toB58String(), async (peerIdPromises) => {
            let peerIdStrings;
            try {
                peerIdStrings = await Promise.all(peerIdPromises);
            }
            catch (err) {
                console.log(chalk_1.default.red(err.message));
                return cb(undefined, [[''], line]);
            }
            if (peerIdStrings != null && peerIdStrings.length < 1) {
                console.log(chalk_1.default.red(`\nCannot close any channel because there are not any open ones and/or channels were opened by a third party.`));
                return cb(undefined, [[''], line]);
            }
            const hits = query ? peerIdStrings.filter((peerId) => peerId.startsWith(query)) : peerIdStrings;
            return cb(undefined, [hits.length ? hits.map((str) => `close ${str}`) : ['close'], line]);
        });
    }
}
exports.default = CloseChannel;
