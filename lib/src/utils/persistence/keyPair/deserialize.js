"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deserializeKeyPair = void 0;
const rlp_1 = require("rlp");
const crypto_1 = require("crypto");
const peer_id_1 = __importDefault(require("peer-id"));
const _1 = require(".");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
/**
 * Deserializes a serialized key pair and returns a peerId.
 *
 * @notice This method will ask for a password to decrypt the encrypted
 * private key.
 * @notice The decryption of the private key makes use of a memory-hard
 * hash function and consumes therefore a lot of memory.
 *
 * @param encryptedSerializedKeyPair the encoded and encrypted key pair
 */
async function deserializeKeyPair(encryptedSerializedKeyPair, password) {
    const [salt, mac, encodedCiphertext] = rlp_1.decode(encryptedSerializedKeyPair);
    if (salt.length != _1.KEYPAIR_SALT_LENGTH) {
        throw Error('Invalid salt length.');
    }
    const key = crypto_1.scryptSync(password, salt, _1.KEYPAIR_CIPHER_KEY_LENGTH, _1.KEYPAIR_SCRYPT_PARAMS);
    if (!hopr_utils_1.u8aEquals(crypto_1.createHmac(_1.KEYPAIR_MESSAGE_DIGEST_ALGORITHM, key)
        .update(encodedCiphertext)
        .digest(), mac)) {
        throw Error(`Invalid MAC. Ciphertext might have been corrupted`);
    }
    const [iv, ciphertext] = rlp_1.decode(encodedCiphertext);
    if (iv.length != _1.KEYPAIR_IV_LENGTH) {
        throw Error('Invalid IV length.');
    }
    let plaintext = crypto_1.createCipheriv(_1.KEYPAIR_CIPHER_ALGORITHM, key, iv).update(ciphertext);
    return await peer_id_1.default.createFromProtobuf(plaintext);
}
exports.deserializeKeyPair = deserializeKeyPair;
//# sourceMappingURL=deserialize.js.map