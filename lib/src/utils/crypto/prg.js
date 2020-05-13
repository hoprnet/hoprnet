"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PRG = void 0;
const crypto_1 = require("crypto");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const BLOCK_LENGTH = 16;
const KEY_LENGTH = BLOCK_LENGTH;
const IV_LENGTH = 12;
const COUNTER_LENGTH = 4;
const PRG_ALGORITHM = 'aes-128-ctr';
class PRG {
    constructor(key, iv) {
        this.initialised = false;
        this.key = key;
        this.iv = iv;
        this.initialised = true;
    }
    static get IV_LENGTH() {
        return IV_LENGTH;
    }
    static get KEY_LENGTH() {
        return KEY_LENGTH;
    }
    static createPRG(key, iv) {
        if (key.length != KEY_LENGTH || iv.length != IV_LENGTH)
            throw Error(`Invalid input parameters. Expected a key of ${KEY_LENGTH} bytes and an initialization vector of ${IV_LENGTH} bytes.`);
        return new PRG(key, iv);
    }
    digest(start, end) {
        if (!this.initialised) {
            throw Error(`Module not initialized. Please do that first.`);
        }
        if (end < start || end == start) {
            throw Error(`Invalid range parameters. 'start' must be strictly smaller than 'end'.`);
        }
        const firstBlock = Math.floor(start / BLOCK_LENGTH);
        const startOffset = start % BLOCK_LENGTH;
        const lastBlock = Math.ceil(end / BLOCK_LENGTH);
        const lastBlockSize = end % BLOCK_LENGTH;
        const amountOfBlocks = lastBlock - firstBlock;
        const iv = hopr_utils_1.u8aConcat(this.iv, hopr_utils_1.toU8a(firstBlock, COUNTER_LENGTH));
        return new Uint8Array(crypto_1.createCipheriv(PRG_ALGORITHM, this.key, iv).update(new Uint8Array(amountOfBlocks * BLOCK_LENGTH))).subarray(startOffset, amountOfBlocks * BLOCK_LENGTH - (lastBlockSize > 0 ? BLOCK_LENGTH - lastBlockSize : 0));
    }
}
exports.PRG = PRG;
//# sourceMappingURL=prg.js.map