"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.serializeKeyPair = void 0;
const rlp_1 = require("rlp");
const crypto_1 = require("crypto");
const _1 = require(".");
/**
 * Serializes a given peerId by serializing the included private key and public key.
 *
 * @param peerId the peerId that should be serialized
 */
async function serializeKeyPair(peerId, password) {
    const salt = crypto_1.randomBytes(_1.KEYPAIR_SALT_LENGTH);
    const key = crypto_1.scryptSync(password, salt, _1.KEYPAIR_CIPHER_KEY_LENGTH, _1.KEYPAIR_SCRYPT_PARAMS);
    const iv = crypto_1.randomBytes(_1.KEYPAIR_IV_LENGTH);
    const ciphertext = crypto_1.createCipheriv(_1.KEYPAIR_CIPHER_ALGORITHM, key, iv).update(peerId.marshal());
    const encodedCipherText = rlp_1.encode([iv, ciphertext]);
    return rlp_1.encode([salt, crypto_1.createHmac(_1.KEYPAIR_MESSAGE_DIGEST_ALGORITHM, key).update(encodedCipherText).digest(), encodedCipherText]);
}
exports.serializeKeyPair = serializeKeyPair;
//# sourceMappingURL=serialize.js.map