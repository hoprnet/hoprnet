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
exports.Utils = exports.Types = void 0;
const crypto_1 = require("crypto");
const web3_1 = __importDefault(require("web3"));
const HoprChannels_json_1 = __importDefault(require("@hoprnet/hopr-ethereum/build/extracted/abis/HoprChannels.json"));
const HoprToken_json_1 = __importDefault(require("@hoprnet/hopr-ethereum/build/extracted/abis/HoprToken.json"));
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const chalk_1 = __importDefault(require("chalk"));
const channel_1 = __importDefault(require("./channel"));
const ticket_1 = __importDefault(require("./ticket"));
const indexer_1 = __importDefault(require("./indexer"));
const dbkeys = __importStar(require("./dbKeys"));
const types = __importStar(require("./types"));
const utils = __importStar(require("./utils"));
const constants = __importStar(require("./constants"));
const config = __importStar(require("./config"));
let HoprEthereum = /** @class */ (() => {
    class HoprEthereum {
        constructor(db, self, account, web3, network, hoprChannels, hoprToken, options) {
            this.db = db;
            this.self = self;
            this.account = account;
            this.web3 = web3;
            this.network = network;
            this.hoprChannels = hoprChannels;
            this.hoprToken = hoprToken;
            this.options = options;
            this._status = 'uninitialized';
            this.dbKeys = dbkeys;
            this.utils = utils;
            this.types = types;
            this.constants = constants;
            this.channel = channel_1.default;
            this.CHAIN_NAME = 'HOPR on Ethereum';
            this.ticket = ticket_1.default;
            this.indexer = new indexer_1.default(this);
            this.signTransaction = utils.TransactionSigner(web3, self.privateKey);
            this.log = utils.Log();
        }
        /**
         * @returns the current balances of the account associated with this node (HOPR)
         */
        get nonce() {
            return new Promise(async (resolve, reject) => {
                try {
                    let nonce;
                    // 'first' call
                    if (typeof this._nonce === 'undefined') {
                        this._nonce = {
                            getTransactionCount: this.web3.eth.getTransactionCount(this.account.toHex()),
                            virtualNonce: 0,
                            nonce: undefined,
                        };
                        nonce = await this._nonce.getTransactionCount;
                    }
                    // called while 'first' call hasnt returned
                    else if (typeof this._nonce.nonce === 'undefined') {
                        this._nonce.virtualNonce += 1;
                        const virtualNonce = this._nonce.virtualNonce;
                        nonce = await this._nonce.getTransactionCount.then((count) => {
                            return count + virtualNonce;
                        });
                    }
                    // called after 'first' call has returned
                    else {
                        nonce = this._nonce.nonce + 1;
                    }
                    this._nonce.nonce = nonce;
                    return resolve(nonce);
                }
                catch (err) {
                    return reject(err.message);
                }
            });
        }
        /**
         * Returns the current balances of the account associated with this node (HOPR)
         * @returns a promise resolved to Balance
         */
        get accountBalance() {
            return this.hoprToken.methods
                .balanceOf(this.account.toHex())
                .call()
                .then((res) => new types.Balance(res));
        }
        /**
         * Returns the current native balance (ETH)
         * @returns a promise resolved to Balance
         */
        get accountNativeBalance() {
            return this.web3.eth.getBalance(this.account.toHex()).then((res) => new types.NativeBalance(res));
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
            return this.setAccountSecret(nonce);
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
                const responses = await Promise.all([
                    // confirm web3 is connected
                    this.checkWeb3(),
                    // initialize account secret
                    this.initializeAccountSecret(),
                    // start channels indexing
                    this.indexer.start(),
                ]);
                const allOk = responses.every((r) => !!r);
                if (!allOk) {
                    throw Error('could not initialize connector');
                }
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
         * Initializes node's account secret, if it doesn't exist
         * it will generate one.
         * @returns a promise resolved true if account secret is set correctly
         */
        async initializeAccountSecret() {
            try {
                this.log('Initializing account secret');
                const ok = await this.checkAccountSecret();
                if (!ok) {
                    this.log('Setting account secret..');
                    await this.setAccountSecret();
                }
                this.log(chalk_1.default.green('Account secret initialized!'));
                return true;
            }
            catch (err) {
                this.log(chalk_1.default.red(`error initializing account secret: ${err.message}`));
                // special message for testnet
                if ([constants.ERRORS.OOF_ETH, constants.ERRORS.OOF_HOPR].includes(err.message) &&
                    ['private', 'kovan'].includes(this.network)) {
                    console.log(`Congratulations - your HOPR testnet node is ready to go!\n` +
                        `Please fund your Ethereum Kovan account ${chalk_1.default.yellow(this.account.toHex())} with some Kovan ETH and Kovan HOPR test tokens\n` +
                        `You can request Kovan ETH from ${chalk_1.default.blue('https://faucet.kovan.network')}\n` +
                        `For Kovan HOPR test tokens visit our Telegram channel at ${chalk_1.default.blue('https://t.me/hoprnet')}\n`);
                    process.exit();
                }
                return false;
            }
        }
        /**
         * Checks whether node has an account secret set onchain and offchain
         * @returns a promise resolved true if secret is set correctly
         */
        async checkAccountSecret() {
            let offChainSecret;
            let onChainSecret;
            // retrieve offChain secret
            try {
                offChainSecret = await this.db.get(Buffer.from(dbkeys.OnChainSecret()));
            }
            catch (err) {
                if (err.notFound != true) {
                    throw err;
                }
                offChainSecret = undefined;
            }
            // retrieve onChain secret
            onChainSecret = await this.hoprChannels.methods
                .accounts(this.account.toHex())
                .call()
                .then((res) => hopr_utils_1.stringToU8a(res.hashedSecret))
                .then((secret) => {
                if (hopr_utils_1.u8aEquals(secret, new Uint8Array(types.Hash.SIZE).fill(0x00))) {
                    return undefined;
                }
                return secret;
            });
            let hasOffChainSecret = typeof offChainSecret !== 'undefined';
            let hasOnChainSecret = typeof onChainSecret !== 'undefined';
            if (hasOffChainSecret !== hasOnChainSecret) {
                if (hasOffChainSecret) {
                    this.log(`Key is present off-chain but not on-chain, submitting..`);
                    await utils.waitForConfirmation((await this.signTransaction(this.hoprChannels.methods.setHashedSecret(hopr_utils_1.u8aToHex(offChainSecret)), {
                        from: this.account.toHex(),
                        to: this.hoprChannels.options.address,
                        nonce: await this.nonce,
                    })).send());
                    hasOnChainSecret = true;
                }
                else {
                    this.log(`Key is present on-chain but not in our database.`);
                    if (this.options.debug) {
                        await this.db.put(Buffer.from(dbkeys.OnChainSecret()), Buffer.from(this.getDebugAccountSecret()));
                        hasOffChainSecret = true;
                    }
                    else {
                        throw Error(`Key is present on-chain but not in our database.`);
                    }
                }
            }
            return hasOffChainSecret && hasOnChainSecret;
        }
        /**
         * generate and set account secret
         */
        async setAccountSecret(nonce) {
            let secret;
            if (this.options.debug) {
                secret = this.getDebugAccountSecret();
            }
            else {
                secret = new Uint8Array(crypto_1.randomBytes(32));
            }
            const dbPromise = this.db.put(Buffer.from(this.dbKeys.OnChainSecret()), Buffer.from(secret.slice()));
            for (let i = 0; i < 500; i++) {
                secret = await this.utils.hash(secret);
            }
            await Promise.all([
                await utils.waitForConfirmation((await this.signTransaction(this.hoprChannels.methods.setHashedSecret(hopr_utils_1.u8aToHex(secret)), {
                    from: this.account.toHex(),
                    to: this.hoprChannels.options.address,
                    nonce: nonce || (await this.nonce),
                })).send()),
                dbPromise,
            ]);
        }
        /**
         * Checks whether web3 connection is alive
         * @returns a promise resolved true if web3 connection is alive
         */
        async checkWeb3() {
            try {
                const isListening = await this.web3.eth.net.isListening();
                if (!isListening)
                    throw Error('web3 is not connected');
                return true;
            }
            catch (err) {
                this.log(chalk_1.default.red(`error checking web3: ${err.message}`));
                return false;
            }
        }
        getDebugAccountSecret() {
            return crypto_1.createHash('sha256').update(this.self.publicKey).digest();
        }
        /**
         * Creates an uninitialised instance.
         *
         * @param db database instance
         * @param seed that is used to derive that on-chain identity
         * @param options.id Id of the demo account
         * @param options.provider provider URI that is used to connect to the blockchain
         * @param options.debug debug mode, will generate account secrets using account's public key
         * @returns a promise resolved to the connector
         */
        static async create(db, seed, options) {
            const usingSeed = typeof seed !== 'undefined';
            const usingOptions = typeof options !== 'undefined';
            if (!usingSeed && !usingOptions) {
                throw Error("'seed' or 'options' must be provided");
            }
            if (usingOptions && typeof options.id !== 'undefined' && options.id > config.DEMO_ACCOUNTS.length) {
                throw Error(`Unable to find demo account for index '${options.id}'. Please make sure that you have specified enough demo accounts.`);
            }
            const providerUri = (options === null || options === void 0 ? void 0 : options.provider) || config.DEFAULT_URI;
            const privateKey = usingSeed ? seed : hopr_utils_1.stringToU8a(config.DEMO_ACCOUNTS[options.id]);
            const publicKey = await utils.privKeyToPubKey(privateKey);
            const address = await utils.pubKeyToAccountId(publicKey);
            const provider = new web3_1.default.providers.WebsocketProvider(providerUri, {
                reconnect: {
                    auto: true,
                    delay: 1000,
                    maxAttempts: 10,
                },
            });
            const web3 = new web3_1.default(provider);
            const account = new types.AccountId(address);
            const network = await utils.getNetworkId(web3);
            if (typeof config.CHANNELS_ADDRESSES[network] === 'undefined') {
                throw Error(`channel contract address from network ${network} not found`);
            }
            if (typeof config.TOKEN_ADDRESSES[network] === 'undefined') {
                throw Error(`token contract address from network ${network} not found`);
            }
            const hoprChannels = new web3.eth.Contract(HoprChannels_json_1.default, config.CHANNELS_ADDRESSES[network]);
            const hoprToken = new web3.eth.Contract(HoprToken_json_1.default, config.TOKEN_ADDRESSES[network]);
            const coreConnector = new HoprEthereum(db, {
                privateKey,
                publicKey,
                onChainKeyPair: {
                    privateKey,
                    publicKey,
                },
            }, account, web3, network, hoprChannels, hoprToken, { debug: (options === null || options === void 0 ? void 0 : options.debug) || false });
            coreConnector.log(`using ethereum address ${account.toHex()}`);
            // begin initializing
            coreConnector.initialize().catch((err) => {
                coreConnector.log(chalk_1.default.red(`coreConnector.initialize error: ${err.message}`));
            });
            coreConnector.start();
            return coreConnector;
        }
    }
    HoprEthereum.constants = constants;
    return HoprEthereum;
})();
exports.default = HoprEthereum;
exports.Types = types;
exports.Utils = utils;
