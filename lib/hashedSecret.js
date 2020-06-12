"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.TOTAL_ITERATIONS = exports.GIANT_STEP_WIDTH = void 0;
const types_1 = require("./types");
const debug_1 = __importDefault(require("debug"));
const log = debug_1.default('hopr-core-ethereum:hashedSecret');
const crypto_1 = require("crypto");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
exports.GIANT_STEP_WIDTH = 10000;
exports.TOTAL_ITERATIONS = 100000;
class HashedSecret {
    constructor(coreConnector) {
        this.coreConnector = coreConnector;
        this._onChainValuesInitialized = false;
    }
    /**
     * generate and set account secret
     */
    async submit(nonce) {
        const hashedSecret = await this.create();
        await this.coreConnector.utils.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprChannels.methods.setHashedSecret(hopr_utils_1.u8aToHex(hashedSecret)), {
            from: (await this.coreConnector.account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address,
            nonce: nonce || (await this.coreConnector.account.nonce),
        })).send());
        this._onChainValuesInitialized = true;
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
                .then((res) => hopr_utils_1.stringToU8a(res.hashedSecret))
                .then((secret) => {
                if (hopr_utils_1.u8aEquals(secret, new Uint8Array(this.coreConnector.types.Hash.SIZE).fill(0x00))) {
                    return undefined;
                }
                return new types_1.Hash(secret);
            }),
            // get offChainSecret
            this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecret())).catch((err) => {
                if (err.notFound != true) {
                    throw err;
                }
            }),
        ]);
        let hasOffChainSecret = typeof offChainSecret !== 'undefined';
        let hasOnChainSecret = typeof onChainSecret !== 'undefined';
        if (hasOffChainSecret && hasOnChainSecret) {
            try {
                await this.getPreimage(onChainSecret);
            }
            catch (err) {
                throw err;
            }
        }
        else if (hasOffChainSecret != hasOnChainSecret) {
            if (hasOffChainSecret) {
                log(`Key is present off-chain but not on-chain, submitting..`);
                let hashedSecret = await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(exports.TOTAL_ITERATIONS - exports.GIANT_STEP_WIDTH)));
                for (let i = 0; i < exports.GIANT_STEP_WIDTH; i++) {
                    hashedSecret = await this.coreConnector.utils.hash(hashedSecret);
                }
                // @TODO this potentially dangerous because it increases the account counter
                await this.coreConnector.utils.waitForConfirmation((await this.coreConnector.signTransaction(this.coreConnector.hoprChannels.methods.setHashedSecret(hopr_utils_1.u8aToHex(hashedSecret)), {
                    from: (await this.coreConnector.account.address).toHex(),
                    to: this.coreConnector.hoprChannels.options.address,
                    nonce: await this.coreConnector.account.nonce,
                })).send());
                hasOnChainSecret = true;
            }
            else {
                log(`Key is present on-chain but not in our database.`);
                if (this.coreConnector.options.debug) {
                    this.create();
                    hasOffChainSecret = true;
                }
            }
        }
        this._onChainValuesInitialized = hasOffChainSecret && hasOnChainSecret;
    }
    /**
     * Returns a deterministic secret that is used in debug mode.
     */
    getDebugAccountSecret() {
        return crypto_1.createHash('sha256').update(this.coreConnector.account.keys.onChain.pubKey).digest();
    }
    /**
     * Creates the on-chain secret and stores the intermediate values
     * into the database.
     */
    async create() {
        let onChainSecret = new types_1.Hash(this.coreConnector.options.debug ? this.getDebugAccountSecret() : crypto_1.randomBytes(types_1.Hash.SIZE));
        let onChainSecretIntermediary = onChainSecret;
        let dbBatch = this.coreConnector.db.batch();
        for (let i = 0; i < exports.TOTAL_ITERATIONS; i++) {
            if (i % exports.GIANT_STEP_WIDTH == 0) {
                dbBatch = dbBatch.put(Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(i)), Buffer.from(onChainSecretIntermediary));
            }
            onChainSecretIntermediary = await this.coreConnector.utils.hash(onChainSecretIntermediary);
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
                hashedIntermediary = await this.coreConnector.utils.hash(intermediary);
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
        return { preImage: intermediary, index };
    }
}
exports.default = HashedSecret;
//# sourceMappingURL=hashedSecret.js.map