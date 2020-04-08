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
        const peerInfos = [];
        for (const peerInfo of this.node.peerStore.peers.values()) {
            if ((!query || peerInfo.id.toB58String().startsWith(query))) {
                peerInfos.push(peerInfo.id.toB58String());
            }
        }
        if (!peerInfos.length) {
            return cb(undefined, [[''], line]);
        }
        return cb(undefined, [peerInfos.map((peerInfo) => `ping ${peerInfo}`), line]);
    }
}
exports.default = Ping;
