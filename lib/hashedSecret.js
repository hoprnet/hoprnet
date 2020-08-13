"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.HASHED_SECRET_WIDTH = exports.TOTAL_ITERATIONS = exports.GIANT_STEP_WIDTH = void 0;
const types_1 = require("./types");
const debug_1 = __importDefault(require("debug"));
const log = debug_1.default('hopr-core-ethereum:hashedSecret');
const crypto_1 = require("crypto");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const secp256k1_1 = require("secp256k1");
exports.GIANT_STEP_WIDTH = 10000;
exports.TOTAL_ITERATIONS = 100000;
exports.HASHED_SECRET_WIDTH = 27;
class HashedSecret {
    constructor(coreConnector) {
        this.coreConnector = coreConnector;
        this._onChainValuesInitialized = false;
    }
    async submitFromDatabase(nonce) {
        log(`Key is present off-chain but not on-chain, submitting..`);
        let hashedSecret = await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(exports.TOTAL_ITERATIONS - exports.GIANT_STEP_WIDTH)));
        for (let i = 0; i < exports.GIANT_STEP_WIDTH; i++) {
            hashedSecret = await this.coreConnector.utils.hash(hashedSecret.slice(0, exports.HASHED_SECRET_WIDTH));
        }
        await this._submit(hashedSecret, nonce);
    }
    /**
     * generate and set account secret
     */
    async submit(nonce) {
        await this._submit(await this.create(), nonce);
        this._onChainValuesInitialized = true;
    }
    async _submit(hashedSecret, nonce) {
        const account = await this.coreConnector.hoprChannels.methods
            .accounts((await this.coreConnector.account.address).toHex())
            .call();
        if (account.accountX == null || ['0', '0x', '0x'.padEnd(66, '0')].includes(account.accountX)) {
            const uncompressedPubKey = secp256k1_1.publicKeyConvert(this.coreConnector.account.keys.onChain.pubKey, false).slice(1);
            await this.coreConnector.utils.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprChannels.methods.init(hopr_utils_1.u8aToHex(uncompressedPubKey.slice(0, 32)), hopr_utils_1.u8aToHex(uncompressedPubKey.slice(32, 64)), hopr_utils_1.u8aToHex(hashedSecret)), {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: nonce || (await this.coreConnector.account.nonce),
            })).send());
        }
        else {
            // @TODO this is potentially dangerous because it increases the account counter
            await this.coreConnector.utils.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprChannels.methods.setHashedSecret(hopr_utils_1.u8aToHex(hashedSecret)), {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: nonce || (await this.coreConnector.account.nonce),
            })).send());
        }
    }
    /**
     * Checks whether node has an account secret set onchain and offchain
     * @returns a promise resolved true if secret is set correctly
     */
    async check() {
        let [onChainSecret, offChainSecret] = await Promise.all([
            // get onChainSecret
            this.coreConnector.hoprChannels.methods
                .accounts((await this.coreConnector.account.address).toHex())
                .call()
                .then((res) => {
                const hashedSecret = hopr_utils_1.stringToU8a(res.hashedSecret);
                if (hopr_utils_1.u8aEquals(hashedSecret, new Uint8Array(exports.HASHED_SECRET_WIDTH).fill(0x00))) {
                    return undefined;
                }
                return new types_1.Hash(hashedSecret);
            }),
            // get offChainSecret
            this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecret())).catch((err) => {
                if (err.notFound != true) {
                    throw err;
                }
            }),
        ]);
        let hasOffChainSecret = offChainSecret != null;
        let hasOnChainSecret = onChainSecret != null;
        if (hasOffChainSecret && hasOnChainSecret) {
            // make sure that we are able to recover the pre-image
            await this.getPreimage(onChainSecret);
        }
        else if (hasOffChainSecret != hasOnChainSecret) {
            if (hasOffChainSecret) {
                await this.submitFromDatabase();
                hasOnChainSecret = true;
            }
            else {
                log(`Key is present on-chain but not in our database.`);
                if (this.coreConnector.options.debug) {
                    log(`DEBUG mode: Writing debug secret to database`);
                    await this.create();
                    hasOffChainSecret = true;
                }
            }
        }
        this._onChainValuesInitialized = hasOffChainSecret && hasOnChainSecret;
    }
    /**
     * Returns a deterministic secret that is used in debug mode.
     */
    async getDebugAccountSecret() {
        const account = await this.coreConnector.hoprChannels.methods
            .accounts((await this.coreConnector.account.address).toHex())
            .call();
        return (await this.coreConnector.utils.hash(hopr_utils_1.u8aConcat(new Uint8Array([parseInt(account.counter)]), this.coreConnector.account.keys.onChain.pubKey))).slice(0, exports.HASHED_SECRET_WIDTH);
    }
    /**
     * Creates the on-chain secret and stores the intermediate values
     * into the database.
     */
    async create() {
        let onChainSecret = this.coreConnector.options.debug
            ? await this.getDebugAccountSecret()
            : new types_1.Hash(crypto_1.randomBytes(exports.HASHED_SECRET_WIDTH));
        let onChainSecretIntermediary = onChainSecret;
        let dbBatch = this.coreConnector.db.batch();
        for (let i = 0; i < exports.TOTAL_ITERATIONS; i++) {
            if (i % exports.GIANT_STEP_WIDTH == 0) {
                dbBatch = dbBatch.put(Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(i)), Buffer.from(onChainSecretIntermediary));
            }
            onChainSecretIntermediary = new types_1.Hash((await this.coreConnector.utils.hash(onChainSecretIntermediary)).slice(0, exports.HASHED_SECRET_WIDTH));
        }
        await dbBatch.write();
        return onChainSecretIntermediary;
    }
    /**
     * Tries to find a pre-image for the given hash by using the intermediate
     * values from the database.
     * @param hash the hash to find a preImage for
     */
    async getPreimage(hash) {
        if (hash.length != exports.HASHED_SECRET_WIDTH) {
            throw Error(`Invalid length. Expected a Uint8Array with ${exports.HASHED_SECRET_WIDTH} elements but got one with ${hash.length}`);
        }
        let closestIntermediary = exports.TOTAL_ITERATIONS - exports.GIANT_STEP_WIDTH;
        let intermediary;
        let upperBound = exports.TOTAL_ITERATIONS;
        let hashedIntermediary;
        let found = false;
        let index;
        do {
            while (true) {
                try {
                    intermediary = await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(closestIntermediary)));
                    break;
                }
                catch (err) {
                    if (err.notFound) {
                        if (closestIntermediary == 0) {
                            throw Error(`Could not find pre-image`);
                        }
                        else {
                            closestIntermediary -= exports.GIANT_STEP_WIDTH;
                        }
                    }
                    else {
                        throw err;
                    }
                }
            }
            for (let i = 0; i < upperBound - closestIntermediary; i++) {
                hashedIntermediary = (await this.coreConnector.utils.hash(intermediary)).slice(0, exports.HASHED_SECRET_WIDTH);
                if (hopr_utils_1.u8aEquals(hashedIntermediary, hash)) {
                    found = true;
                    index = closestIntermediary + i;
                    break;
                }
                else {
                    intermediary = hashedIntermediary;
                }
            }
            closestIntermediary -= exports.GIANT_STEP_WIDTH;
        } while (!found && closestIntermediary >= 0);
        if (!found) {
            throw Error('notFound');
        }
        return { preImage: new types_1.Hash(intermediary), index };
    }
}
exports.default = HashedSecret;
//# sourceMappingURL=hashedSecret.js.map