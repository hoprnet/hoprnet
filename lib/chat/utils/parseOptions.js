"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseOptions = void 0;
const getopts_1 = __importDefault(require("getopts"));
const chalk_1 = __importDefault(require("chalk"));
const peer_info_1 = __importDefault(require("peer-info"));
const peer_id_1 = __importDefault(require("peer-id"));
const multiaddr_1 = __importDefault(require("multiaddr"));
const listConnectors_1 = __importDefault(require("../commands/listConnectors"));
const _1 = require(".");
const __1 = require("..");
const listConnectors = new listConnectors_1.default();
/**
 * Parses the given command-line options and returns a configuration object.
 *
 * @returns
 */
async function parseOptions() {
    const unknownOptions = [];
    let cli_options = getopts_1.default(process.argv.slice(2), {
        boolean: ['debug', 'bootstrapNode', 'help', 'listConnectors', 'verbose'],
        string: ['network', 'password'],
        alias: {
            l: 'listConnectors',
            p: 'password',
            bootstrap: 'bootstrapNode',
            b: 'bootstrapNode',
            h: 'help',
            n: 'network',
            v: 'verbose',
        },
        default: {
            network: 'ethereum',
            bootstrapNode: false,
        },
        unknown: (option) => {
            unknownOptions.push(option);
            return false;
        },
    });
    if (cli_options._.length > 1) {
        console.log(`Found more than the allowed options. Got ${chalk_1.default.yellow(cli_options._.join(', '))}\n`);
        _1.displayHelp();
        process.exit(0);
    }
    let id;
    for (let i = 0; i < cli_options._.length; i++) {
        try {
            const int = parseInt(cli_options._[i]);
            if (isFinite(int)) {
                id = int;
            }
        }
        catch {
            console.log(chalk_1.default.yellow(`Got unknown option '${cli_options._[i]}'.`));
            _1.displayHelp();
            process.exit(0);
        }
    }
    if (unknownOptions.length > 0) {
        console.log(chalk_1.default.yellow(`Got unknown option${unknownOptions.length == 1 ? '' : 's'} [${unknownOptions.join(', ')}]\n`));
        _1.displayHelp();
        process.exit(0);
    }
    if (cli_options.verbose) {
        require('debug').enable('*');
    }
    if (cli_options.help) {
        _1.displayHelp();
        process.exit(0);
    }
    if (cli_options.listConnectors) {
        await listConnectors.execute();
        process.exit();
    }
    if (!__1.knownConnectors.some(connector => connector[1] == cli_options.network)) {
        console.log(`Unknown network! <${chalk_1.default.red(cli_options.network)}>\n`);
        await listConnectors.execute();
        return;
    }
    let connector;
    try {
        connector = (await Promise.resolve().then(() => __importStar(require(`@hoprnet/hopr-core-${cli_options.network}`))))
            .default;
    }
    catch (err) {
        console.log(`Could not find <${chalk_1.default.red(`@hoprnet/hopr-core-${cli_options.network}`)}>. Please make sure it is available under ./node_modules!\n`);
        await listConnectors.execute();
        return;
    }
    if (!cli_options.bootstrapNode && process.env.BOOTSTRAP_SERVERS == null) {
        console.log(chalk_1.default.red('Cannot start HOPR without a bootstrap node'));
    }
    const bootstrapAddresses = process.env.BOOTSTRAP_SERVERS.split(',');
    if (bootstrapAddresses.length == 0) {
        console.log(chalk_1.default.red('Invalid bootstrap servers. Cannot start HOPR without a bootstrap node'));
    }
    let addr, bootstrapServerMap = new Map();
    for (let i = 0; i < bootstrapAddresses.length; i++) {
        addr = multiaddr_1.default(bootstrapAddresses[i].trim());
        let peerInfo = bootstrapServerMap.get(addr.getPeerId());
        if (peerInfo == null) {
            peerInfo = await peer_info_1.default.create(peer_id_1.default.createFromB58String(addr.getPeerId()));
        }
        peerInfo.multiaddrs.add(addr);
        bootstrapServerMap.set(addr.getPeerId(), peerInfo);
    }
    if (process.env[`${cli_options.network.toUpperCase()}_PROVIDER`] == null) {
        throw Error(`Could not find any connector for ${chalk_1.default.magenta(cli_options.network)}. Please specify ${chalk_1.default.yellow(`${cli_options.network.toUpperCase()}_PROVIDER`)} in ${chalk_1.default.yellow('.env')}.`);
    }
    let options = {
        debug: cli_options.debug || false,
        bootstrapNode: cli_options.bootstrapNode,
        network: cli_options.network,
        connector,
        bootstrapServers: [...bootstrapServerMap.values()],
        provider: process.env[`${cli_options.network.toUpperCase()}_PROVIDER`],
        output(encoded) {
            const { latency, msg } = _1.decodeMessage(encoded);
            let str = `\n`;
            str += `===== New message ======\n`;
            str += `Message: ${chalk_1.default.yellow(msg.toString())}\n`;
            str += `Latency: ${chalk_1.default.green(latency.toString())} ms\n`;
            str += `========================\n`;
            console.log(str);
        },
    };
    if (id != null) {
        options.id = id;
    }
    if (cli_options.password) {
        options.password = cli_options.password;
    }
    return options;
}
exports.parseOptions = parseOptions;
//# sourceMappingURL=parseOptions.js.map