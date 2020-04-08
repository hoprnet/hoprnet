"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
const utils_1 = require("../utils");
class SendMessage {
    constructor(node) {
        this.node = node;
    }
    /**
     * Encapsulates the functionality that is executed once the user decides to send a message.
     * @param query peerId string to send message to
     */
    async execute(rl, query) {
        if (query == null) {
            console.log(chalk_1.default.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`));
            return;
        }
        let peerId;
        try {
            peerId = await utils_1.checkPeerIdInput(query);
        }
        catch (err) {
            console.log(chalk_1.default.red(err.message));
            return;
        }
        rl.question(`Sending message to ${chalk_1.default.blue(peerId.toB58String())}\nType in your message and press ENTER to send:\n`, async (message) => {
            try {
                await this.node.sendMessage(utils_1.encodeMessage(message), peerId);
            }
            catch (err) {
                console.log(chalk_1.default.red(err.message));
            }
        });
    }
    async complete(line, cb, query) {
        const peerInfos = [];
        for (const peerInfo of this.node.peerStore.peers.values()) {
            if ((!query || peerInfo.id.toB58String().startsWith(query)) && !utils_1.isBootstrapNode(this.node, peerInfo.id)) {
                peerInfos.push(peerInfo);
            }
        }
        if (!peerInfos.length) {
            console.log(chalk_1.default.red(`\nDoesn't know any other node except apart from bootstrap node${this.node.bootstrapServers.length == 1 ? '' : 's'}!`));
            return cb(undefined, [[''], line]);
        }
        return cb(undefined, [peerInfos.map((peerInfo) => `send ${peerInfo.id.toB58String()}`), line]);
    }
}
exports.default = SendMessage;
