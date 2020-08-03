"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const chalk_1 = __importDefault(require("chalk"));
const utils_1 = require("../utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const constants_1 = require("@hoprnet/hopr-core/lib/constants");
const readline_1 = __importDefault(require("readline"));
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
            console.log(chalk_1.default.red(`Invalid arguments. Expected 'send <peerId>'. Received '${query}'`));
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
        const manualPath = process.env.MULTIHOP
            ? await (async () => {
                const manualIntermediateNodesQuestion = `Do you want to manually set the intermediate nodes? (${chalk_1.default.green('y')}, ${chalk_1.default.red('N')}): `;
                const manualIntermediateNodesAnswer = await new Promise((resolve) => rl.question(manualIntermediateNodesQuestion, resolve));
                hopr_utils_1.clearString(manualIntermediateNodesQuestion + manualIntermediateNodesAnswer, rl);
                return (manualIntermediateNodesAnswer.toLowerCase().match(/^y(es)?$/i) || '').length >= 1;
            })()
            : true;
        const messageQuestion = `${chalk_1.default.yellow(`Type in your message and press ENTER to send:`)}\n`;
        const message = await new Promise((resolve) => rl.question(messageQuestion, resolve));
        hopr_utils_1.clearString(messageQuestion + message, rl);
        console.log(`Sending message to ${chalk_1.default.blue(query)} ...`);
        try {
            if (manualPath) {
                await this.node.sendMessage(utils_1.encodeMessage(message), peerId, async () => {
                    if (process.env.MULTIHOP)
                        return this.selectIntermediateNodes(rl, peerId);
                    return [];
                });
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
        const peerIds = utils_1.getPeers(this.node, {
            noBootstrapNodes: true,
        }).map((peerId) => peerId.toB58String());
        const validPeerIds = query ? peerIds.filter((peerId) => peerId.startsWith(query)) : peerIds;
        if (!validPeerIds.length) {
            console.log(chalk_1.default.red(`\nDoesn't know any other node except apart from bootstrap node${this.node.bootstrapServers.length == 1 ? '' : 's'}!`));
            return cb(undefined, [[''], line]);
        }
        return cb(undefined, [validPeerIds.map((peerId) => `send ${peerId}`), line]);
    }
    async selectIntermediateNodes(rl, destination) {
        let done = false;
        let selected = [];
        // ask for node until user fills all nodes or enters an empty id
        while (!done) {
            console.log(chalk_1.default.yellow(`Please select intermediate node ${selected.length}: (leave empty to exit)`));
            const lastSelected = selected.length > 0 ? selected[selected.length - 1] : this.node.peerInfo.id;
            const openChannels = await utils_1.getOpenChannels(this.node, lastSelected);
            const validPeers = openChannels.map((peer) => peer.toB58String());
            if (validPeers.length === 0) {
                console.log(chalk_1.default.yellow(`No peers with open channels found, you may enter a peer manually.`));
            }
            // detach prompt
            // @ts-ignore
            const oldPrompt = rl._prompt;
            // @ts-ignore
            const oldCompleter = rl.completer;
            const oldListeners = rl.listeners('line');
            rl.removeAllListeners('line');
            // attach new prompt
            rl.setPrompt('');
            // @ts-ignore
            rl.completer = (line, cb) => {
                return cb(undefined, [validPeers.filter((peerId) => peerId.startsWith(line)), line]);
            };
            // wait for peerId to be selected
            const peerId = await new Promise((resolve) => rl.on('line', async (query) => {
                if (query == null || query === '\n' || query === '' || query.length == 0) {
                    rl.removeAllListeners('line');
                    return resolve(undefined);
                }
                let peerId;
                try {
                    peerId = await utils_1.checkPeerIdInput(query);
                }
                catch (err) {
                    console.log(chalk_1.default.red(err.message));
                }
                readline_1.default.moveCursor(process.stdout, -rl.line, -1);
                readline_1.default.clearLine(process.stdout, 0);
                console.log(chalk_1.default.blue(query));
                return resolve(peerId);
            }));
            // no peerId selected, stop selecting nodes
            if (typeof peerId === 'undefined') {
                done = true;
            }
            // check if peerId selected is destination peerId
            else if (destination.equals(peerId)) {
                console.log(chalk_1.default.yellow(`Peer selected is same as destination peer.`));
            }
            // check if peerId selected is already in the list
            else if (selected.find((p) => p.equals(peerId))) {
                console.log(chalk_1.default.yellow(`Peer is already an intermediate peer.`));
            }
            // update list
            else {
                selected.push(peerId);
            }
            // we selected all peers
            if (selected.length >= constants_1.MAX_HOPS - 1) {
                done = true;
            }
            // reattach prompt
            rl.setPrompt(oldPrompt);
            // @ts-ignore
            rl.completer = oldCompleter;
            // @ts-ignore
            oldListeners.forEach((oldListener) => rl.on('line', oldListener));
        }
        return selected;
    }
}
exports.default = SendMessage;
//# sourceMappingURL=sendMessage.js.map