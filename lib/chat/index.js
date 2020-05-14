"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.knownConnectors = exports.cli_options = exports.keywords = void 0;
const dotenv_1 = __importDefault(require("dotenv"));
// @ts-ignore
const dotenvExpand = require('dotenv-expand');
const env = dotenv_1.default.config();
dotenvExpand(env);
const chalk_1 = __importDefault(require("chalk"));
const readline_1 = __importDefault(require("readline"));
const src_1 = __importDefault(require("../src"));
const figlet_1 = __importDefault(require("figlet"));
const clear_1 = __importDefault(require("clear"));
const utils_1 = require("./utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const commands_1 = __importDefault(require("./commands"));
const SPLIT_OPERAND_QUERY_REGEX = /([\w\-]+)(?:\s+)?([\w\s\-.]+)?/;
// Allowed keywords
exports.keywords = [
    ['open', 'opens a payment channel'],
    ['send', 'sends a message to another party'],
    ['quit', 'stops the node and terminates the process'],
    ['crawl', 'crawls the network and tries to find other nodes'],
    ['openChannels', 'lists all currently open channels'],
    ['closeAll', 'closes all payment channel of this node'],
    ['myAddress', 'shows the address of this node'],
    ['balance', 'shows our current balance'],
    ['listConnectors', 'lists all installed blockchain connectors'],
    ['ping', 'pings another node to check its availability'],
    ['help', 'shows this help page'],
].sort((a, b) => a[0].localeCompare(b[0], 'en', { sensitivity: 'base' }));
// Allowed CLI options
exports.cli_options = [
    ['-b', '--bootstrapNode', undefined, 'starts HOPR as a bootstrap node'],
    ['-n', '--network', '<connector>', 'starts HOPR with blockchain connector <connector>'],
    ['-h', '--help', undefined, 'shows this help page'],
    ['-l', '--listConnectors', undefined, 'shows all available connectors'],
    ['-p', '--password', '<password>', 'start HOPR with <password>'],
    ['-v', '--verbose', undefined, 'show debug info'],
    [undefined, '--debug', undefined, 'run HOPR in debug mode [insecure, only used for development]'],
].sort((a, b) => {
    let tmpA;
    let tmpB;
    if (a[0] === undefined) {
        tmpA = a[1].slice(2);
    }
    else {
        tmpA = a[0].slice(1);
    }
    if (b[0] === undefined) {
        tmpB = b[1].slice(2);
    }
    else {
        tmpB = b[0].slice(1);
    }
    return tmpA.localeCompare(tmpB, 'en', { sensitivity: 'base' });
});
// Name our process 'hopr'
process.title = 'hopr';
/**
 * Alphabetical list of known connectors.
 */
exports.knownConnectors = [
    /* prettier-ignore */
    ['@hoprnet/hopr-core-ethereum', 'ethereum'],
    ['@hoprnet/hopr-core-polkadot', 'polkadot'],
];
let node;
/**
 * Completes a given input with possible endings. Used for convenience.
 *
 * @param line the current input
 * @param cb to returns the suggestions
 */
function tabCompletion(commands) {
    return async (line, cb) => {
        if (line == null || line == '') {
            return cb(undefined, [exports.keywords.map(entry => entry[0]), line]);
        }
        const [command, query] = line
            .trim()
            .split(SPLIT_OPERAND_QUERY_REGEX)
            .slice(1);
        if (command == null || command === '') {
            return cb(undefined, [exports.keywords.map(entry => entry[0]), line]);
        }
        switch (command.trim()) {
            case 'send':
                await commands.sendMessage.complete(line, cb, query);
                break;
            case 'open':
                await commands.openChannel.complete(line, cb, query);
                break;
            case 'close':
                await commands.closeChannel.complete(line, cb, query);
                break;
            case 'ping': {
                await commands.ping.complete(line, cb, query);
            }
            default:
                const hits = exports.keywords.reduce((acc, keyword) => {
                    if (keyword[0].startsWith(line)) {
                        acc.push(keyword[0]);
                    }
                    return acc;
                }, []);
                return cb(undefined, [hits.length ? hits : exports.keywords.map(keyword => keyword[0]), line]);
        }
    };
}
async function runAsRegularNode() {
    const commands = new commands_1.default(node);
    let rl = readline_1.default.createInterface({
        input: process.stdin,
        output: process.stdout,
        completer: tabCompletion(commands),
    });
    rl.on('SIGINT', async () => {
        const question = `Are you sure you want to exit? (${chalk_1.default.green('y')}, ${chalk_1.default.red('N')}): `;
        const answer = await new Promise(resolve => rl.question(question, resolve));
        if (answer.match(/^y(es)?$/i)) {
            hopr_utils_1.clearString(question, rl);
            await commands.stopNode.execute();
            return;
        }
        rl.prompt();
    });
    rl.once('close', async () => {
        await commands.stopNode.execute();
        return;
    });
    console.log(`Connecting to bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}...`);
    rl.on('line', async (input) => {
        if (input == null || input == '') {
            console.log(chalk_1.default.red('Unknown command!'));
            rl.prompt();
            return;
        }
        const [command, query] = input
            .trim()
            .split(SPLIT_OPERAND_QUERY_REGEX)
            .slice(1);
        if (command == null) {
            console.log(chalk_1.default.red('Unknown command!'));
            rl.prompt();
            return;
        }
        switch (command.trim()) {
            case 'balance':
                await commands.printBalance.execute();
                break;
            case 'close':
                await commands.closeChannel.execute(query);
                break;
            case 'crawl':
                await commands.crawl.execute();
                break;
            case 'help':
                commands.listCommands.execute();
                break;
            case 'quit':
                await commands.stopNode.execute();
                break;
            case 'openChannels':
                await commands.listOpenChannels.execute();
                break;
            case 'open':
                await commands.openChannel.execute(rl, query);
                break;
            case 'send':
                await commands.sendMessage.execute(rl, query);
                break;
            case 'listConnectors':
                await commands.listConnectors.execute();
                break;
            case 'myAddress':
                await commands.printAddress.execute();
                break;
            case 'ping':
                await commands.ping.execute(query);
                break;
            default:
                console.log(chalk_1.default.red('Unknown command!'));
                break;
        }
        rl.prompt();
    });
    rl.prompt();
}
function runAsBootstrapNode() {
    console.log(`... running as bootstrap node!.`);
    node.on('peer:connect', (peer) => {
        console.log(`Incoming connection from ${chalk_1.default.blue(peer.id.toB58String())}.`);
    });
    process.once('exit', async () => {
        await node.down();
        return;
    });
}
async function main() {
    clear_1.default();
    console.log(figlet_1.default.textSync('HOPRnet.eth', {
        horizontalLayout: 'fitted',
    }));
    console.log(`Welcome to ${chalk_1.default.bold('HOPR')}!\n`);
    let options;
    try {
        options = await utils_1.parseOptions();
    }
    catch (err) {
        console.log(err.message + '\n');
        return;
    }
    try {
        node = await src_1.default.create(options);
    }
    catch (err) {
        console.log(chalk_1.default.red(err.message));
        process.exit(1);
    }
    console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs
        .toArray()
        .join('\n ')}\n`);
    if (options.bootstrapNode) {
        runAsBootstrapNode();
    }
    else {
        runAsRegularNode();
    }
}
main();
//# sourceMappingURL=index.js.map