"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
const utils_1 = require("../utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const utils_2 = require("../../src/utils");
const constants_1 = require("../../src/constants");
const readline_1 = __importDefault(require("readline"));
const getOpenChannels = async (node) => {
    return new Promise((resolve, reject) => {
        let openChannels = [];
        try {
            node.paymentChannels.channel.getAll(node.paymentChannels, async (channel) => {
                const peerId = await utils_2.pubKeyToPeerId(await channel.offChainCounterparty);
                const peerIdStr = peerId.toB58String();
                if (!openChannels.includes(peerIdStr)) {
                    openChannels.push(peerIdStr);
                }
                return;
            }, async (promises) => {
                await Promise.all(promises);
                return resolve(openChannels);
            });
        }
        catch (err) {
            return reject(err);
        }
    });
};
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
        // @ts-ignore
        const oldCompleter = rl.completer;
        // @ts-ignore
        rl.completer = undefined;
        const manualIntermediateNodesQuestion = `Do you want to manually set the intermediate nodes? (${chalk_1.default.green('y')}, ${chalk_1.default.red('N')}): `;
        const manualIntermediateNodesAnswer = await new Promise(resolve => rl.question(manualIntermediateNodesQuestion, resolve));
        hopr_utils_1.clearString(manualIntermediateNodesQuestion + manualIntermediateNodesAnswer, rl);
        const manualPath = (manualIntermediateNodesAnswer.toLowerCase().match(/^y(es)?$/i) || '').length;
        const messageQuestion = `${chalk_1.default.yellow(`Type in your message and press ENTER to send:`)}\n`;
        const message = await new Promise(resolve => rl.question(messageQuestion, resolve));
        hopr_utils_1.clearString(messageQuestion + message, rl);
        console.log(`Sending message to ${chalk_1.default.blue(query)} ...`);
        try {
            if (manualPath) {
                await this.node.sendMessage(utils_1.encodeMessage(message), peerId, () => this.selectIntermediateNodes(rl, query));
            }
            else {
                await this.node.sendMessage(utils_1.encodeMessage(message), peerId);
            }
        }
        catch (err) {
            console.log(chalk_1.default.red(err.message));
        }
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
    async selectIntermediateNodes(rl, destination) {
        console.log(chalk_1.default.yellow('Please select the intermediate nodes: (hint use tabCompletion)'));
        const openChannels = await getOpenChannels(this.node);
        let localPeers = [];
        for (let peer of this.node.peerStore.peers.values()) {
            let peerIdString = peer.id.toB58String();
            if (peerIdString !== destination && openChannels.includes(peerIdString)) {
                localPeers.push(peerIdString);
            }
        }
        if (localPeers.length === 0) {
            console.log(chalk_1.default.yellow('Cannot find peers in which you have open payment channels with.'));
        }
        // @ts-ignore
        const oldPrompt = rl._prompt;
        // @ts-ignore
        const oldCompleter = rl.completer;
        const oldListeners = rl.listeners('line');
        rl.removeAllListeners('line');
        rl.setPrompt('');
        // @ts-ignore
        rl.completer = (line, cb) => {
            return cb(undefined, [localPeers.filter(localPeer => localPeer.startsWith(line)), line]);
        };
        let selected = [];
        await new Promise(resolve => rl.on('line', async (query) => {
            if (query == null || query === '\n' || query === '' || query.length == 0) {
                rl.removeAllListeners('line');
                return resolve();
            }
            let peerId;
            try {
                peerId = await utils_1.checkPeerIdInput(query);
            }
            catch (err) {
                console.log(chalk_1.default.red(err.message));
            }
            const peerIndex = localPeers.findIndex((str) => str == query);
            readline_1.default.moveCursor(process.stdout, -rl.line, -1);
            readline_1.default.clearLine(process.stdout, 0);
            console.log(chalk_1.default.blue(query));
            selected.push(peerId);
            localPeers.splice(peerIndex, 1);
            if (selected.length >= constants_1.MAX_HOPS - 1) {
                rl.removeAllListeners('line');
                return resolve();
            }
        }));
        rl.setPrompt(oldPrompt);
        // @ts-ignore
        rl.completer = oldCompleter;
        // @ts-ignore
        oldListeners.forEach(oldListener => rl.on('line', oldListener));
        return selected;
    }
}
exports.default = SendMessage;
