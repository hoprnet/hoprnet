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
    if (mod != null) for (var k in mod) if (k !== "default" && Object.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Utils = exports.Types = void 0;
const web3_1 = __importDefault(require("web3"));
const HoprChannels_json_1 = __importDefault(require("@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json"));
const HoprToken_json_1 = __importDefault(require("@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json"));
const chalk_1 = __importDefault(require("chalk"));
const channel_1 = require("./channel");
const types_1 = __importDefault(require("./types"));
const tickets_1 = __importDefault(require("./tickets"));
const indexer_1 = __importDefault(require("./indexer"));
const dbkeys = __importStar(require("./dbKeys"));
const utils = __importStar(require("./utils"));
const constants = __importStar(require("./constants"));
const config = __importStar(require("./config"));
const account_1 = __importDefault(require("./account"));
const hashedSecret_1 = __importDefault(require("./hashedSecret"));
class HoprEthereum {
    constructor(db, web3, chainId, network, hoprChannels, hoprToken, options, privateKey, publicKey) {
        this.db = db;
        this.web3 = web3;
        this.chainId = chainId;
        this.network = network;
        this.hoprChannels = hoprChannels;
        this.hoprToken = hoprToken;
        this.options = options;
        this._status = 'uninitialized';
        this.dbKeys = dbkeys;
        this.utils = utils;
        this.constants = constants;
        this.CHAIN_NAME = 'HOPR on Ethereum';
        this.hashedSecret = new hashedSecret_1.default(this);
        this.account = new account_1.default(this, privateKey, publicKey);
        this.indexer = new indexer_1.default(this);
        this.tickets = new tickets_1.default(this);
        this.types = new types_1.default();
        this.channel = new channel_1.ChannelFactory(this);
        this.signTransaction = utils.TransactionSigner(web3, privateKey);
        this.log = utils.Log();
    }
    /**
     * Initialises the connector, e.g. connect to a blockchain node.
     */
    async start() {
        this.log('Starting connector..');
        if (typeof this._starting !== 'undefined') {
            this.log('Connector is already starting..');
            return this._starting;
        }
        else if (this._status === 'started') {
            this.log('Connector has already started');
            return;
        }
        else if (this._status === 'uninitialized' && typeof this._initializing === 'undefined') {
            this.log('Connector was asked to start but state was not asked to initialize, initializing..');
            this.initialize().catch((err) => {
                this.log(chalk_1.default.red(err.message));
            });
        }
        this._starting = Promise.resolve()
            .then(async () => {
            // agnostic check if connector can start
            while (this._status !== 'initialized') {
                await utils.wait(1 * 1e3);
            }
            // restart
            await this.indexer.start();
            this._status = 'started';
            this.log(chalk_1.default.green('Connector started'));
        })
            .catch((err) => {
            this.log(chalk_1.default.red(`Connector failed to start: ${err.message}`));
        })
            .finally(() => {
            this._starting = undefined;
        });
        return this._starting;
    }
    /**
     * Stops the connector.
     */
    async stop() {
        this.log('Stopping connector..');
        if (typeof this._stopping !== 'undefined') {
            this.log('Connector is already stopping..');
            return this._stopping;
        }
        else if (this._status === 'stopped') {
            this.log('Connector has already stopped');
            return;
        }
        this._stopping = Promise.resolve()
            .then(async () => {
            // connector is starting
            if (typeof this._starting !== 'undefined') {
                this.log("Connector will stop once it's started");
                // @TODO: cancel initializing & starting
                await this._starting;
            }
            await this.indexer.stop();
            this._status = 'stopped';
            this.log(chalk_1.default.green('Connector stopped'));
        })
            .catch((err) => {
            this.log(chalk_1.default.red(`Connector failed to stop: ${err.message}`));
        })
            .finally(() => {
            this._stopping = undefined;
        });
        return this._stopping;
    }
    get started() {
        return this._status === 'started';
    }
    /**
     * Initializes the on-chain values of our account.
     * @param nonce optional specify nonce of the account to run multiple queries simultaneously
     */
    async initOnchainValues(nonce) {
        await this.hashedSecret.submit(nonce);
    }
    /**
     * Initializes connector, insures that connector is only initialized once,
     * and it only resolves once it's done initializing.
     */
    async initialize() {
        this.log('Initializing connector..');
        if (typeof this._initializing !== 'undefined') {
            this.log('Connector is already initializing..');
            return this._initializing;
        }
        else if (this._status === 'initialized') {
            this.log('Connector has already initialized');
            return;
        }
        else if (this._status !== 'uninitialized') {
            throw Error(`invalid status '${this._status}', could not initialize`);
        }
        this._initializing = Promise.resolve()
            .then(async () => {
            // initialize stuff
            await Promise.all([
                // confirm web3 is connected
                this.checkWeb3(),
                // start channels indexing
                this.indexer.start(),
                // check account secret
                this.hashedSecret.check(),
            ]);
            this._status = 'initialized';
            this.log(chalk_1.default.green('Connector initialized'));
        })
            .catch((err) => {
            this.log(`Connector failed to initialize: ${err.message}`);
        })
            .finally(() => {
            this._initializing = undefined;
        });
        return this._initializing;
    }
    /**
     * Checks whether web3 connection is alive
     * @returns a promise resolved true if web3 connection is alive
     */
    async checkWeb3() {
        let isListening;
        try {
            isListening = await this.web3.eth.net.isListening();
        }
        catch (err) {
            this.log(chalk_1.default.red(`error checking web3: ${err.message}`));
        }
        if (!isListening) {
            throw Error('web3 is not connected');
        }
    }
    static get constants() {
        return constants;
    }
    /**
     * Creates an uninitialised instance.
     *
     * @param db database instance
     * @param seed that is used to derive that on-chain identity
     * @param options.provider provider URI that is used to connect to the blockchain
     * @param options.debug debug mode, will generate account secrets using account's public key
     * @returns a promise resolved to the connector
     */
    static async create(db, seed, options) {
        const providerUri = (options === null || options === void 0 ? void 0 : options.provider) || config.DEFAULT_URI;
        const provider = new web3_1.default.providers.WebsocketProvider(providerUri, {
            reconnect: {
                auto: true,
                delay: 1000,
                maxAttempts: 10,
            },
        });
        const web3 = new web3_1.default(provider);
        const [chainId, publicKey] = await Promise.all([
            /* prettier-ignore */
            utils.getChainId(web3),
            utils.privKeyToPubKey(seed),
        ]);
        const network = utils.getNetworkName(chainId);
        if (typeof config.CHANNELS_ADDRESSES[network] === 'undefined') {
            throw Error(`channel contract address from network ${network} not found`);
        }
        if (typeof config.TOKEN_ADDRESSES[network] === 'undefined') {
            throw Error(`token contract address from network ${network} not found`);
        }
        const hoprChannels = new web3.eth.Contract(HoprChannels_json_1.default, config.CHANNELS_ADDRESSES[network]);
        const hoprToken = new web3.eth.Contract(HoprToken_json_1.default, config.TOKEN_ADDRESSES[network]);
        const coreConnector = new HoprEthereum(db, web3, chainId, network, hoprChannels, hoprToken, { debug: (options === null || options === void 0 ? void 0 : options.debug) || false }, seed, publicKey);
        coreConnector.log(`using ethereum address ${(await coreConnector.account.address).toHex()}`);
        // begin initializing
        coreConnector.initialize().catch((err) => {
            coreConnector.log(chalk_1.default.red(`coreConnector.initialize error: ${err.message}`));
        });
        coreConnector.start();
        return coreConnector;
    }
}
exports.default = HoprEthereum;
exports.Types = types_1.default;
exports.Utils = utils;
//# sourceMappingURL=index.js.map