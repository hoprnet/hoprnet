"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.serializePeerInfo = void 0;
// @TODO get back to proper types
// const Multiaddr = require('multiaddr')
const rlp_1 = require("rlp");
/**
 * Serializes peerInfos including their multiaddrs.
 * @param peerInfo PeerInfo to serialize
 */
function serializePeerInfo(peerInfo) {
    const result = [peerInfo.id.toBytes(), peerInfo.multiaddrs.toArray().map((multiaddr) => multiaddr.buffer)];
    if (peerInfo.id.pubKey) {
        result.push(peerInfo.id.pubKey.bytes);
    }
    return rlp_1.encode(result);
}
exports.serializePeerInfo = serializePeerInfo;
//# sourceMappingURL=serialize.js.map