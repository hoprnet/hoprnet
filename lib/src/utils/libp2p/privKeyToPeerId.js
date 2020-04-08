"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const peer_id_1 = __importDefault(require("peer-id"));
const libp2p_crypto_1 = require("libp2p-crypto");
const u8a_1 = require("../u8a");
const secp256k1_1 = __importDefault(require("secp256k1"));
const PRIVKEY_LENGTH = 32;
/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 *
 * @param privKey the plain private key
 */
function privKeyToPeerId(privKey) {
    if (typeof privKey == 'string') {
        privKey = u8a_1.stringToU8a(privKey, PRIVKEY_LENGTH);
    }
    if (privKey.length != PRIVKEY_LENGTH) {
        throw Error(`Invalid private key. Expected a buffer of size ${PRIVKEY_LENGTH} bytes. Got one of ${privKey.length} bytes.`);
    }
    const secp256k1PrivKey = new libp2p_crypto_1.keys.supportedKeys.secp256k1.Secp256k1PrivateKey(Buffer.from(privKey), secp256k1_1.default.publicKeyCreate(privKey));
    return peer_id_1.default.createFromPrivKey(secp256k1PrivKey.bytes);
}
exports.privKeyToPeerId = privKeyToPeerId;
