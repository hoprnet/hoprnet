"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.pubKeyToPeerId = void 0;
const peer_id_1 = __importDefault(require("peer-id"));
const libp2p_crypto_1 = require("libp2p-crypto");
const hopr_utils_1 = require("@hoprnet/hopr-utils");
const COMPRESSED_PUBLIC_KEY_LENGTH = 33;
/**
 * Converts a plain compressed ECDSA public key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 *
 * @notice Libp2p stores the keys in format that is derived from `protobuf`.
 * Using `libsecp256k1` directly does not work.
 *
 * @param pubKey the plain public key
 */
function pubKeyToPeerId(pubKey) {
    if (typeof pubKey == 'string') {
        pubKey = hopr_utils_1.stringToU8a(pubKey, COMPRESSED_PUBLIC_KEY_LENGTH);
    }
    if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH) {
        throw Error(`Invalid public key. Expected a buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH} bytes. Got one of ${pubKey.length} bytes.`);
    }
    const secp256k1PubKey = new libp2p_crypto_1.keys.supportedKeys.secp256k1.Secp256k1PublicKey(Buffer.from(pubKey));
    return peer_id_1.default.createFromPubKey(secp256k1PubKey.bytes);
}
exports.pubKeyToPeerId = pubKeyToPeerId;
//# sourceMappingURL=pubKeyToPeerId.js.map