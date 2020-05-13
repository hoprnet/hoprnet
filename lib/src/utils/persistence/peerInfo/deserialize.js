"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deserializePeerInfo = void 0;
const rlp_1 = require("rlp");
const peer_id_1 = __importDefault(require("peer-id"));
const peer_info_1 = __importDefault(require("peer-info"));
// @TODO get back to proper types
const Multiaddr = require('multiaddr');
const libp2p_crypto_1 = require("libp2p-crypto");
/**
 * Deserializes a serialized PeerInfo
 * @param arr Uint8Array that contains a serialized PeerInfo
 */
async function deserializePeerInfo(arr) {
    const serialized = rlp_1.decode(Buffer.from(arr));
    const peerId = peer_id_1.default.createFromBytes(serialized[0]);
    if (serialized.length == 3) {
        peerId.pubKey = libp2p_crypto_1.keys.unmarshalPublicKey(serialized[2]);
    }
    const peerInfo = await peer_info_1.default.create(peerId);
    serialized[1].forEach((multiaddr) => peerInfo.multiaddrs.add(Multiaddr(multiaddr)));
    return peerInfo;
}
exports.deserializePeerInfo = deserializePeerInfo;
//# sourceMappingURL=deserialize.js.map