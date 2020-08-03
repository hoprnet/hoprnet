"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const dotenv_1 = __importDefault(require("dotenv"));
// @ts-ignore
const dotenvExpand = require('dotenv-expand');
const env = dotenv_1.default.config();
dotenvExpand(env);
const chalk_1 = __importDefault(require("chalk"));
const readline_1 = __importDefault(require("readline"));
const hopr_core_1 = __importDefault(require("@hoprnet/hopr-core"));
const clear_1 = __importDefault(require("clear"));
const utils_1 = require("./utils");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const commands_1 = __importDefault(require("./commands"));
const dependancies_1 = __importDefault(require("./utils/dependancies"));
const logo_1 = require("./logo");
const SPLIT_OPERAND_QUERY_REGEX = /([\w\-]+)(?:\s+)?([\w\s\-.]+)?/;
// Name our process 'hopr'
process.title = 'hopr';
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
            return cb(undefined, [utils_1.keywords.map((entry) => entry[0]), line]);
        }
        const [command, query] = line.trim().split(SPLIT_OPERAND_QUERY_REGEX).slice(1);
        if (command == null || command === '') {
            return cb(undefined, [utils_1.keywords.map((entry) => entry[0]), line]);
        }
        switch (command.trim()) {
            case 'send':
                await commands.sendMessage.complete(line, cb, query);
                break;
            case 'open':
                await commands.openChannel.complete(line, cb, query);
                break;
            case 'close':
                commands.closeChannel.complete(line, cb, query);
                break;
            case 'ping': {
                commands.ping.complete(line, cb, query);
                break;
            }
            case 'tickets': {
                await commands.tickets.complete(line, cb, query);
            }
            default:
                const hits = utils_1.keywords.reduce((acc, keyword) => {
                    if (keyword[0].startsWith(line)) {
                        acc.push(keyword[0]);
                    }
                    return acc;
                }, []);
                return cb(undefined, [hits.length ? hits : utils_1.keywords.map((keyword) => keyword[0]), line]);
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
        const answer = await new Promise((resolve) => rl.question(question, resolve));
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
            rl.prompt();
            return;
        }
        const [command, query] = input.trim().split(SPLIT_OPERAND_QUERY_REGEX).slice(1);
        if (command == null) {
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
            case 'version':
                await commands.version.execute();
                break;
            case 'tickets':
                await commands.tickets.execute(query);
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
    logo_1.renderHoprLogo();
    console.log(`Welcome to ${chalk_1.default.bold('HOPR')}!\n`);
    console.log(`Chat Version: ${chalk_1.default.bold(dependancies_1.default['@hoprnet/hopr-chat'])}`);
    console.log(`Core Version: ${chalk_1.default.bold(dependancies_1.default['@hoprnet/hopr-core'])}`);
    console.log(`Utils Version: ${chalk_1.default.bold(dependancies_1.default['@hoprnet/hopr-utils'])}`);
    console.log(`Connector Version: ${chalk_1.default.bold(dependancies_1.default['@hoprnet/hopr-core-connector-interface'])}\n`);
    console.log(`Bootstrap Servers: ${chalk_1.default.bold(process.env['BOOTSTRAP_SERVERS'])}\n`);
    let options;
    try {
        options = await utils_1.parseOptions();
    }
    catch (err) {
        console.log(err.message + '\n');
        return;
    }
    try {
        node = await hopr_core_1.default.create(options);
    }
    catch (err) {
        console.log(chalk_1.default.red(err.message));
        process.exit(1);
    }
    console.log('Successfully started HOPR Chat.\n');
    console.log(`Your HOPR Chat node is available at the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`);
    console.log('Use the “help” command to see which commands are available.\n');
    if (options.bootstrapNode) {
        runAsBootstrapNode();
    }
    else {
        runAsRegularNode();
    }
}
main();
//# sourceMappingURL=index.js.map