"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PRP = void 0;
const crypto_1 = require("crypto");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const INTERMEDIATE_KEY_LENGTH = 32;
const INTERMEDIATE_IV_LENGTH = 16;
const HASH_LENGTH = 32;
const KEY_LENGTH = 4 * INTERMEDIATE_KEY_LENGTH; // 128 Bytes
const IV_LENGTH = 4 * INTERMEDIATE_IV_LENGTH; // Bytes
const MIN_LENGTH = HASH_LENGTH; // Bytes
const HASH_ALGORITHM = 'blake2s256';
const CIPHER_ALGORITHM = 'chacha20';
class PRP {
    constructor(key, iv) {
        this.initialised = false;
        if (key.length != KEY_LENGTH) {
            throw Error(`Invalid key. Expected ${Uint8Array.name} of size ${KEY_LENGTH} bytes but got a ${typeof key} of ${key.length} bytes.`);
        }
        if (iv.length != IV_LENGTH) {
            throw Error(`Invalid initialisation vector. Expected ${Uint8Array.name} of size ${IV_LENGTH} bytes but got a ${typeof key} of ${key.length} bytes..`);
        }
        this.k1 = key.subarray(0, INTERMEDIATE_KEY_LENGTH);
        this.k2 = key.subarray(INTERMEDIATE_KEY_LENGTH, 2 * INTERMEDIATE_KEY_LENGTH);
        this.k3 = key.subarray(2 * INTERMEDIATE_KEY_LENGTH, 3 * INTERMEDIATE_KEY_LENGTH);
        this.k4 = key.subarray(3 * INTERMEDIATE_KEY_LENGTH, 4 * INTERMEDIATE_KEY_LENGTH);
        this.iv1 = iv.subarray(0, INTERMEDIATE_IV_LENGTH);
        this.iv2 = iv.subarray(INTERMEDIATE_IV_LENGTH, 2 * INTERMEDIATE_IV_LENGTH);
        this.iv3 = iv.subarray(2 * INTERMEDIATE_IV_LENGTH, 3 * INTERMEDIATE_IV_LENGTH);
        this.iv4 = iv.subarray(3 * INTERMEDIATE_IV_LENGTH, 4 * INTERMEDIATE_IV_LENGTH);
        this.initialised = true;
    }
    static get KEY_LENGTH() {
        return KEY_LENGTH;
    }
    static get IV_LENGTH() {
        return IV_LENGTH;
    }
    static get MIN_LENGTH() {
        return MIN_LENGTH;
    }
    static createPRP(key, iv) {
        return new PRP(key, iv);
    }
    permutate(plaintext) {
        if (!this.initialised) {
            throw Error(`Uninitialised. Provide key and iv first.`);
        }
        if (plaintext.length < MIN_LENGTH) {
            throw Error(`Expected plaintext with a length of a least '${MIN_LENGTH}' bytes. Got '${plaintext.length}'.`);
        }
        const data = plaintext;
        encrypt(data, this.k1, this.iv1);
        hash(data, this.k2, this.iv2);
        encrypt(data, this.k3, this.iv3);
        hash(data, this.k4, this.iv4);
        return plaintext;
    }
    inverse(ciphertext) {
        if (!this.initialised) {
            throw Error(`Uninitialised. Provide key and iv first.`);
        }
        if (ciphertext.length < MIN_LENGTH) {
            throw Error(`Expected ciphertext with a length of a least '${MIN_LENGTH}' bytes. Got '${ciphertext.length}'.`);
        }
        const data = ciphertext;
        hash(data, this.k4, this.iv4);
        encrypt(data, this.k3, this.iv3);
        hash(data, this.k2, this.iv2);
        encrypt(data, this.k1, this.iv1);
        return data;
    }
}
exports.PRP = PRP;
function hash(data, k, iv) {
    const hash = crypto_1.createHmac(HASH_ALGORITHM, Buffer.concat([k, iv], INTERMEDIATE_KEY_LENGTH + INTERMEDIATE_IV_LENGTH));
    hash.update(data.subarray(HASH_LENGTH));
    hopr_utils_1.u8aXOR(true, data.subarray(0, HASH_LENGTH), hash.digest());
}
function encrypt(data, k, iv) {
    const cipher = crypto_1.createCipheriv(CIPHER_ALGORITHM, hopr_utils_1.u8aXOR(false, k, data.subarray(0, HASH_LENGTH)), iv);
    const ciphertext = cipher.update(data.subarray(HASH_LENGTH));
    data.set(ciphertext, HASH_LENGTH);
}
//# sourceMappingURL=prp.js.map