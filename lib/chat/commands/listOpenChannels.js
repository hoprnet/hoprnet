"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
const utils_1 = require("../../src/utils");
class ListOpenChannels {
    constructor(node) {
        this.node = node;
    }
    /**
     * Lists all channels that we have with other nodes. Triggered from the CLI.
     */
    async execute() {
        let str = `${chalk_1.default.yellow('ChannelId:'.padEnd(66, ' '))} - ${chalk_1.default.blue('PeerId:')}\n`;
        try {
            await this.node.paymentChannels.channel.getAll(this.node.paymentChannels, async (channel) => {
                const channelId = await channel.channelId;
                if (!channel.counterparty) {
                    str += `${chalk_1.default.yellow(utils_1.u8aToHex(channelId))} - ${chalk_1.default.gray('pre-opened')}`;
                    return;
                }
                const peerId = await utils_1.pubKeyToPeerId(await channel.offChainCounterparty);
                str += `${chalk_1.default.yellow(utils_1.u8aToHex(channelId))} - ${chalk_1.default.blue(peerId.toB58String())}\n`;
                return;
            }, async (promises) => {
                if (promises.length == 0) {
                    str = chalk_1.default.yellow(`  There are currently no open channels.`);
                    return;
                }
                await Promise.all(promises);
                return;
            });
            console.log(str);
            return;
        }
        catch (err) {
            console.log(chalk_1.default.red(err.message));
            return;
        }
    }
    complete(line, cb) {
        cb(undefined, [[''], line]);
    }
}
exports.default = ListOpenChannels;
