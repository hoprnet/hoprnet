"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TOTAL_ITERATIONS = exports.GIANT_STEP_WIDTH = void 0;
const types_1 = require("./types");
const crypto_1 = require("crypto");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
exports.GIANT_STEP_WIDTH = 10000;
exports.TOTAL_ITERATIONS = 100000;
class HashedSecret {
    constructor(coreConnector) {
        this.coreConnector = coreConnector;
    }
    async create() {
        let onChainSecret = new types_1.Hash(crypto_1.randomBytes(types_1.Hash.SIZE));
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
//# sourceMappingURL=preImage.js.map