"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const utils_1 = require("../utils");
const chalk_1 = __importDefault(require("chalk"));
class Ping {
    constructor(node) {
        this.node = node;
    }
    async execute(query) {
        if (query == null) {
            console.log(chalk_1.default.red(`Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`));
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
        if (utils_1.isBootstrapNode(this.node, peerId)) {
            console.log(chalk_1.default.gray(`Pinging the bootstrap node ...`));
        }
        try {
            const latency = await this.node.ping(peerId);
            console.log(`Pong received in:`, chalk_1.default.magenta(String(latency)), `ms`);
        }
        catch (err) {
            console.log(`Could not ping node. Error was: ${chalk_1.default.red(err.message)}`);
        }
    }
    complete(line, cb, query) {
        const peers = utils_1.getPeers(this.node);
        const peerIds = !query || query.length == 0
            ? peers.map((peer) => peer.toB58String())
            : peers.reduce((acc, peer) => {
                const peerString = peer.toB58String();
                if (peerString.startsWith(query)) {
                    acc.push(peerString);
                }
                return acc;
            }, []);
        if (!peerIds.length) {
            return cb(undefined, [[''], line]);
        }
        return cb(undefined, [peerIds.map((peerId) => `ping ${peerId}`), line]);
    }
}
exports.default = Ping;
//# sourceMappingURL=ping.js.map