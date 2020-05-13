"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.addPubKey = void 0;
const peer_id_1 = __importDefault(require("peer-id"));
const libp2p_crypto_1 = require("libp2p-crypto");
// @ts-ignore
const Multihash = require("multihashes");
/**
 * Takes a peerId and returns a peerId with the public key set to the corresponding
 * public key.
 *
 * @param peerId the PeerId instance that has probably no pubKey set
 */
async function addPubKey(peerId) {
    if (peer_id_1.default.isPeerId(peerId) && peerId.pubKey)
        return peerId;
    peerId.pubKey = await libp2p_crypto_1.keys.unmarshalPublicKey(Multihash.decode(peerId.toBytes()).digest);
    return peerId;
}
exports.addPubKey = addPubKey;
//# sourceMappingURL=addPubKey.js.map